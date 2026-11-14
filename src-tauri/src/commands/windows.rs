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

/// The open windows, so a tab can be sent to another one.
#[tauri::command]
pub fn get_browser_windows(state: State<'_, BrowserState>) -> Vec<WindowSummary> {
    browser_windows::summaries(&state)
}
