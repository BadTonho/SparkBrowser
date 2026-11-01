use crate::state::browser_state::{BrowserState, ViewportBounds};
use crate::state::tab_model::{derive_title_from_url, TabInfo};
use crate::window::webview_manager::{
    navigate_content_webview, sync_all_webviews, CONTENT_WEBVIEW_LABEL,
};
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub async fn navigate_to(
    app: AppHandle,
    state: State<'_, BrowserState>,
    input: String,
) -> Result<String, String> {
    let url_string = navigate_content_webview(&app, &input)?;

    let active_id = state
        .active_tab_id
        .lock()
        .map(|id| id.clone())
        .unwrap_or_default();

    let new_title = derive_title_from_url(&url_string);

    {
        if let Ok(mut tabs) = state.tabs.lock() {
            if let Some(tab) = tabs.iter_mut().find(|t| t.id == active_id) {
                tab.url = url_string.clone();
                tab.title = new_title.clone();
                tab.is_hibernated = false;
                tab.last_active_epoch = TabInfo::current_epoch_secs();
            }
        }
    }

    state.record_history_visit(&new_title, &url_string);
    state.persist_session();

    let snapshot = state.snapshot_tabs();
    let _ = app.emit("tabs-updated", &snapshot);

    Ok(url_string)
}

#[tauri::command]
pub async fn go_back(app: AppHandle) -> Result<(), String> {
    if let Some(webview) = app.get_webview(CONTENT_WEBVIEW_LABEL) {
        webview
            .eval("window.history.back();")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn go_forward(app: AppHandle) -> Result<(), String> {
    if let Some(webview) = app.get_webview(CONTENT_WEBVIEW_LABEL) {
        webview
            .eval("window.history.forward();")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn reload_page(app: AppHandle) -> Result<(), String> {
    if let Some(webview) = app.get_webview(CONTENT_WEBVIEW_LABEL) {
        webview
            .eval("window.location.reload();")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn sync_viewport_bounds(
    app: AppHandle,
    state: State<'_, BrowserState>,
    bounds: ViewportBounds,
) -> Result<(), String> {
    if let Ok(mut stored_bounds) = state.bounds.lock() {
        *stored_bounds = bounds;
    }
    sync_all_webviews(&app, &state)
}
