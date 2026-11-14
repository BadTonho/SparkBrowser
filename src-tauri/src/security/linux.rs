//! WebKitGTK counterpart of `security::windows`: the shell may only show shell documents,
//! internal views never open windows of their own, and DevTools are development-only.
use tauri::{plugin::TauriPlugin, AppHandle, Manager, Webview, Wry};

/// The shell (`main`) may only navigate to shell documents. Enforced through Tauri's plugin
/// navigation hook because wry's own `decide-policy` handler runs first and settles the decision.
pub(crate) fn shell_navigation_guard() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::<Wry>::new("spark-shell-navigation")
        .on_navigation(|webview, url| {
            !crate::state::tab_model::is_window_label(webview.label())
                || super::documents::is_shell(webview.app_handle(), url)
        })
        .build()
}

pub(super) fn install(_app: &AppHandle, view: &Webview, internal: bool) -> Result<(), String> {
    view.with_webview(move |platform| {
        use webkit2gtk::{PolicyDecisionExt, PolicyDecisionType, SettingsExt, WebViewExt};
        let webview = platform.inner();
        if let Some(settings) = webview.settings() {
            settings.set_enable_developer_extras(tauri::is_dev());
        }
        if internal {
            webview.connect_decide_policy(move |_, decision, kind| {
                let new_window = kind == PolicyDecisionType::NewWindowAction;
                if new_window {
                    decision.ignore();
                }
                new_window
            });
        }
    })
    .map_err(|error| error.to_string())
}
