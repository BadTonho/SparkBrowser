use super::webview_manager::{ensure_active_content_webview, CONTENT_WEBVIEW_PREFIX};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager};

/// Rebuild views when their immutable initialization policy changes.
pub fn reset_content_webviews(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    state
        .keyboard_bridge_tokens
        .lock()
        .map_err(|e| e.to_string())?
        .clear();
    for (label, view) in app.webviews() {
        if label.starts_with(CONTENT_WEBVIEW_PREFIX) {
            view.close().map_err(|e| e.to_string())?;
        }
    }
    let active = state
        .active_tab_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    for tab in state.tabs.lock().map_err(|e| e.to_string())?.iter_mut() {
        tab.is_hibernated = tab.id != active;
        if tab.id == active
            && crate::privacy::adblock_engine::should_block_url(
                &tab.url,
                *state.shield_enabled.lock().map_err(|e| e.to_string())?,
            )
        {
            tab.url = crate::state::browser_state::DEFAULT_HOME_URL.to_string();
        }
    }
    ensure_active_content_webview(app)
}
