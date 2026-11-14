use crate::state::browser_state::{BrowserState, ViewportBounds};
use crate::window::viewport::resize_visible_webviews;
use crate::window::webview_manager::{
    active_content_webview_in, go_back_content_webview, go_forward_content_webview,
    navigate_window_with_search_engine,
};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn focus_address_bar(app: AppHandle, webview: tauri::Webview) -> Result<(), String> {
    app.get_webview(&super::access::caller_window(&webview))
        .ok_or("Browser shell unavailable")?
        .set_focus()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn navigate_to(
    app: AppHandle,
    webview: tauri::Webview,
    input: String,
    search_engine: Option<String>,
) -> Result<String, String> {
    let state = app.state::<BrowserState>();
    let _operation = state.session().operation().await;
    navigate_window_with_search_engine(
        &app,
        &super::access::caller_window(&webview),
        &input,
        search_engine.as_deref().unwrap_or("google"),
    )
}

#[tauri::command]
pub async fn go_back(app: AppHandle, webview: tauri::Webview) -> Result<(), String> {
    go_back_content_webview(&app, &super::access::caller_window(&webview))
}

#[tauri::command]
pub async fn go_forward(app: AppHandle, webview: tauri::Webview) -> Result<(), String> {
    go_forward_content_webview(&app, &super::access::caller_window(&webview))
}

#[tauri::command]
pub async fn reload_page(app: AppHandle, webview: tauri::Webview) -> Result<(), String> {
    active_content_webview_in(&app, &super::access::caller_window(&webview))
        .ok_or("Active WebView was not found")?
        .eval("window.location.reload();")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_viewport_bounds(
    app: AppHandle,
    state: State<'_, BrowserState>,
    bounds: ViewportBounds,
    split_ratio: Option<f64>,
    split_key: Option<String>,
) -> Result<(), String> {
    if ![bounds.x, bounds.y, bounds.width, bounds.height]
        .iter()
        .all(|v| v.is_finite())
        || bounds.width < 100.0
        || bounds.height < 100.0
    {
        return Err("Invalid viewport bounds".into());
    }
    let ratio_changed = match (split_ratio, split_key) {
        (Some(ratio), Some(key)) => state.preview_split_ratio(&key, ratio).is_ok(),
        _ => false,
    };
    {
        let mut stored_bounds = state
            .surfaces()
            .bounds()
            .lock()
            .map_err(|e| e.to_string())?;
        if *stored_bounds == bounds && !ratio_changed {
            return Ok(());
        }
        *stored_bounds = bounds;
    }
    let mut span = state.infrastructure().diagnostics().span("viewport.resize");
    let result = resize_visible_webviews(&app, &state);
    span.finish(result.is_ok(), 0);
    result
}
