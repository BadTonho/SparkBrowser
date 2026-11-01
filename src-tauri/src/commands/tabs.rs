use crate::database::tab_usage_repo;
use crate::state::browser_state::{BrowserState, TabSnapshot, DEFAULT_HOME_URL};
use crate::state::tab_model::TabInfo;
use crate::state::workspace_model::DEFAULT_WORKSPACE_ID;
use crate::window::webview_manager::{
    ensure_active_content_webview, resolve_input_to_url_with_search_engine, search_engine_home_url,
    sync_all_webviews,
};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn get_tabs(state: State<'_, BrowserState>) -> Result<TabSnapshot, String> {
    Ok(state.snapshot_packet())
}

#[tauri::command]
pub async fn create_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    url: Option<String>,
    search_engine: Option<String>,
) -> Result<TabSnapshot, String> {
    let mut span = state.diagnostics.span("tabs.create");
    let _operation = state.tab_operations.lock().await;
    let result = create_tab_inner(&app, &state, url, search_engine);
    span.finish(result.is_ok(), 0);
    result
}

pub(super) fn create_tab_inner(
    app: &AppHandle,
    state: &BrowserState,
    url: Option<String>,
    search_engine: Option<String>,
) -> Result<TabSnapshot, String> {
    let raw_target = url.unwrap_or_else(|| DEFAULT_HOME_URL.to_string());
    let resolved_url = resolve_input_to_url_with_search_engine(
        &raw_target,
        search_engine.as_deref().unwrap_or("google"),
    )?
    .to_string();
    let shield_enabled = *state.shield_enabled.lock().map_err(|e| e.to_string())?;
    if crate::privacy::adblock_engine::should_block_url(&resolved_url, shield_enabled) {
        return Err("Navigation blocked by the privacy shield".to_string());
    }

    let new_tab_id = {
        let mut counter = state.next_tab_counter.lock().map_err(|e| e.to_string())?;
        let id = format!("tab-{}", *counter);
        *counter += 1;
        id
    };

    let active_ws_id = state
        .active_workspace_id
        .lock()
        .map(|ws| ws.clone())
        .unwrap_or_else(|_| DEFAULT_WORKSPACE_ID.to_string());

    let previous_active = state
        .active_tab_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let new_tab = TabInfo::new(new_tab_id.clone(), resolved_url.clone(), active_ws_id, true);

    {
        if let Ok(mut tabs) = state.tabs.lock() {
            tabs.push(new_tab);
        }
    }
    state.record_new_tab_activation(&new_tab_id)?;

    state.select_tab(&new_tab_id)?;
    if let Err(error) = ensure_active_content_webview(app) {
        state
            .tabs
            .lock()
            .map_err(|e| e.to_string())?
            .retain(|tab| tab.id != new_tab_id);
        if !previous_active.is_empty() {
            let _ = state.select_tab(&previous_active);
        }
        return Err(error);
    }
    sync_all_webviews(app, state)?;
    state.persist_session()?;

    let snapshot = state.snapshot_packet();
    let _ = app.emit("tabs-updated", &snapshot);
    let _ = app.emit("content-url-changed", &resolved_url);

    Ok(snapshot)
}

#[tauri::command]
pub async fn switch_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
) -> Result<TabSnapshot, String> {
    let mut span = state.diagnostics.span("tabs.switch");
    let _operation = state.tab_operations.lock().await;
    let result = switch_tab_inner(&app, &state, tab_id);
    span.finish(result.is_ok(), 0);
    result
}

pub(super) fn switch_tab_inner(
    app: &AppHandle,
    state: &BrowserState,
    tab_id: String,
) -> Result<TabSnapshot, String> {
    let previous_active_id = state
        .active_tab_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let previous_tab = state
        .tabs
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|t| t.id == previous_active_id)
        .cloned();
    let selected = state.select_tab(&tab_id)?;
    let active_url = selected.url.clone();
    if let Err(error) = ensure_active_content_webview(app) {
        if let Some(previous) = previous_tab {
            let _ = state.select_tab(&previous.id);
        }
        return Err(error);
    }

    if previous_active_id != tab_id {
        state.record_tab_activation(&tab_id)?;
    }

    sync_all_webviews(app, state)?;
    state.persist_session()?;

    let snapshot = state.snapshot_packet();
    let stats = state.snapshot_privacy_stats();
    let _ = app.emit("tabs-updated", &snapshot);
    let _ = app.emit("privacy-stats-updated", &stats);
    if !active_url.is_empty() {
        let _ = app.emit("content-url-changed", &active_url);
    }

    Ok(snapshot)
}

