use crate::database::tab_usage_repo;
use crate::state::browser_state::{BrowserState, TabSnapshot, DEFAULT_HOME_URL};
use crate::state::workspace_model::DEFAULT_WORKSPACE_ID;
use crate::state::{tab_checkpoint::TabCheckpoint, tab_model::TabInfo};
use crate::window::webview_manager::sync_all_webviews;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn get_tabs(state: State<'_, BrowserState>) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    Ok(state.snapshot_packet())
}

#[tauri::command]
pub async fn create_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    url: Option<String>,
    search_engine: Option<String>,
    side: Option<crate::state::split_docking::DockSide>,
) -> Result<TabSnapshot, String> {
    let mut span = state.diagnostics.span("tabs.create");
    let _operation = state.tab_operations.lock().await;
    let result = create_tab_inner(&app, &state, url, search_engine, side);
    span.finish(result.is_ok(), 0);
    result
}

pub(super) fn create_tab_inner(
    app: &AppHandle,
    state: &BrowserState,
    url: Option<String>,
    search_engine: Option<String>,
    side: Option<crate::state::split_docking::DockSide>,
) -> Result<TabSnapshot, String> {
    let raw_target = url.unwrap_or_else(|| DEFAULT_HOME_URL.to_string());
    let resolved_url = crate::window::url_input::resolve_browser_input(
        app,
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

    TabCheckpoint::commit_with_activation(state, Some((&new_tab_id, true)), || {
        state
            .tabs
            .lock()
            .map_err(|e| e.to_string())?
            .push(TabInfo::new(
                new_tab_id.clone(),
                resolved_url.clone(),
                active_ws_id,
                true,
            ));
        if let Some(side) = side {
            if let Some(layout) = state
                .split_layouts
                .lock()
                .map_err(|e| e.to_string())?
                .get_mut(
                    &*state
                        .active_workspace_id
                        .lock()
                        .map_err(|e| e.to_string())?,
                )
            {
                layout.group_mut(side).push(new_tab_id.clone());
            }
        }
        state.select_tab(&new_tab_id)?;
        Ok(())
    })?;
    reconcile_committed(app, state);

    let snapshot = state.snapshot_packet();
    let _ = crate::security::emit_shell(app, "tabs-updated", &snapshot);
    let _ = crate::security::emit_shell(app, "content-url-changed", &resolved_url);

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
    let activation = (previous_active_id != tab_id).then_some((tab_id.as_str(), false));
    let active_url = TabCheckpoint::commit_with_activation(state, activation, || {
        let selected = state.select_tab(&tab_id)?;
        Ok(selected.url)
    })?;
    reconcile_committed(app, state);

    let snapshot = state.snapshot_packet();
    let stats = state.snapshot_privacy_stats();
    let _ = crate::security::emit_shell(app, "tabs-updated", &snapshot);
    let _ = crate::security::emit_shell(app, "privacy-stats-updated", &stats);
    if !active_url.is_empty() {
        let _ = crate::security::emit_shell(app, "content-url-changed", &active_url);
    }

    Ok(snapshot)
}

#[tauri::command]
pub async fn close_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
    search_engine: Option<String>,
    focus_content: Option<bool>,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    let restore_focus =
        focus_content.unwrap_or(false) && state.active_tab_id.lock().is_ok_and(|id| *id == tab_id);
    let snapshot = super::closed_tabs::close_one(&app, &state, &tab_id, search_engine.as_deref())?;
    if restore_focus {
        crate::window::webview_manager::focus_active_content_webview(&app)?;
    }
    Ok(snapshot)
}

#[tauri::command]
pub async fn hibernate_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    let before = TabCheckpoint::capture(&state)?;
    let changed = (|| -> Result<bool, String> {
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

        Ok(true)
    })();
    finish_change(&app, &state, before, changed)?;

    let snapshot = state.snapshot_packet();
    let stats = state.snapshot_privacy_stats();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &snapshot);
    let _ = crate::security::emit_shell(&app, "privacy-stats-updated", &stats);

    Ok(snapshot)
}

#[tauri::command]
pub async fn hibernate_inactive_tabs(
    app: AppHandle,
    state: State<'_, BrowserState>,
    min_idle_secs: Option<u64>,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    let before = TabCheckpoint::capture(&state)?;
    let changed = (|| -> Result<bool, String> {
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
        Ok(changed)
    })();
    finish_change(&app, &state, before, changed)?;
    let snapshot = state.snapshot_packet();
    let stats = state.snapshot_privacy_stats();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &snapshot);
    let _ = crate::security::emit_shell(&app, "privacy-stats-updated", &stats);

    Ok(snapshot)
}

#[tauri::command]
pub async fn auto_hibernate_inactive_tabs(
    app: AppHandle,
    state: State<'_, BrowserState>,
    mode: String,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    let before = TabCheckpoint::capture(&state)?;
    let changed = (|| -> Result<bool, String> {
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
            return Ok(changed > 0);
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
        Ok(changed)
    })();
    finish_change(&app, &state, before, changed)?;
    let snapshot = state.snapshot_packet();
    let stats = state.snapshot_privacy_stats();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &snapshot);
    let _ = crate::security::emit_shell(&app, "privacy-stats-updated", &stats);
    Ok(snapshot)
}

#[tauri::command]
pub async fn reorder_tabs(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_ids: Vec<String>,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    TabCheckpoint::commit(&state, || {
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

        Ok(())
    })?;
    reconcile_committed(&app, &state);
    let snapshot = state.snapshot_packet();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &snapshot);
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
    TabCheckpoint::commit(&state, || {
        state.move_tab(&tab_id, &workspace_id)?;

        Ok(())
    })?;
    reconcile_committed(&app, &state);
    let snapshot = state.snapshot_packet();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &snapshot);
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
    TabCheckpoint::commit(&state, || {
        state
            .resident_limit
            .store(limit, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    })?;
    reconcile_committed(&app, &state);
    let snapshot = state.snapshot_packet();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &snapshot);
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
    TabCheckpoint::commit(&state, || {
        state
            .tabs
            .lock()
            .map_err(|e| e.to_string())?
            .iter_mut()
            .find(|tab| tab.id == tab_id)
            .ok_or("Tab was not found")?
            .is_pinned = pinned;
        Ok(())
    })?;
    reconcile_committed(&app, &state);
    let snapshot = state.snapshot_packet();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &snapshot);
    Ok(snapshot)
}

/// A committed session remains authoritative if native reconciliation fails.
pub(super) fn reconcile_committed(app: &AppHandle, state: &BrowserState) {
    if let Err(error) = sync_all_webviews(app, state) {
        let _ = crate::security::emit_shell(
            app,
            "browser-error",
            format!("The session was saved, but a page could not be displayed: {error}"),
        );
    }
}

fn finish_change(
    app: &AppHandle,
    state: &BrowserState,
    before: TabCheckpoint,
    changed: Result<bool, String>,
) -> Result<(), String> {
    match changed.and_then(|changed| {
        if changed {
            state.persist_session()?;
        }
        Ok(changed)
    }) {
        Ok(true) => reconcile_committed(app, state),
        Ok(false) => {}
        Err(error) => {
            before.restore(state)?;
            return Err(error);
        }
    }
    Ok(())
}
