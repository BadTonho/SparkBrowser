pub(crate) mod audio;
mod bridge;
mod message;
mod recent;
mod suggest;
pub mod widgets;

use crate::state::browser_state::BrowserState;
use crate::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Manager, Url};

pub const URL: &str = "spark://newtab";
/// Address saved by builds from before the rename; tabs, sessions and backups may still carry it.
pub const LEGACY_URL: &str = "veloce://newtab";
pub fn is_new_tab(raw: &str) -> bool {
    raw == URL || raw == LEGACY_URL
}
pub fn valid_tab_url(raw: &str) -> bool {
    is_new_tab(raw)
        || super::settings_tab::is_settings_url(raw)
        || super::wallpaper_tab::is_wallpaper_url(raw)
        || Url::parse(raw)
            .is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
}

#[derive(Clone, serde::Serialize)]
pub struct Preferences {
    pub engine: String,
    pub language: String,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            engine: "google".into(),
            language: "en".into(),
        }
    }
}
#[derive(Default)]
pub struct NewTabService {
    pub(crate) history_epoch: AtomicU64,
    preferences: Mutex<Preferences>,
    revision: AtomicU64,
    recent_cache: Mutex<recent::Cache>,
    sessions: Mutex<std::collections::HashMap<String, Session>>,
}
struct Session {
    document: String,
    sequence: u64,
    last_read: u64,
    navigation: u64,
    presented: bool,
}
impl Session {
    fn accept(&mut self, document: &str, sequence: u64, navigation: bool) -> bool {
        if self.document != document || sequence <= self.sequence {
            return false;
        }
        self.sequence = sequence;
        if navigation {
            self.navigation = sequence;
        }
        true
    }
}

pub fn revoke(app: &AppHandle, label: &str) {
    crate::media::visualizer::unsubscribe(app, label);
    widgets::revoke(app, label);
    if let Ok(mut sessions) = app.state::<NewTabService>().sessions.lock() {
        sessions.remove(label);
    }
}

pub fn document_url(app: &AppHandle) -> Result<Url, String> {
    let base = crate::security::documents::asset_base(app);
    base.join("/new-tab.html").map_err(|e| e.to_string())
}
pub fn is_document(app: &AppHandle, url: &Url) -> bool {
    document_url(app).is_ok_and(|expected| expected == *url)
}
pub fn logical_url(app: &AppHandle, url: &Url) -> Option<String> {
    if is_document(app, url) {
        Some(URL.into())
    } else if super::settings_tab::is_document(app, url) {
        Some(super::settings_tab::URL.into())
    } else if super::wallpaper_tab::is_document(app, url) {
        Some(super::wallpaper_tab::URL.into())
    } else if matches!(url.scheme(), "http" | "https") && url.host_str() != Some("tauri.localhost")
    {
        Some(url.to_string())
    } else {
        None
    }
}
pub fn physical_url(app: &AppHandle, raw: &str) -> Result<Url, String> {
    if is_new_tab(raw) {
        document_url(app)
    } else if super::settings_tab::is_settings_url(raw) {
        super::settings_tab::document_url(app)
    } else if super::wallpaper_tab::is_wallpaper_url(raw) {
        super::wallpaper_tab::document_url(app)
    } else {
        Url::parse(raw).map_err(|e| e.to_string())
    }
}
pub fn title(app: &AppHandle) -> String {
    let pt = app
        .state::<NewTabService>()
        .preferences
        .lock()
        .map(|p| p.language == "pt-BR")
        .unwrap_or(false);
    if pt { "Nova guia" } else { "New tab" }.into()
}
pub fn settings_title(app: &AppHandle) -> String {
    let pt = app
        .state::<NewTabService>()
        .preferences
        .lock()
        .map(|p| p.language == "pt-BR")
        .unwrap_or(false);
    if pt { "Configurações" } else { "Settings" }.into()
}
pub fn wallpaper_title(app: &AppHandle) -> String {
    let pt = app
        .state::<NewTabService>()
        .preferences
        .lock()
        .map(|p| p.language == "pt-BR")
        .unwrap_or(false);
    if pt {
        "Plano de fundo personalizado"
    } else {
        "Custom background"
    }
    .into()
}
pub fn initializer(app: &AppHandle, token: &str, id: &str) -> Result<String, String> {
    Ok(include_str!("bridge.js")
        .replace(
            "__NEW_TAB_URL__",
            &serde_json::to_string(document_url(app)?.as_str()).unwrap(),
        )
        .replace("__NEW_TAB_TOKEN__", &serde_json::to_string(token).unwrap())
        .replace("__NEW_TAB_ID__", &serde_json::to_string(id).unwrap()))
}
pub fn invalidate(app: &AppHandle) {
    let service = app.state::<NewTabService>();
    service.revision.fetch_add(1, Ordering::SeqCst);
    for tab in app.state::<BrowserState>().snapshot_tabs() {
        if !is_new_tab(&tab.url) {
            continue;
        }
        if let Some(view) =
            app.get_webview(&crate::state::tab_model::webview_label_for_tab(&tab.id))
        {
            crate::error_log::ignored(view.eval("window.__sparkRefreshNewTab?.()"));
        }
    }
}
pub fn set_preferences(app: &AppHandle, engine: String, language: String) -> Result<(), String> {
    if !crate::search_engines::is_valid(&engine) || !["en", "pt-BR"].contains(&language.as_str()) {
        return Err("Invalid new tab preferences".into());
    }
    *app.state::<NewTabService>()
        .preferences
        .lock()
        .map_err(|e| e.to_string())? = Preferences { engine, language };
    invalidate(app);
    Ok(())
}
pub use bridge::handle;

#[cfg(test)]
mod tests {
    #[test]
    fn refresh_cannot_cancel_an_accepted_navigation() {
        let mut session = super::Session {
            document: "current".into(),
            sequence: 1,
            last_read: 1,
            navigation: 0,
            presented: true,
        };
        assert!(session.accept("current", 2, true));
        assert!(session.accept("current", 3, false));
        assert_eq!(session.navigation, 2);
        assert_eq!(
            session.last_read, 1,
            "widget reads cannot cancel pending history reads"
        );
        assert!(!session.accept("old", 4, true));
        assert!(!session.accept("current", 2, true));
        assert!(session.accept("current", 4, true));
        assert_eq!(session.navigation, 4);
    }
    #[test]
    fn only_the_exact_internal_address_is_supported() {
        assert!(super::valid_tab_url(super::URL));
        assert!(super::valid_tab_url(super::super::settings_tab::URL));
        assert!(super::valid_tab_url(super::super::wallpaper_tab::URL));
        for raw in [
            "spark://newtab/",
            "spark://newtab?x=1",
            "spark://settings?x=1",
            "file:///tmp/x",
            "javascript:alert(1)",
        ] {
            assert!(!super::valid_tab_url(raw));
        }
        assert!(crate::backup::validation::tab_url(super::URL).is_ok());
        assert!(crate::backup::validation::page_url(super::URL).is_err());
        assert_eq!(
            crate::backup::validation::tab_url(super::super::settings_tab::URL).unwrap(),
            super::super::settings_tab::URL
        );
        assert!(crate::backup::validation::page_url(super::super::settings_tab::URL).is_err());
        assert_eq!(
            super::super::url_input::resolve_input_to_url_with_search_engine(
                super::super::settings_tab::URL,
                "google"
            )
            .unwrap()
            .as_str(),
            super::super::settings_tab::URL
        );
    }
}