#[tauri::command]
pub async fn close_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
    search_engine: Option<String>,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    let active_ws_id = state
        .active_workspace_id
        .lock()
        .map(|ws| ws.clone())
        .unwrap_or_else(|_| DEFAULT_WORKSPACE_ID.to_string());

    let mut remaining_in_ws = 0;
    let mut next_active_id = String::new();
    let mut next_active_url = String::new();
    let current_active = state
        .active_tab_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();

    {
        if let Ok(mut tabs) = state.tabs.lock() {
            if let Some(idx) = tabs.iter().position(|t| t.id == tab_id) {
                tabs.remove(idx);
            }
            let ws_tabs: Vec<&mut TabInfo> = tabs
                .iter_mut()
                .filter(|t| t.workspace_id == active_ws_id)
                .collect();
            remaining_in_ws = ws_tabs.len();
            if let Some(last_ws_tab) = ws_tabs.into_iter().last() {
                if current_active == tab_id {
                    last_ws_tab.is_hibernated = false;
                    last_ws_tab.last_active_epoch = TabInfo::current_epoch_secs();
                }
                next_active_id = last_ws_tab.id.clone();
                next_active_url = last_ws_tab.url.clone();
            }
        }
    }

    if remaining_in_ws == 0 {
        let home_url = search_engine_home_url(search_engine.as_deref()).to_string();
        return create_tab_inner(&app, &state, Some(home_url), search_engine);
    }

    if current_active == tab_id && !next_active_id.is_empty() {
        state.select_tab(&next_active_id)?;
        state.record_tab_activation(&next_active_id)?;
        ensure_active_content_webview(&app)?;
        let _ = app.emit("content-url-changed", &next_active_url);
    }

    sync_all_webviews(&app, &state)?;
    state.persist_session()?;

    let snapshot = state.snapshot_packet();
    let stats = state.snapshot_privacy_stats();
    let _ = app.emit("tabs-updated", &snapshot);
    let _ = app.emit("privacy-stats-updated", &stats);

    Ok(snapshot)
}

#[tauri::command]
pub async fn hibernate_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    let active_id = state
        .active_tab_id
        .lock()
        .map(|id| id.clone())
        .unwrap_or_default();

    if tab_id == active_id {
        return Err("Cannot hibernate the currently active tab".to_string());
    }

    if let Ok(mut tabs) = state.tabs.lock() {
        if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
            if tab.protected_from_hibernation() {
                return Err("This tab is protected from hibernation".into());
            }
            tab.is_hibernated = true;
        }
    }

    sync_all_webviews(&app, &state)?;
    state.persist_session()?;

    let snapshot = state.snapshot_packet();
    let stats = state.snapshot_privacy_stats();
    let _ = app.emit("tabs-updated", &snapshot);
    let _ = app.emit("privacy-stats-updated", &stats);

    Ok(snapshot)
}

#[tauri::command]
pub async fn hibernate_inactive_tabs(
    app: AppHandle,
    state: State<'_, BrowserState>,
    min_idle_secs: Option<u64>,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    let threshold = min_idle_secs.unwrap_or(0);
    let now = TabInfo::current_epoch_secs();
    let active_id = state
        .active_tab_id
        .lock()
        .map(|id| id.clone())
        .unwrap_or_default();

    let mut changed = false;
    if let Ok(mut tabs) = state.tabs.lock() {
        for tab in tabs.iter_mut() {
            if tab.id != active_id
                && !tab.protected_from_hibernation()
                && !tab.is_hibernated
                && now.saturating_sub(tab.last_active_epoch) >= threshold
            {
                tab.is_hibernated = true;
                changed = true;
            }
        }
    }

    changed |= crate::state::memory_policy::enforce_resident_limit(
        &mut state.tabs.lock().map_err(|e| e.to_string())?,
        state
            .resident_limit
            .load(std::sync::atomic::Ordering::Relaxed),
    ) > 0;
    if !changed {
        return Ok(state.snapshot_packet());
    }
    sync_all_webviews(&app, &state)?;
    state.persist_session()?;

    let snapshot = state.snapshot_packet();
    let stats = state.snapshot_privacy_stats();
    let _ = app.emit("tabs-updated", &snapshot);
    let _ = app.emit("privacy-stats-updated", &stats);

    Ok(snapshot)
}

