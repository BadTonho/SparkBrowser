use crate::state::browser_state::BrowserState;
use crate::window::webview_manager::CONTENT_WEBVIEW_LABEL;
use serde_json::Value;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn set_keyboard_shortcuts(
    app: AppHandle,
    state: State<'_, BrowserState>,
    shortcuts: Value,
) -> Result<(), String> {
    let object = shortcuts
        .as_object()
        .ok_or_else(|| "Keyboard shortcuts must be an object".to_string())?;
    for (id, bindings) in object {
        if !id.chars().all(|ch| ch.is_ascii_lowercase() || ch == '-') || !bindings.is_array() {
            return Err("Invalid keyboard shortcut configuration".to_string());
        }
    }

    let hold_key = shortcuts
        .get("workspace-picker")
        .and_then(Value::as_array)
        .and_then(|bindings| bindings.first())
        .and_then(|binding| binding.get("key"))
        .and_then(Value::as_str)
        .filter(|key| matches!(*key, "Alt" | "Control" | "Shift" | "Meta"))
        .unwrap_or("Alt")
        .to_string();

    if let Ok(mut current) = state.keyboard_shortcuts.lock() {
        *current = shortcuts.clone();
    }
    if let Ok(mut current) = state.workspace_picker_hold_key.lock() {
        *current = hold_key;
    }

    let keyboard_token = state
        .keyboard_bridge_token
        .lock()
        .map(|token| token.clone())
        .unwrap_or(None);
    if let (Some(webview), Some(token)) = (app.get_webview(CONTENT_WEBVIEW_LABEL), keyboard_token) {
        let payload = serde_json::to_string(&shortcuts).map_err(|error| error.to_string())?;
        webview
            .eval(&format!(
                "window.__veloceSetKeyboardShortcuts?.(\"{token}\", {payload});"
            ))
            .map_err(|error| format!("Failed to update WebView keyboard shortcuts: {error}"))?;
    }
    Ok(())
}
