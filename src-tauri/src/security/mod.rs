pub mod bridge;
pub mod documents;
#[cfg(target_os = "linux")]
mod linux;
pub mod sequence;
#[cfg(windows)]
mod windows;

/// Internal events have an explicit recipient, independently of listener permissions. Events that
/// are not about one window go to the shell of every open window.
pub fn emit_shell<S: serde::Serialize + Clone>(
    app: &tauri::AppHandle,
    event: &str,
    payload: S,
) -> tauri::Result<()> {
    if event == "browser-error" {
        // Every error the browser shows is also logged and offered for reporting.
        if let Ok(serde_json::Value::String(message)) = serde_json::to_value(&payload) {
            if message != "Navigation blocked" {
                crate::error_log::report("native", &message, "");
            }
        }
    }
    emit_shell_all(app, event, payload);
    Ok(())
}

/// Sends an event to the shell of the window that shows a tab.
pub fn emit_shell_for_tab<S: serde::Serialize + Clone>(
    app: &tauri::AppHandle,
    tab_id: &str,
    event: &str,
    payload: S,
) -> tauri::Result<()> {
    use tauri::Manager;
    let window = app
        .try_state::<crate::state::browser_state::BrowserState>()
        .and_then(|state| state.window_of_tab(tab_id).ok())
        .unwrap_or_else(|| crate::state::tab_model::MAIN_WINDOW_ID.to_string());
    emit_shell_to(app, &window, event, payload)
}

/// Sends an event to the shell of the window that contains a web view (a page or an overlay).
pub fn emit_shell_for_webview<S: serde::Serialize + Clone>(
    app: &tauri::AppHandle,
    label: &str,
    event: &str,
    payload: S,
) -> tauri::Result<()> {
    use tauri::Manager;
    let window = app
        .get_webview(label)
        .map(|view| view.window().label().to_string())
        .filter(|id| crate::state::tab_model::is_window_label(id))
        .unwrap_or_else(|| crate::state::tab_model::MAIN_WINDOW_ID.to_string());
    emit_shell_to(app, &window, event, payload)
}

/// Sends an event to the shell of one browser window (its label is the window id).
pub fn emit_shell_to<S: serde::Serialize + Clone>(
    app: &tauri::AppHandle,
    window_id: &str,
    event: &str,
    payload: S,
) -> tauri::Result<()> {
    use tauri::Emitter;
    app.emit_to(
        tauri::EventTarget::Webview {
            label: window_id.into(),
        },
        event,
        payload,
    )
}

/// Sends an event to the shell of every open window.
pub fn emit_shell_all<S: serde::Serialize + Clone>(
    app: &tauri::AppHandle,
    event: &str,
    payload: S,
) {
    use tauri::Manager;
    let ids = app
        .try_state::<crate::state::browser_state::BrowserState>()
        .map(|state| state.session().window_ids())
        .unwrap_or_else(|| vec![crate::state::tab_model::MAIN_WINDOW_ID.to_string()]);
    for id in ids {
        let _ = emit_shell_to(app, &id, event, payload.clone());
    }
}

#[cfg(target_os = "linux")]
pub fn linux_shell_navigation_guard() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    linux::shell_navigation_guard()
}

pub fn harden(app: &tauri::AppHandle, view: &tauri::Webview, internal: bool) -> Result<(), String> {
    #[cfg(windows)]
    windows::install(app, view, internal)?;
    #[cfg(target_os = "linux")]
    linux::install(app, view, internal)?;
    #[cfg(not(any(windows, target_os = "linux")))]
    let _ = (app, view, internal);
    Ok(())
}

#[cfg(windows)]
pub async fn set_smart_screen(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    use tauri::Manager;
    use windows_core::Interface;
    let state = app.state::<crate::state::browser_state::BrowserState>();
    let tabs = state
        .session()
        .tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let mut changed = 0usize;
    for tab in tabs {
        let label = crate::state::tab_model::webview_label_for_tab(&tab.id);
        let Some(view) = app.get_webview(&label) else {
            continue;
        };
        let (send, receive) = tokio::sync::oneshot::channel();
        view.with_webview(move |platform| {
            use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings8;
            let result = unsafe {
                platform
                    .controller()
                    .CoreWebView2()
                    .and_then(|core| core.Settings())
                    .and_then(|settings| settings.cast::<ICoreWebView2Settings8>())
                    .and_then(|settings| settings.SetIsReputationCheckingRequired(enabled))
            }
            .map_err(|error| error.to_string());
            let _ = send.send(result);
        })
        .map_err(|error| error.to_string())?;
        receive.await.map_err(|error| error.to_string())??;
        changed += 1;
    }
    if changed == 0 {
        return Err("SmartScreen is unavailable in the current WebView2 runtime".into());
    }
    state.policies().set_smart_screen_supported(true);
    Ok(())
}

#[cfg(not(windows))]
pub async fn set_smart_screen(_app: &tauri::AppHandle, _enabled: bool) -> Result<(), String> {
    Err("SmartScreen is unavailable on this platform".into())
}
