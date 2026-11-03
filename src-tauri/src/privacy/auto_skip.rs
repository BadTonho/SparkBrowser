use crate::state::browser_state::BrowserState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};

pub fn initializer(token: &str) -> String {
    include_str!("auto_skip.js").replace(
        "__AUTO_SKIP_TOKEN__",
        &serde_json::to_string(token).expect("token serializable"),
    )
}

/// No content-page IPC. The immutable initializer starts disabled on every document.
pub fn apply_view(view: &tauri::Webview, token: &str, enabled: bool) -> Result<(), String> {
    view.eval(format!(
        "window.__veloceSetAutoSkip?.({}, {enabled});",
        serde_json::to_string(token).map_err(|e| e.to_string())?
    ))
    .map_err(|e| e.to_string())
}

pub fn apply(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let tokens = app
        .state::<BrowserState>()
        .keyboard_bridge_tokens
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
    let _ = apply_view(
        view,
        token,
        state.auto_skip_ads_enabled.load(Ordering::Acquire),
    );
}
