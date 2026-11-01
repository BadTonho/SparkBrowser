use crate::state::browser_state::{BrowserState, ViewportBounds};
use crate::window::webview_manager::{
    active_content_webview, go_back_content_webview, go_forward_content_webview,
    navigate_content_webview_with_search_engine, sync_all_webviews,
};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn navigate_to(
    app: AppHandle,
    input: String,
    search_engine: Option<String>,
) -> Result<String, String> {
    navigate_content_webview_with_search_engine(
        &app,
        &input,
        search_engine.as_deref().unwrap_or("google"),
    )
}

#[tauri::command]
pub async fn go_back(app: AppHandle) -> Result<(), String> {
    go_back_content_webview(&app)
}

#[tauri::command]
pub async fn go_forward(app: AppHandle) -> Result<(), String> {
    go_forward_content_webview(&app)
}

#[tauri::command]
pub async fn reload_page(app: AppHandle) -> Result<(), String> {
    active_content_webview(&app)
        .ok_or("Active WebView was not found")?
        .eval("window.location.reload();")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_viewport_bounds(
    app: AppHandle,
    state: State<'_, BrowserState>,
    bounds: ViewportBounds,
) -> Result<(), String> {
    let _operation = state.tab_operations.lock().await;
    if let Ok(mut stored_bounds) = state.bounds.lock() {
        *stored_bounds = bounds;
    }
    sync_all_webviews(&app, &state)
}
