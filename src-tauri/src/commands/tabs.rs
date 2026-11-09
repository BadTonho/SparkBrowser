use crate::database::tab_usage_repo;
use crate::state::browser_state::{BrowserState, TabSnapshot, DEFAULT_HOME_URL};
use crate::state::workspace_model::DEFAULT_WORKSPACE_ID;
use crate::state::{tab_checkpoint::TabCheckpoint, tab_model::TabInfo};
use crate::window::webview_manager::sync_all_webviews;
use tauri::{AppHandle, Manager, State};

pub fn popup_target_allowed(enabled: bool, url: &tauri::Url) -> bool {
    enabled
        && matches!(url.scheme(), "http" | "https")
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
}

/// Opens an authenticated user-selected link without activating the new tab.
pub fn open_user_link(app: AppHandle, source_tab: String, generation: String, url: tauri::Url) {
    if !popup_target_allowed(true, &url) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let state = app.state::<BrowserState>();
        let _operation = state.tab_operations.lock().await;
        let label = crate::state::tab_model::webview_label_for_tab(&source_tab);
        if !crate::window::navigation_state::is_current_generation(&app, &label, &generation) {
            return;
        }
        let source = state
            .tabs
            .lock()
            .ok()
            .and_then(|tabs| tabs.iter().find(|tab| tab.id == source_tab).cloned());
        let Some(source) = source else {
            return;
        };
        let shield = state.shield_enabled.lock().map(|v| *v).unwrap_or(true);
        if crate::privacy::adblock_engine::site_is_protected(&state, url.as_str())
            && crate::privacy::adblock_engine::should_block_url(url.as_str(), shield)
        {
            return;
        }
        let id = match state.next_tab_counter.lock() {
            Ok(mut next) => {
                let id = format!("tab-{}", *next);
                *next += 1;
                id
            }
            Err(_) => return,
        };
        let result = TabCheckpoint::commit(&state, || {
            let mut tab = if source.is_ephemeral {
                TabInfo::new_ephemeral(
                    id.clone(),
                    url.to_string(),
                    source.workspace_id.clone(),
                    false,
                )
            } else {
                TabInfo::new(
                    id.clone(),
                    url.to_string(),
                    source.workspace_id.clone(),
                    false,
                )
            };
            if !source.is_ephemeral {
                tab.container_id = source.container_id.clone();
            }
            tab.is_hibernated = true;
            state.tabs.lock().map_err(|e| e.to_string())?.push(tab);
            Ok(())
        });
        if result.is_ok() {
            reconcile_committed(&app, &state);
            let _ = crate::security::emit_shell(&app, "tabs-updated", state.snapshot_packet());
        }
    });
}

/// Web pop-ups open as normal browser tabs; internal PiP windows are handled separately.
pub fn open_popup_url(app: AppHandle, source_tab: String, generation: String, url: tauri::Url) {
    if !popup_target_allowed(
        app.state::<BrowserState>()
            .allow_popups
            .load(std::sync::atomic::Ordering::Acquire),
        &url,
    ) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let state = app.state::<BrowserState>();
        let _operation = state.tab_operations.lock().await;
        let label = crate::state::tab_model::webview_label_for_tab(&source_tab);
        if !crate::window::navigation_state::is_current_generation(&app, &label, &generation) {
            return;
        }
        if !popup_target_allowed(
            state
                .allow_popups
                .load(std::sync::atomic::Ordering::Acquire),
            &url,
        ) {
            return;
        }
        let source = state
            .tabs
            .lock()
            .ok()
            .and_then(|tabs| tabs.iter().find(|tab| tab.id == source_tab).cloned());
        if let Some(source) = source {
            let _ = create_tab_inner_ext(
                &app,
                &state,
                Some(url.to_string()),
                None,
                None,
                TabProfile {
                    is_ephemeral: source.is_ephemeral,
                    container_id: source.container_id,
                    workspace_id: Some(source.workspace_id),
                },
            );
        }
    });
}

#[tauri::command]
pub async fn get_tabs(state: State<'_, BrowserState>) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    Ok(state.snapshot_packet())
}

