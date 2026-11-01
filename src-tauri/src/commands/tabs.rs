use crate::state::browser_state::{BrowserState, DEFAULT_HOME_URL};
use crate::state::tab_model::TabInfo;
use crate::state::workspace_model::WorkspaceItem;
use crate::state::workspace_model::DEFAULT_WORKSPACE_ID;
use crate::window::webview_manager::{
    build_workspace_switcher_update_script, create_internal_bridge_token, navigate_content_webview,
    resolve_input_to_url, search_engine_home_url, sync_all_webviews, CONTENT_WEBVIEW_LABEL,
    WORKSPACE_SWITCHER_WEBVIEW_LABEL,
};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

const WORKSPACE_SWITCHER_CLOSE_ANIMATION: Duration = Duration::from_millis(140);

#[tauri::command]
pub async fn get_tabs(state: State<'_, BrowserState>) -> Result<Vec<TabInfo>, String> {
    Ok(state.snapshot_tabs())
}

#[tauri::command]
pub async fn create_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    url: Option<String>,
) -> Result<Vec<TabInfo>, String> {
    let raw_target = url.unwrap_or_else(|| DEFAULT_HOME_URL.to_string());
    let resolved_url = resolve_input_to_url(&raw_target)?.to_string();

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

    let new_tab = TabInfo::new(new_tab_id.clone(), resolved_url.clone(), active_ws_id, true);

    {
        if let Ok(mut tabs) = state.tabs.lock() {
            tabs.push(new_tab);
        }
        if let Ok(mut active) = state.active_tab_id.lock() {
            *active = new_tab_id;
        }
    }

    // Navigate the persistent child WebView after all state locks are dropped
    let _ = navigate_content_webview(&app, &resolved_url);
    sync_all_webviews(&app, &state)?;
    state.persist_session();

    let snapshot = state.snapshot_tabs();
    let _ = app.emit("tabs-updated", &snapshot);
    let _ = app.emit("content-url-changed", &resolved_url);

    Ok(snapshot)
}

#[tauri::command]
pub async fn switch_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
) -> Result<Vec<TabInfo>, String> {
    let mut active_url = String::new();
    let mut target_ws_id = String::new();

    {
        if let Ok(mut tabs) = state.tabs.lock() {
            if let Some(found) = tabs.iter_mut().find(|t| t.id == tab_id) {
                found.is_hibernated = false;
                found.last_active_epoch = TabInfo::current_epoch_secs();
                active_url = found.url.clone();
                target_ws_id = found.workspace_id.clone();
            } else {
                return Err(format!("Tab {} not found", tab_id));
            }
        }
        if !target_ws_id.is_empty() {
            if let Ok(mut ws_guard) = state.active_workspace_id.lock() {
                *ws_guard = target_ws_id;
            }
        }
        if let Ok(mut active) = state.active_tab_id.lock() {
            *active = tab_id;
        }
    }

    if !active_url.is_empty() {
        let _ = navigate_content_webview(&app, &active_url);
    }

    sync_all_webviews(&app, &state)?;
    state.persist_session();

    let snapshot = state.snapshot_tabs();
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
) -> Result<Vec<TabInfo>, String> {
    let active_ws_id = state
        .active_workspace_id
        .lock()
        .map(|ws| ws.clone())
        .unwrap_or_else(|_| DEFAULT_WORKSPACE_ID.to_string());

    let mut remaining_in_ws = 0;
    let mut next_active_id = String::new();
    let mut next_active_url = String::new();

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
                last_ws_tab.is_hibernated = false;
                last_ws_tab.last_active_epoch = TabInfo::current_epoch_secs();
                next_active_id = last_ws_tab.id.clone();
                next_active_url = last_ws_tab.url.clone();
            }
        }
    }

    if remaining_in_ws == 0 {
        let home_url = search_engine_home_url(search_engine.as_deref()).to_string();
        return create_tab(app, state, Some(home_url)).await;
    }

    let current_active = state
        .active_tab_id
        .lock()
        .map(|id| id.clone())
        .unwrap_or_default();

    if current_active == tab_id && !next_active_id.is_empty() {
        if let Ok(mut active) = state.active_tab_id.lock() {
            *active = next_active_id;
        }
        let _ = navigate_content_webview(&app, &next_active_url);
        let _ = app.emit("content-url-changed", &next_active_url);
    }

    sync_all_webviews(&app, &state)?;
    state.persist_session();

    let snapshot = state.snapshot_tabs();
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
) -> Result<Vec<TabInfo>, String> {
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
            tab.is_hibernated = true;
        }
    }

    state.persist_session();

    let snapshot = state.snapshot_tabs();
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
) -> Result<Vec<TabInfo>, String> {
    let threshold = min_idle_secs.unwrap_or(0);
    let now = TabInfo::current_epoch_secs();
    let active_id = state
        .active_tab_id
        .lock()
        .map(|id| id.clone())
        .unwrap_or_default();

    if let Ok(mut tabs) = state.tabs.lock() {
        for tab in tabs.iter_mut() {
            if tab.id != active_id
                && !tab.is_hibernated
                && now.saturating_sub(tab.last_active_epoch) >= threshold
            {
                tab.is_hibernated = true;
            }
        }
    }

    state.persist_session();

    let snapshot = state.snapshot_tabs();
    let stats = state.snapshot_privacy_stats();
    let _ = app.emit("tabs-updated", &snapshot);
    let _ = app.emit("privacy-stats-updated", &stats);

    Ok(snapshot)
}

