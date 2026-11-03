mod bridge;
mod recent;

use crate::state::browser_state::BrowserState;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use tauri::{AppHandle, Manager, Url};

pub const URL: &str = "veloce://newtab";
pub fn is_new_tab(raw: &str) -> bool {
    raw == URL
}
pub fn valid_tab_url(raw: &str) -> bool {
    is_new_tab(raw)
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
    preferences: Mutex<Preferences>,
    revision: AtomicU64,
    sessions: Mutex<std::collections::HashMap<String, Session>>,
}
struct Session {
    document: String,
    sequence: u64,
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
    if let Ok(mut sessions) = app.state::<NewTabService>().sessions.lock() {
        sessions.remove(label);
    }
}

pub fn document_url(app: &AppHandle) -> Result<Url, String> {
    // During setup the shell is still about:blank. Use the same configured base
    // as WebviewUrl::App rather than observing a document that has not loaded.
    let configured = if tauri::is_dev() {
        app.config().build.dev_url.clone()
    } else {
        match &app.config().build.frontend_dist {
            Some(tauri::utils::config::FrontendDist::Url(url)) => Some(url.clone()),
            _ => None,
        }
    };
    let base = configured.unwrap_or_else(|| {
        Url::parse(if cfg!(windows) {
            "http://tauri.localhost/"
        } else {
            "tauri://localhost/"
        })
        .expect("application asset origin")
    });
    base.join("/new-tab.html").map_err(|e| e.to_string())
}
pub fn is_document(app: &AppHandle, url: &Url) -> bool {
    document_url(app).is_ok_and(|expected| expected == *url)
}
pub fn logical_url(app: &AppHandle, url: &Url) -> Option<String> {
    if is_document(app, url) {
        Some(URL.into())
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
            let _ = view.eval("window.__veloceRefreshNewTab?.()");
        }
    }
}
pub fn set_preferences(app: &AppHandle, engine: String, language: String) -> Result<(), String> {
    if !["google", "duckduckgo"].contains(&engine.as_str())
        || !["en", "pt-BR"].contains(&language.as_str())
    {
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
            navigation: 0,
            presented: true,
        };
        assert!(session.accept("current", 2, true));
        assert!(session.accept("current", 3, false));
        assert_eq!(session.navigation, 2);
        assert!(!session.accept("old", 4, true));
        assert!(!session.accept("current", 2, true));
        assert!(session.accept("current", 4, true));
        assert_eq!(session.navigation, 4);
    }
    #[test]
    fn only_the_exact_internal_address_is_supported() {
        assert!(super::valid_tab_url(super::URL));
        for raw in [
            "veloce://newtab/",
            "veloce://newtab?x=1",
            "veloce://settings",
            "file:///tmp/x",
            "javascript:alert(1)",
        ] {
            assert!(!super::valid_tab_url(raw));
        }
        assert!(crate::backup::validation::tab_url(super::URL).is_ok());
        assert!(crate::backup::validation::page_url(super::URL).is_err());
    }
}