#[tauri::command]
pub async fn get_tabs_memory(
    app: AppHandle,
    state: State<'_, BrowserState>,
) -> Result<std::collections::HashMap<String, u64>, String> {
    let _operation = state.tab_operations.lock().await;
    let tabs = state.tabs.lock().map_err(|e| e.to_string())?.clone();
    let resident_tabs: Vec<_> = tabs.iter().filter(|t| !t.is_hibernated).collect();
    let mut result = std::collections::HashMap::new();
    for tab in &tabs {
        if tab.is_hibernated {
            result.insert(tab.id.clone(), 0);
        }
    }

    let isolated_workspaces = state
        .workspace_isolation
        .load(std::sync::atomic::Ordering::Acquire);
    let candidates: Vec<_> = resident_tabs
        .iter()
        .filter_map(|tab| {
            let label = crate::state::tab_model::webview_label_for_tab(&tab.id);
            let view = app.get_webview(&label)?;
            let source = crate::window::tab_memory::current_webview_source(&view)?;
            let environment_group = if isolated_workspaces {
                tab.workspace_id.clone()
            } else {
                String::new()
            };
            Some((tab.id.clone(), view, source, environment_group))
        })
        .collect();
    let mut resident_sources_by_environment =
        std::collections::HashMap::<String, Vec<String>>::new();
    for (_, _, source, environment_group) in &candidates {
        resident_sources_by_environment
            .entry(environment_group.clone())
            .or_default()
            .push(source.clone());
    }
    let mut measurements = tokio::task::JoinSet::new();
    for (tab_id, view, source, environment_group) in candidates {
        let sources = resident_sources_by_environment
            .get(&environment_group)
            .cloned()
            .unwrap_or_default();
        measurements.spawn(async move {
            let bytes = crate::window::tab_memory::measure_webview_memory(&view, source, sources)
                .await
                .unwrap_or(0);
            (tab_id, bytes)
        });
    }
    while let Some(measurement) = measurements.join_next().await {
        if let Ok((tab_id, bytes)) = measurement {
            result.insert(tab_id, bytes);
        }
    }
    for tab in resident_tabs {
        result.entry(tab.id.clone()).or_insert(0);
    }
    Ok(result)
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

#[tauri::command]
pub async fn create_ephemeral_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    url: Option<String>,
    search_engine: Option<String>,
) -> Result<TabSnapshot, String> {
    let mut span = state.diagnostics.span("tabs.create_ephemeral");
    let _operation = state.tab_operations.lock().await;
    let result = create_tab_inner_ext(
        &app,
        &state,
        url,
        search_engine,
        None,
        TabProfile {
            is_ephemeral: true,
            ..Default::default()
        },
    );
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
    create_tab_inner_ext(app, state, url, search_engine, side, TabProfile::default())
}

#[derive(Default)]
pub(super) struct TabProfile {
    pub is_ephemeral: bool,
    pub container_id: Option<String>,
    pub workspace_id: Option<String>,
}

