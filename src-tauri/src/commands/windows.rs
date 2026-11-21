use crate::state::browser_state::BrowserState;
use crate::window::browser_windows::{self, WindowSummary};
use tauri::{AppHandle, State};

/// Opens a new browser window with a new-tab page; returns its id.
#[tauri::command]
pub async fn open_browser_window(
    app: AppHandle,
    webview: tauri::Webview,
) -> Result<String, String> {
    browser_windows::open(&app, &super::access::caller_window(&webview)).await
}

/// Sends a tab to another window; the page reloads there.
#[tauri::command]
pub async fn move_tab_to_window(
    app: AppHandle,
    tab_id: String,
    window_id: String,
) -> Result<(), String> {
    browser_windows::move_tab(&app, &tab_id, &window_id).await
}

/// Moves a tab into a new window placed at the screen position where it was dropped.
#[tauri::command]
pub async fn detach_tab_to_new_window(
    app: AppHandle,
    tab_id: String,
    x: i32,
    y: i32,
) -> Result<String, String> {
    browser_windows::detach_tab(&app, &tab_id, x, y).await
}

/// The open windows, so a tab can be sent to another one.
#[tauri::command]
pub fn get_browser_windows(state: State<'_, BrowserState>) -> Vec<WindowSummary> {
    browser_windows::summaries(&state)
}
