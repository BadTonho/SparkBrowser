use crate::commands::tabs::create_tab;
use crate::state::browser_state::{BrowserState, DEFAULT_HOME_URL};
use crate::state::tab_model::TabInfo;
use crate::state::workspace_model::WorkspaceItem;
use crate::window::webview_manager::{navigate_content_webview, sync_all_webviews};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn get_workspaces(state: State<'_, BrowserState>) -> Result<Vec<WorkspaceItem>, String> {
    Ok(state
        .workspaces
        .lock()
        .map(|ws| ws.clone())
        .unwrap_or_else(|_| WorkspaceItem::default_workspaces()))
}

#[tauri::command]
pub async fn get_active_workspace(state: State<'_, BrowserState>) -> Result<String, String> {
    Ok(state
        .active_workspace_id
        .lock()
        .map(|id| id.clone())
        .unwrap_or_else(|_| "ws-personal".to_string()))
}

#[tauri::command]
pub async fn switch_workspace(
    app: AppHandle,
    state: State<'_, BrowserState>,
    workspace_id: String,
) -> Result<Vec<TabInfo>, String> {
    if let Ok(mut active_ws) = state.active_workspace_id.lock() {
        *active_ws = workspace_id.clone();
    }

    let existing_tab = {
        state.tabs.lock().ok().and_then(|mut tabs| {
            if let Some(found) = tabs.iter_mut().find(|t| t.workspace_id == workspace_id) {
                found.is_hibernated = false;
                found.last_active_epoch = TabInfo::current_epoch_secs();
                Some(found.clone())
            } else {
                None
            }
        })
    };

    if let Some(tab) = existing_tab {
        if let Ok(mut active_id) = state.active_tab_id.lock() {
            *active_id = tab.id;
        }
        let _ = navigate_content_webview(&app, &tab.url);
        sync_all_webviews(&app, &state)?;
        state.persist_session();

        let snapshot = state.snapshot_tabs();
        let _ = app.emit("tabs-updated", &snapshot);
        let _ = app.emit("content-url-changed", &tab.url);
        return Ok(snapshot);
    }

    create_tab(app, state, Some(DEFAULT_HOME_URL.to_string())).await
}
