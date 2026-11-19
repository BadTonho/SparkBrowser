use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager};

/// `enabled` is the current setting, so a new page can be darkened as soon as it has loaded instead of
/// waiting for the shell to switch the script on.
/// The browser's own pages: they paint their own theme and must never be darkened by the filter.
pub fn is_own_document(app: &AppHandle, url: &tauri::Url) -> bool {
    crate::window::settings_tab::is_document(app, url)
        || crate::window::new_tab::is_document(app, url)
}

/// Whether the filter may be on: the setting, except on the browser's own pages.
pub fn effective(enabled: bool, own_page: bool) -> bool {
    enabled && !own_page
}

pub fn initializer(token: &str, enabled: bool) -> String {
    include_str!("forced_dark.js")
        .replace(
            "__FORCED_DARK_TOKEN__",
            &serde_json::to_string(token).expect("token serializable"),
        )
        .replace(
            "__FORCED_DARK_INITIAL__",
            if enabled { "true" } else { "false" },
        )
}

/// No content-page IPC. The immutable initializer starts disabled on every document, and the shell
/// switches it on with the per-view token, as it does for the auto skip script.
pub fn apply_view(view: &tauri::Webview, token: &str, enabled: bool) -> Result<(), String> {
    // The browser's own pages (settings, new tab) already follow the theme. Inverting them swaps
    // dark and light, so the filter is always off there, even when the setting is on.
    let own_page = view
        .url()
        .is_ok_and(|url| is_own_document(view.app_handle(), &url));
    let enabled = effective(enabled, own_page);
    view.eval(format!(
        "window.__sparkSetForcedDark?.({}, {enabled});",
        serde_json::to_string(token).map_err(|e| e.to_string())?
    ))
    .map_err(|e| e.to_string())
}

pub fn apply(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let tokens = app
        .state::<BrowserState>()
        .security()
        .keyboard_bridge_tokens()
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
    crate::error_log::ignored(apply_view(
        view,
        token,
        state.policies().forced_dark_enabled(),
    ));
}

#[cfg(test)]
mod tests {
    use super::effective;

    #[test]
    fn the_filter_never_applies_to_the_browsers_own_pages() {
        assert!(effective(true, false));
        assert!(!effective(true, true));
        assert!(!effective(false, false));
        assert!(!effective(false, true));
    }
}
