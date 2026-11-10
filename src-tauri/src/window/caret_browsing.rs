use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager};

pub(super) fn toggle(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<BrowserState>();
    let previous = state.policies().caret_browsing_enabled();
    let enabled = !previous;
    let labels = app
        .webviews()
        .into_iter()
        .filter_map(|(label, _)| {
            label
                .starts_with(crate::window::webview_manager::CONTENT_WEBVIEW_PREFIX)
                .then_some(label)
        })
        .collect::<Vec<_>>();
    let mut applied = Vec::new();

    for label in labels {
        if let Err(error) = apply_to_webview(app, &label, enabled) {
            for applied_label in applied {
                let _ = apply_to_webview(app, &applied_label, previous);
            }
            return Err(error);
        }
        applied.push(label);
    }

    state.policies().set_caret_browsing_enabled(enabled);
    Ok(())
}

pub(super) fn apply_to_webview(app: &AppHandle, label: &str, enabled: bool) -> Result<(), String> {
    let Some(webview) = app.get_webview(label) else {
        return Ok(());
    };

    webview
        .with_webview(move |platform| {
            use webkit2gtk::prelude::{SettingsExt, WebViewExt};
            platform
                .inner()
                .settings()
                .set_enable_caret_browsing(enabled);
        })
        .map_err(|error| error.to_string())
}