pub(super) fn create_tab_inner_ext(
    app: &AppHandle,
    state: &BrowserState,
    url: Option<String>,
    search_engine: Option<String>,
    side: Option<crate::state::split_docking::DockSide>,
    profile: TabProfile,
) -> Result<TabSnapshot, String> {
    let TabProfile {
        is_ephemeral,
        container_id,
        workspace_id,
    } = profile;
    let raw_target = url.unwrap_or_else(|| DEFAULT_HOME_URL.to_string());
    let resolved_url = crate::window::url_input::resolve_browser_input(
        app,
        &raw_target,
        search_engine.as_deref().unwrap_or("google"),
    )?
    .to_string();
    let shield_enabled = *state.shield_enabled.lock().map_err(|e| e.to_string())?;
    if crate::privacy::adblock_engine::site_is_protected(state, &resolved_url)
        && crate::privacy::adblock_engine::should_block_url(&resolved_url, shield_enabled)
    {
        return Err("Navigation blocked by the privacy shield".to_string());
    }

    let new_tab_id = {
        let mut counter = state.next_tab_counter.lock().map_err(|e| e.to_string())?;
        let id = format!("tab-{}", *counter);
        *counter += 1;
        id
    };

    let active_ws_id = workspace_id.unwrap_or_else(|| {
        state
            .active_workspace_id
            .lock()
            .map(|ws| ws.clone())
            .unwrap_or_else(|_| DEFAULT_WORKSPACE_ID.to_string())
    });

    TabCheckpoint::commit_with_activation(state, Some((&new_tab_id, true)), || {
        let mut tab_info = if is_ephemeral {
            TabInfo::new_ephemeral(new_tab_id.clone(), resolved_url.clone(), active_ws_id, true)
        } else {
            TabInfo::new(new_tab_id.clone(), resolved_url.clone(), active_ws_id, true)
        };
        if !is_ephemeral {
            tab_info.container_id = container_id;
        }
        state.tabs.lock().map_err(|e| e.to_string())?.push(tab_info);
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
    if previous_active_id != tab_id {
        if let Some(label) = crate::window::fullscreen::active(app) {
            crate::window::fullscreen::exit(app, &label, true);
        }
    }
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
            &state
                .hibernation_exemptions
                .lock()
                .map_err(|e| e.to_string())?,
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
                &state
                    .hibernation_exemptions
                    .lock()
                    .map_err(|e| e.to_string())?,
            );
            return Ok(changed > 0);
        };

        let now = TabInfo::current_epoch_secs();
        let exemptions = state
            .hibernation_exemptions
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
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
                if tab.id == active_id
                    || tab.is_hibernated
                    || tab.protected_from_hibernation()
                    || crate::state::tab_model::hibernation_domain_matches(&tab.url, &exemptions)
                {
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
            &exemptions,
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
    // With workspace isolation, the tab's WebView must never keep writing into
    // the previous workspace's data folder; recreate it on the next sync.
    if state
        .workspace_isolation
        .load(std::sync::atomic::Ordering::Acquire)
    {
        crate::window::webview_manager::close_tab_webview(&app, &tab_id)?;
    }
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
pub async fn set_hibernation_exemptions(
    app: AppHandle,
    state: State<'_, BrowserState>,
    domains: Vec<String>,
) -> Result<TabSnapshot, String> {
    if domains.len() > 256 {
        return Err("settings.invalid_hibernation_domain".into());
    }
    let mut normalized = Vec::with_capacity(domains.len());
    for domain in domains {
        let domain = crate::state::tab_model::normalize_hibernation_domain(&domain)?;
        if !normalized.contains(&domain) {
            normalized.push(domain);
        }
    }
    normalized.sort();

    let _operation = state.tab_operations.lock().await;
    let previous = state
        .hibernation_exemptions
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    *state
        .hibernation_exemptions
        .lock()
        .map_err(|e| e.to_string())? = normalized;
    if let Err(error) = TabCheckpoint::commit(&state, || Ok(())) {
        *state
            .hibernation_exemptions
            .lock()
            .map_err(|e| e.to_string())? = previous;
        return Err(error);
    }
    reconcile_committed(&app, &state);
    let snapshot = state.snapshot_packet();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn set_ask_download_destination(state: State<'_, BrowserState>, enabled: bool) {
    state
        .ask_download_destination
        .store(enabled, std::sync::atomic::Ordering::Release);
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
    if let Some(owner) = crate::window::fullscreen::active(app) {
        let selected = state
            .active_tab_id
            .lock()
            .ok()
            .map(|id| crate::state::tab_model::webview_label_for_tab(&id));
        if selected.as_deref() != Some(owner.as_str()) {
            crate::window::fullscreen::exit(app, &owner, true);
        }
    }
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

#[cfg(test)]
mod popup_tests {
    use super::*;

    #[test]
    fn popup_policy_allows_only_credential_free_web_urls_when_enabled() {
        for value in ["https://example.test/path", "http://example.test/"] {
            assert!(popup_target_allowed(true, &value.parse().unwrap()));
        }
        for value in [
            "about:blank",
            "javascript:alert(1)",
            "file:///tmp/file",
            "https://user:pass@example.test/",
        ] {
            assert!(!popup_target_allowed(true, &value.parse().unwrap()));
        }
        assert!(!popup_target_allowed(
            false,
            &"https://example.test/".parse().unwrap()
        ));
    }
}
