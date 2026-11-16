//! Isolated native color-scheme probe. This command is unavailable to the production app.
use std::sync::mpsc;
use tauri::AppHandle;

#[tauri::command]
pub async fn set_dark_mode_probe_scheme(app: AppHandle, mode: String) -> Result<String, String> {
    if app.config().identifier != "org.spark.browser.native-tests" {
        return Err("dark_mode_probe.test_application_only".into());
    }
    if !matches!(mode.as_str(), "auto" | "light" | "dark") {
        return Err("dark_mode_probe.invalid_scheme".into());
    }
    let view = crate::window::webview_manager::active_content_webview(&app)
        .ok_or("dark_mode_probe.active_webview_missing")?;
    let (send, receive) = mpsc::channel();

    #[cfg(windows)]
    view.with_webview(move |platform| {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            ICoreWebView2_13, COREWEBVIEW2_PREFERRED_COLOR_SCHEME_AUTO,
            COREWEBVIEW2_PREFERRED_COLOR_SCHEME_DARK, COREWEBVIEW2_PREFERRED_COLOR_SCHEME_LIGHT,
        };
        use windows_core::Interface;

        let result = (|| {
            let core = unsafe { platform.controller().CoreWebView2() }
                .map_err(|error| error.to_string())?;
            let profile = core
                .cast::<ICoreWebView2_13>()
                .and_then(|core| unsafe { core.Profile() })
                .map_err(|error| error.to_string())?;
            let mut previous = COREWEBVIEW2_PREFERRED_COLOR_SCHEME_AUTO;
            unsafe { profile.PreferredColorScheme(&mut previous) }
                .map_err(|error| error.to_string())?;
            let selected = match mode.as_str() {
                "auto" => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_AUTO,
                "light" => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_LIGHT,
                "dark" => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_DARK,
                _ => unreachable!(),
            };
            unsafe { profile.SetPreferredColorScheme(selected) }
                .map_err(|error| error.to_string())?;
            Ok(match previous.0 {
                1 => "light",
                2 => "dark",
                _ => "auto",
            }
            .to_string())
        })();
        crate::error_log::ignored(send.send(result));
    })
    .map_err(|error| error.to_string())?;

    #[cfg(target_os = "linux")]
    view.with_webview(move |_| {
        use gtk::prelude::GtkSettingsExt;
        let result = (|| {
            let settings =
                gtk::Settings::default().ok_or("dark_mode_probe.gtk_settings_missing")?;
            let previous = if settings.is_gtk_application_prefer_dark_theme() {
                "dark"
            } else {
                "light"
            };
            match mode.as_str() {
                "dark" => settings.set_gtk_application_prefer_dark_theme(true),
                "light" => settings.set_gtk_application_prefer_dark_theme(false),
                "auto" => return Err("dark_mode_probe.linux_auto_not_supported".into()),
                _ => unreachable!(),
            }
            Ok(previous.to_string())
        })();
        let _ = send.send(result);
    })
    .map_err(|error| error.to_string())?;

    #[cfg(not(any(windows, target_os = "linux")))]
    return Err("dark_mode_probe.unsupported_platform".into());

    receive
        .recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|error| error.to_string())?
}
