use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager};

/// `enabled` is the current setting, so a new page can be darkened as soon as it has loaded instead of
/// waiting for the shell to switch the script on.
pub fn initializer(token: &str, enabled: bool) -> String {
    include_str!("forced_dark.js")
        .replace(
            "__FORCED_DARK_TOKEN__",
            &serde_json::to_string(token).expect("token serializable"),
        )
        .replace(
            "__FORCED_DARK_INITIAL__",
            if enabled { "true" } else { "false" },
        )
}

/// No content-page IPC. The immutable initializer starts disabled on every document, and the shell
/// switches it on with the per-view token, as it does for the auto skip script.
pub fn apply_view(view: &tauri::Webview, token: &str, enabled: bool) -> Result<(), String> {
    view.eval(format!(
        "window.__sparkSetForcedDark?.({}, {enabled});",
        serde_json::to_string(token).map_err(|e| e.to_string())?
    ))
    .map_err(|e| e.to_string())
}

pub fn apply(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let tokens = app
        .state::<BrowserState>()
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    for (label, token) in tokens {
        if let Some(view) = app.get_webview(&label) {
            apply_view(&view, &token, enabled)?;
        }
    }
    Ok(())
}

pub fn loaded(view: &tauri::Webview, state: &BrowserState, token: &str) {
    let _ = apply_view(view, token, state.policies().forced_dark_enabled());
}