#[tauri::command]
pub async fn auto_hibernate_inactive_tabs(
    app: AppHandle,
    state: State<'_, BrowserState>,
    mode: String,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    const FIVE_HOURS_SECS: u64 = 5 * 60 * 60;
    let mode = match mode.as_str() {
        "5" => Some((5 * 60, false)),
        "10" => Some((10 * 60, false)),
        "30" => Some((30 * 60, false)),
        "auto" => Some((10 * 60, true)),
        "off" => None,
        _ => return Err("Invalid automatic hibernation mode".to_string()),
    };

    let Some((regular_threshold, adaptive)) = mode else {
        let changed = crate::state::memory_policy::enforce_resident_limit(
            &mut state.tabs.lock().map_err(|e| e.to_string())?,
            state
                .resident_limit
                .load(std::sync::atomic::Ordering::Relaxed),
        );
        if changed > 0 {
            sync_all_webviews(&app, &state)?;
            state.persist_session()?;
        }
        return Ok(state.snapshot_packet());
    };

    let now = TabInfo::current_epoch_secs();
    let active_id = state
        .active_tab_id
        .lock()
        .map(|id| id.clone())
        .unwrap_or_default();
    let mut frequent_tabs = std::collections::HashSet::new();

    if adaptive {
        if let Ok(db_guard) = state.db.lock() {
            if let Some(conn) = db_guard.as_ref() {
                tab_usage_repo::prune_expired(conn, now)?;
                frequent_tabs = tab_usage_repo::frequent_tab_ids(conn, now)?;
            }
        }
    }

    let mut changed = false;
    if let Ok(mut tabs) = state.tabs.lock() {
        for tab in tabs.iter_mut() {
            if tab.id == active_id || tab.is_hibernated || tab.protected_from_hibernation() {
                continue;
            }
            let threshold = if adaptive && frequent_tabs.contains(&tab.id) {
                FIVE_HOURS_SECS
            } else {
                regular_threshold
            };
            if now.saturating_sub(tab.last_active_epoch) >= threshold {
                tab.is_hibernated = true;
                changed = true;
            }
        }
    }

    changed |= crate::state::memory_policy::enforce_resident_limit(
        &mut state.tabs.lock().map_err(|e| e.to_string())?,
        state
            .resident_limit
            .load(std::sync::atomic::Ordering::Relaxed),
    ) > 0;
    if !changed {
        return Ok(state.snapshot_packet());
    }
    sync_all_webviews(&app, &state)?;
    state.persist_session()?;
    let snapshot = state.snapshot_packet();
    let stats = state.snapshot_privacy_stats();
    let _ = app.emit("tabs-updated", &snapshot);
    let _ = app.emit("privacy-stats-updated", &stats);
    Ok(snapshot)
}

#[tauri::command]
pub async fn reorder_tabs(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_ids: Vec<String>,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    {
        if let Ok(mut tabs) = state.tabs.lock() {
            let previous = std::mem::take(&mut *tabs);
            let original_ids: Vec<_> = previous.iter().map(|tab| tab.id.clone()).collect();
            let mut by_id: std::collections::HashMap<_, _> = previous
                .into_iter()
                .map(|tab| (tab.id.clone(), tab))
                .collect();
            let mut ordered = Vec::with_capacity(by_id.len());
            for id in tab_ids.iter().chain(original_ids.iter()) {
                if let Some(tab) = by_id.remove(id) {
                    ordered.push(tab);
                }
            }
            *tabs = ordered;
        }
    }

    state.persist_session()?;
    let snapshot = state.snapshot_packet();
    let _ = app.emit("tabs-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn move_tab_to_workspace(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
    workspace_id: String,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    state.move_tab(&tab_id, &workspace_id)?;

    state.persist_session()?;
    let snapshot = state.snapshot_packet();
    let _ = app.emit("tabs-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn set_resident_limit(
    app: AppHandle,
    state: State<'_, BrowserState>,
    limit: usize,
) -> Result<TabSnapshot, String> {
    if !(1..=100).contains(&limit) {
        return Err("Resident limit must be between 1 and 100".into());
    }
    let _operation = state.tab_operations.lock().await;
    state
        .resident_limit
        .store(limit, std::sync::atomic::Ordering::Relaxed);
    sync_all_webviews(&app, &state)?;
    state.persist_session()?;
    let snapshot = state.snapshot_packet();
    let _ = app.emit("tabs-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn set_tab_pinned(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
    pinned: bool,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    state
        .tabs
        .lock()
        .map_err(|e| e.to_string())?
        .iter_mut()
        .find(|tab| tab.id == tab_id)
        .ok_or("Tab was not found")?
        .is_pinned = pinned;
    state.persist_session()?;
    let snapshot = state.snapshot_packet();
    let _ = app.emit("tabs-updated", &snapshot);
    Ok(snapshot)
}
