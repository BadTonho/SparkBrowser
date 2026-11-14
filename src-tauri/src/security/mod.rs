pub mod bridge;
pub mod documents;
#[cfg(target_os = "linux")]
mod linux;
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
