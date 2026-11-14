//! Isolated WebKitGTK minimum-font probe. This command is unavailable to the production app.
use tauri::AppHandle;

#[tauri::command]
pub async fn set_minimum_font_probe_size(app: AppHandle, size: u32) -> Result<u32, String> {
    if app.config().identifier != "org.spark.browser.native-tests" {
        return Err("minimum_font_probe.test_application_only".into());
    }
    if size > 72 {
        return Err("minimum_font_probe.invalid_size".into());
    }
    let view = crate::window::webview_manager::active_content_webview(&app)
        .ok_or("minimum_font_probe.active_webview_missing")?;

    #[cfg(target_os = "linux")]
    {
        let (send, receive) = std::sync::mpsc::channel();
        view.with_webview(move |platform| {
            use webkit2gtk::prelude::{SettingsExt, WebViewExt};
            let settings = platform.inner().settings();
            let previous = settings.minimum_font_size();
            settings.set_minimum_font_size(size);
            let _ = send.send(previous);
        })
        .map_err(|error| error.to_string())?;
        return receive
            .recv_timeout(std::time::Duration::from_secs(10))
            .map_err(|error| error.to_string());
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = view;
        Err("minimum_font_probe.native_setting_unavailable".into())
    }
}
