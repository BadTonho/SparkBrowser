//! Authenticated content messages share validation with the legacy navigation transport.
use super::navigation_state::update_navigation;
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager, Url};

/// Edits only count as unsaved work on web pages. The browser's own pages (new tab, settings) apply every
/// control as soon as it changes, so flagging them made the exit review appear for no reason.
fn tracks_form_edits(url: &str) -> bool {
    !super::new_tab::is_new_tab(url)
        && !super::settings_tab::is_settings_url(url)
        && !super::wallpaper_tab::is_wallpaper_url(url)
        && !super::history_overlay::tab::is_history_url(url)
}

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
    if nav_url.scheme() == "spark-internal" && nav_url.host_str() == Some("history-tab") {
        super::history_overlay::tab::handle(app, tab_id, label, token, nav_url);
        return false;
    }
    if nav_url.scheme() == "spark-internal" && nav_url.host_str() == Some("new-tab") {
        super::new_tab::handle(app, tab_id, label, token, nav_url);
        return false;
    }
    if nav_url.scheme() == "spark-internal" && nav_url.host_str() == Some("autofill") {
        crate::autofill::handle(app, label, token, nav_url);
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
                // The new tab and settings pages apply their controls immediately; nothing there is unsaved.
                tab.activity.dirty_form = dirty && tracks_form_edits(&tab.url);
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
        let expected = crate::security::bridge::channel_token(
            token,
            crate::security::bridge::KEYBOARD_CHANNEL,
        );
        if nav_url
            .query_pairs()
            .any(|(key, value)| key == "token" && value == expected.as_str())
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

#[cfg(any(windows, target_os = "linux", test))]
/// The browser's own page a content message comes from: its channel is only accepted from it.
#[cfg(any(windows, target_os = "linux", test))]
#[derive(Clone, Copy, PartialEq, Eq)]
enum OwnDocument {
    Web,
    Settings,
    History,
}

#[cfg(any(windows, target_os = "linux", test))]
fn message_url(json: &str, source: &str, current: &str, document: OwnDocument) -> Option<Url> {
    let settings_document = document == OwnDocument::Settings;
    if source != current
        || json.len()
            > if settings_document {
                2_000_000
            } else {
                262_144
            }
    {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    if value.as_object()?.len() != 1 {
        return None;
    }
    let url = Url::parse(value.get("sparkContent")?.as_str()?).ok()?;
    (url.scheme() == "spark-internal"
        && (matches!(
            url.host_str(),
            Some("new-tab" | "page-activity" | "autofill" | "keyboard-shortcut" | "pane-focus")
        ) || (settings_document && url.host_str() == Some("settings-tab"))
            || (document == OwnDocument::History && url.host_str() == Some("history-tab")))
        && (url.host_str() == Some("settings-tab") || json.len() <= 262_144))
        .then_some(url)
}

#[cfg(any(windows, target_os = "linux"))]
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
    let page = app.get_webview(label).and_then(|view| view.url().ok());
    let document = match page {
        Some(url) if super::settings_tab::is_document(app, &url) => OwnDocument::Settings,
        Some(url) if super::history_overlay::tab::is_document(app, &url) => OwnDocument::History,
        _ => OwnDocument::Web,
    };
    if let (Some(id), Some(url)) = (
        label.strip_prefix("content-webview-"),
        message_url(json, origin, current_origin, document),
    ) {
        handle(app, id, label, token, &url);
    }
}

/// WebKitGTK has no runtime bindings: content messages arrive through a script message handler,
/// so they never travel as a navigation the page could observe (Navigation API). Script message
/// handlers carry no frame, so the source is the WebView's own document, like the legacy transport.
#[cfg(target_os = "linux")]
const LINUX_HANDLER: &str = "sparkContent";

#[cfg(target_os = "linux")]
pub(super) fn install(app: &AppHandle, view: &tauri::Webview, token: &str) -> Result<(), String> {
    let (app, label, token) = (app.clone(), view.label().to_owned(), token.to_owned());
    view.with_webview(move |platform| {
        use javascriptcore::ValueExt as _;
        use webkit2gtk::{UserContentManagerExt, WebViewExt};
        let webview = platform.inner();
        let Some(manager) = webview.user_content_manager() else {
            crate::error_log::report(
                crate::error_log::Level::Error,
                "native",
                "Content messages are unavailable: the WebView has no user content manager",
                &label,
            );
            return;
        };
        manager.register_script_message_handler(LINUX_HANDLER);
        manager.connect_script_message_received(Some(LINUX_HANDLER), move |_, result| {
            let Some(raw) = result.js_value().filter(|value| value.is_string()) else {
                return;
            };
            let origin = app
                .get_webview(&label)
                .and_then(|view| view.url().ok())
                .map(|url| url.origin().ascii_serialization())
                .unwrap_or_default();
            receive(
                &app,
                &label,
                &token,
                raw.to_str().as_str(),
                &origin,
                &origin,
            );
        });
    })
    .map_err(|error| format!("Failed to install the content message handler: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn messages_are_bounded_and_bound_to_the_current_document_and_allowed_channels() {
        let valid = r#"{"sparkContent":"spark-internal://page-activity/state?token=fixture"}"#;
        assert!(message_url(
            valid,
            "https://site.test/a",
            "https://site.test/a",
            OwnDocument::Web
        )
        .is_some());
        let settings = r#"{"sparkContent":"spark-internal://settings-tab/action?token=fixture"}"#;
        assert!(message_url(
            settings,
            "http://tauri.localhost",
            "http://tauri.localhost",
            OwnDocument::Settings
        )
        .is_some());
        assert!(message_url(
            settings,
            "http://tauri.localhost",
            "http://tauri.localhost",
            OwnDocument::Web
        )
        .is_none());
        assert!(message_url(
            settings,
            "https://evil.test",
            "http://tauri.localhost",
            OwnDocument::Settings
        )
        .is_none());
        assert!(message_url(
            valid,
            "https://site.test/old",
            "https://site.test/current",
            OwnDocument::Web
        )
        .is_none());
        for invalid in [
            "null",
            "[]",
            r#"{"sparkContent":42}"#,
            r#"{"sparkContent":"https://site.test"}"#,
            r#"{"sparkContent":"spark-internal://workspace-switcher/select"}"#,
            r#"{"sparkContent":"spark-internal://settings-popover/action"}"#,
            r#"{"sparkContent":"spark-internal://new-tab/read","other":true}"#,
        ] {
            assert!(message_url(invalid, "same", "same", OwnDocument::Web).is_none());
        }
        assert!(message_url(&" ".repeat(262_145), "same", "same", OwnDocument::Web).is_none());
        // The history tab's channel is only accepted from the history page.
        let history = r#"{"sparkContent":"spark-internal://history-tab/action?token=fixture"}"#;
        assert!(message_url(history, "same", "same", OwnDocument::History).is_some());
        assert!(message_url(history, "same", "same", OwnDocument::Web).is_none());
        assert!(message_url(history, "same", "same", OwnDocument::Settings).is_none());
        // Keyboard shortcuts and pane focus use the same authenticated transport (no navigation).
        for channel in ["keyboard-shortcut/action", "pane-focus"] {
            let message = format!(r#"{{"sparkContent":"spark-internal://{channel}?token=x"}}"#);
            assert!(message_url(&message, "same", "same", OwnDocument::Web).is_some());
        }
    }

    #[test]
    fn only_web_pages_track_form_edits() {
        assert!(tracks_form_edits("https://example.com/form"));
        assert!(!tracks_form_edits("spark://newtab"));
        assert!(!tracks_form_edits("spark://settings"));
    }
}
