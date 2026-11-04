pub mod bridge;
pub mod documents;
pub mod sequence;
#[cfg(windows)]
mod windows;

/// Internal events have an explicit recipient, independently of listener permissions.
pub fn emit_shell<S: serde::Serialize + Clone>(
    app: &tauri::AppHandle,
    event: &str,
    payload: S,
) -> tauri::Result<()> {
    use tauri::Emitter;
    app.emit_to(
        tauri::EventTarget::Webview {
            label: "main".into(),
        },
        event,
        payload,
    )
}

pub fn harden(app: &tauri::AppHandle, view: &tauri::Webview, internal: bool) -> Result<(), String> {
    #[cfg(windows)]
    windows::install(app, view, internal)?;
    #[cfg(not(windows))]
    let _ = (app, view, internal);
    Ok(())
}
