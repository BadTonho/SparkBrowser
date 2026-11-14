use crate::state::browser_state::BrowserState;
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

    state.policies().set_keyboard_shortcuts(shortcuts.clone())?;
    state.policies().set_workspace_picker_hold_key(hold_key)?;

    let keyboard_tokens = state
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    for (label, token) in keyboard_tokens {
        if let Some(webview) = app.get_webview(&label) {
            let payload = serde_json::to_string(&shortcuts).map_err(|error| error.to_string())?;
            webview
                .eval(format!(
                    "window.__sparkSetKeyboardShortcuts?.(\"{token}\", {payload});"
                ))
                .map_err(|error| format!("Failed to update WebView keyboard shortcuts: {error}"))?;
        }
    }
    Ok(())
}