#[tauri::command]
pub async fn set_overlay_visibility(
    app: AppHandle,
    state: State<'_, BrowserState>,
    visible: bool,
) -> Result<(), String> {
    if let Ok(mut overlay) = state.is_overlay_open.lock() {
        *overlay = visible;
    }
    sync_all_webviews(&app, &state)?;
    Ok(())
}

#[tauri::command]
pub async fn set_workspace_switcher_overlay(
    app: AppHandle,
    state: State<'_, BrowserState>,
    visible: bool,
    workspaces: Vec<WorkspaceItem>,
    active_workspace_id: String,
    backdrop_blur_px: u8,
) -> Result<(), String> {
    let webview = app
        .get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL)
        .ok_or_else(|| "Workspace switcher WebView was not found".to_string())?;

    // A fresh token also identifies this close animation, so a stale delayed
    // hide cannot hide the WebView after the picker has been reopened.
    let token = create_internal_bridge_token();
    let script = build_workspace_switcher_update_script(
        &workspaces,
        &active_workspace_id,
        &token,
        visible,
        backdrop_blur_px.min(24),
    )?;

    if let Ok(mut current_token) = state.workspace_switcher_token.lock() {
        *current_token = Some(token.clone());
    }
    if let Ok(mut open) = state.workspace_switcher_open.lock() {
        // Keep the native WebView shown during its exit animation.
        *open = true;
    }

    if let Some(content_webview) = app.get_webview(CONTENT_WEBVIEW_LABEL) {
        let _ = content_webview.eval(&format!(
            "window.__veloceWorkspaceSwitcherOpen = {visible};"
        ));
    }

    if let Err(error) = webview.eval(&script) {
        if let Ok(mut current_token) = state.workspace_switcher_token.lock() {
            *current_token = None;
        }
        if let Ok(mut open) = state.workspace_switcher_open.lock() {
            *open = false;
        }
        if let Some(content_webview) = app.get_webview(CONTENT_WEBVIEW_LABEL) {
            let _ = content_webview.eval("window.__veloceWorkspaceSwitcherOpen = false;");
        }
        let _ = sync_all_webviews(&app, &state);
        return Err(format!("Failed to show workspace switcher: {error}"));
    }

    if !visible {
        let app_for_hide = app.clone();
        let closing_token = token;
        std::thread::spawn(move || {
            std::thread::sleep(WORKSPACE_SWITCHER_CLOSE_ANIMATION);
            let Some(state) = app_for_hide.try_state::<BrowserState>() else {
                return;
            };

            // Serialize the token check and state update with a possible reopen.
            let Ok(mut current_token) = state.workspace_switcher_token.lock() else {
                return;
            };
            if current_token.as_deref() != Some(closing_token.as_str()) {
                return;
            }
            let Ok(mut open) = state.workspace_switcher_open.lock() else {
                return;
            };
            *current_token = None;
            *open = false;
            drop(open);
            drop(current_token);

            let _ = sync_all_webviews(&app_for_hide, &state);
            let another_overlay_open = state
                .is_overlay_open
                .lock()
                .map(|overlay| *overlay)
                .unwrap_or(true);
            let app_is_focused = app_for_hide
                .get_webview_window("main")
                .and_then(|window| window.is_focused().ok())
                .unwrap_or(false);
            if app_is_focused {
                if another_overlay_open {
                    if let Some(main_window) = app_for_hide.get_webview_window("main") {
                        let _ = main_window.set_focus();
                    }
                } else if let Some(content_webview) =
                    app_for_hide.get_webview(CONTENT_WEBVIEW_LABEL)
                {
                    let _ = content_webview.set_focus();
                }
            }
        });
        return Ok(());
    }

    sync_all_webviews(&app, &state)?;
    Ok(())
}

#[tauri::command]
pub async fn reorder_tabs(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_ids: Vec<String>,
) -> Result<Vec<TabInfo>, String> {
    {
        if let Ok(mut tabs) = state.tabs.lock() {
            let mut ordered: Vec<TabInfo> = Vec::with_capacity(tabs.len());
            for id in &tab_ids {
                if let Some(pos) = tabs.iter().position(|t| &t.id == id) {
                    ordered.push(tabs.remove(pos));
                }
            }
            ordered.append(&mut *tabs);
            *tabs = ordered;
        }
    }

    state.persist_session();
    let snapshot = state.snapshot_tabs();
    let _ = app.emit("tabs-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn move_tab_to_workspace(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
    workspace_id: String,
) -> Result<Vec<TabInfo>, String> {
    {
        if let Ok(mut tabs) = state.tabs.lock() {
            if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.workspace_id = workspace_id;
            }
        }
    }

    state.persist_session();
    let snapshot = state.snapshot_tabs();
    let _ = app.emit("tabs-updated", &snapshot);
    Ok(snapshot)
}
