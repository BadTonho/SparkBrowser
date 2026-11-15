//! Authenticated content messages share validation with the legacy navigation transport.
use super::navigation_state::update_navigation;
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager, Url};

pub(super) fn handle(
    app: &AppHandle,
    tab_id: &str,
    label: &str,
    token: &str,
    nav_url: &Url,
) -> bool {
    if nav_url.scheme() == "spark-internal" && nav_url.host_str() == Some("settings-tab") {
        super::settings_tab::handle(app, tab_id, label, token, nav_url);
        return false;
    }
    if nav_url.scheme() == "spark-internal" && nav_url.host_str() == Some("new-tab") {
        super::new_tab::handle(app, tab_id, label, token, nav_url);
        return false;
    }
    if nav_url.scheme() == "spark-internal" && nav_url.host_str() == Some("page-activity") {
        if nav_url.path() != "/state" {
            return false;
        }
        let Some(values) = crate::security::bridge::query(
            nav_url,
            &["token", "document", "sequence", "dirty", "media"],
        ) else {
            return false;
        };
        let (Some(dirty), Some(media)) = (
            crate::security::bridge::boolean(values.get("dirty")),
            crate::security::bridge::boolean(values.get("media")),
        ) else {
            return false;
        };
        let (Some(document), Some(sequence)) = (
            values.get("document"),
            values.get("sequence").and_then(|v| v.parse::<u64>().ok()),
        ) else {
            return false;
        };
        if values.get("token").map(String::as_str) == Some(token)
            && super::navigation_state::is_current_generation(app, label, token)
            && app
                .state::<BrowserState>()
                .security()
                .keyboard_bridge_sequences()
                .lock()
                .is_ok_and(|mut states| {
                    states.admit(&format!("{label}:activity"), document, sequence)
                })
        {
            update_navigation(app, tab_id, label, token, |tab| {
                tab.activity.dirty_form = dirty;
                tab.activity.document_media = media;
                tab.activity.refresh_media();
            });
        }
        return false;
    }
    if nav_url.scheme() == "spark-internal" && nav_url.host_str() == Some("workspace-switcher") {
        // Workspace selection belongs only to the packaged picker surface.
        // Content Escape uses the authenticated keyboard channel instead.
        return false;
    }

    if nav_url.scheme() == "spark-internal" && nav_url.host_str() == Some("keyboard-shortcut") {
        super::keyboard_bridge::handle_keyboard_navigation(app, nav_url, tab_id, label, token);
        return false;
    }

    if nav_url.scheme() == "spark-internal" && nav_url.host_str() == Some("pane-focus") {
        if !matches!(nav_url.path(), "" | "/") {
            return false;
        }
        if crate::security::bridge::query(nav_url, &["token"]).is_none() {
            return false;
        }
        if nav_url
            .query_pairs()
            .any(|(key, value)| key == "token" && value == token)
        {
            super::split_focus::focus_pane(app, tab_id, label, token);
        }
        return false;
    }
    // No internal URL is a supported keyboard transport on Windows; keep
    // the channel scheme from ever reaching the web content as navigation.
    if nav_url.scheme() == "spark-internal" {
        return false;
    }
    false
}

#[cfg(any(windows, test))]
fn message_url(json: &str, source: &str, current: &str, settings_document: bool) -> Option<Url> {
    if source != current || json.len() > if settings_document { 2_000_000 } else { 262_144 } {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    if value.as_object()?.len() != 1 {
        return None;
    }
    let url = Url::parse(value.get("sparkContent")?.as_str()?).ok()?;
    (url.scheme() == "spark-internal"
        && (matches!(url.host_str(), Some("new-tab" | "page-activity"))
            || (settings_document && url.host_str() == Some("settings-tab")))
        && (url.host_str() == Some("settings-tab") || json.len() <= 262_144))
    .then_some(url)
}

#[cfg(windows)]
pub(crate) fn receive(
    app: &AppHandle,
    label: &str,
    token: &str,
    json: &str,
    origin: &str,
    current_origin: &str,
) {
    if !super::navigation_state::is_current_generation(app, label, token) {
        return;
    }
    use tauri::Manager;
    let settings_document = app.get_webview(label).and_then(|view| view.url().ok()).is_some_and(|url| super::settings_tab::is_document(app, &url));
    if let (Some(id), Some(url)) = (
        label.strip_prefix("content-webview-"),
        message_url(json, origin, current_origin, settings_document),
    ) {
        handle(app, id, label, token, &url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn messages_are_bounded_and_bound_to_the_current_document_and_allowed_channels() {
        let valid = r#"{"sparkContent":"spark-internal://page-activity/state?token=fixture"}"#;
        assert!(message_url(valid, "https://site.test/a", "https://site.test/a", false).is_some());
        let settings = r#"{"sparkContent":"spark-internal://settings-tab/action?token=fixture"}"#;
        assert!(message_url(settings, "http://tauri.localhost", "http://tauri.localhost", true).is_some());
        assert!(message_url(settings, "http://tauri.localhost", "http://tauri.localhost", false).is_none());
        assert!(message_url(settings, "https://evil.test", "http://tauri.localhost", true).is_none());
        assert!(message_url(valid, "https://site.test/old", "https://site.test/current", false).is_none());
        for invalid in [
            "null",
            "[]",
            r#"{"sparkContent":42}"#,
            r#"{"sparkContent":"https://site.test"}"#,
            r#"{"sparkContent":"spark-internal://keyboard-shortcut/action"}"#,
            r#"{"sparkContent":"spark-internal://new-tab/read","other":true}"#,
        ] {
            assert!(message_url(invalid, "same", "same", false).is_none());
        }
        assert!(message_url(&" ".repeat(262_145), "same", "same", false).is_none());
    }
}
