//! Session validation for the settings tab's presentation-only bridge.
//!
//! The owning shell remains the only caller of privileged commands. Content
//! messages are accepted only for the document and session that received the
//! presentation and cannot choose arbitrary commands.
use super::panel_popover::bridge::UiAction;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Url};

pub const URL: &str = "spark://settings";

pub fn is_settings_url(raw: &str) -> bool {
    raw == URL
}

pub fn document_url(app: &AppHandle) -> Result<Url, String> {
    let mut url = crate::security::documents::asset_base(app)
        .join("/index.html")
        .map_err(|e| e.to_string())?;
    url.set_query(Some("surface=settings-tab"));
    Ok(url)
}

pub fn is_document(app: &AppHandle, url: &Url) -> bool {
    document_url(app)
        .is_ok_and(|expected| crate::security::documents::same_document(url, &expected))
}

struct Session {
    token: String,
    document: String,
    generation: String,
    sequence: u64,
    session_id: u64,
    revision: u64,
}

impl Session {
    fn advance(
        &mut self,
        generation: &str,
        session_id: u64,
        revision: u64,
    ) -> Result<bool, &'static str> {
        if self.generation != generation || self.session_id > session_id {
            return Err("settings_tab.stale_session");
        }
        if self.session_id < session_id {
            self.token = super::webview_manager::create_internal_bridge_token();
            self.document = super::webview_manager::create_internal_bridge_token();
            self.sequence = 0;
            self.revision = 0;
        }
        if self.revision > revision {
            return Ok(false);
        }
        self.session_id = session_id;
        self.revision = revision;
        Ok(true)
    }
}

#[derive(Default)]
pub struct SettingsTabSessions(Mutex<HashMap<String, Session>>);

impl SettingsTabSessions {
    #[cfg(test)]
    pub fn register(
        &self,
        tab: &str,
        token: String,
        document: String,
        generation: String,
    ) -> Result<(), String> {
        if [tab, token.as_str(), document.as_str(), generation.as_str()]
            .iter()
            .any(|value| value.is_empty() || value.len() > 128)
        {
            return Err("settings_tab.invalid_session".into());
        }
        self.0.lock().map_err(|e| e.to_string())?.insert(
            tab.to_owned(),
            Session {
                token,
                document,
                generation,
                sequence: 0,
                session_id: 0,
                revision: 0,
            },
        );
        Ok(())
    }

    pub fn revoke(&self, tab: &str) {
        if let Ok(mut sessions) = self.0.lock() {
            sessions.remove(tab);
        }
    }

    pub fn accept(
        &self,
        tab: &str,
        token: &str,
        document: &str,
        generation: &str,
        sequence: u64,
        payload: &str,
    ) -> Option<(UiAction, u64)> {
        let action: UiAction = serde_json::from_str(payload).ok()?;
        if !action.valid() {
            return None;
        }
        let mut sessions = self.0.lock().ok()?;
        let session = sessions.get_mut(tab)?;
        if sequence <= session.sequence
            || session.token != token
            || session.document != document
            || session.generation != generation
        {
            return None;
        }
        session.sequence = sequence;
        Some((action, session.session_id))
    }
}

pub fn present(
    app: &AppHandle,
    window_id: &str,
    tab_id: &str,
    session_id: u64,
    presentation: super::settings_overlay::Presentation,
) -> Result<(), String> {
    let state = app.state::<crate::state::browser_state::BrowserState>();
    let tab = state
        .session()
        .tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|tab| tab.id == tab_id)
        .cloned()
        .ok_or("settings_tab.invalid_tab")?;
    if tab.window_id != window_id || !is_settings_url(&tab.url) {
        return Err("settings_tab.invalid_tab".into());
    }
    if tab.is_hibernated || tab.navigation.loading || (!tab.is_active && !tab.is_visible) {
        return Err("settings_tab.not_ready".into());
    }
    let label = crate::state::tab_model::webview_label_for_tab(tab_id);
    let view = app.get_webview(&label).ok_or("settings_tab.not_ready")?;
    if !view.url().is_ok_and(|url| is_document(app, &url)) {
        return Err("settings_tab.not_ready".into());
    }
    let generation = state
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .map_err(|e| e.to_string())?
        .get(&label)
        .cloned()
        .ok_or("settings_tab.not_ready")?;
    let mut payload = serde_json::to_value(&presentation).map_err(|e| e.to_string())?;
    if serde_json::to_vec(&payload)
        .map_err(|e| e.to_string())?
        .len()
        > 2_000_000
    {
        return Err("settings_tab.presentation_too_large".into());
    }
    let revision = payload
        .get("revision")
        .and_then(|value| value.as_u64())
        .ok_or("settings_tab.invalid_presentation")?;
    let service = app.state::<SettingsTabSessions>();
    let mut sessions = service.0.lock().map_err(|e| e.to_string())?;
    let entry = sessions.entry(label).or_insert_with(|| Session {
        token: super::webview_manager::create_internal_bridge_token(),
        document: super::webview_manager::create_internal_bridge_token(),
        generation: generation.clone(),
        sequence: 0,
        session_id,
        revision: 0,
    });
    if !entry.advance(&generation, session_id, revision)? {
        return Ok(());
    }
    payload["visible"] = true.into();
    payload["sessionId"] = session_id.into();
    payload["focus"] = tab.is_active.into();
    payload["token"] = entry.token.clone().into();
    payload["document"] = entry.document.clone().into();
    drop(sessions);
    let expected = serde_json::to_string(document_url(app)?.as_str()).map_err(|e| e.to_string())?;
    view.eval(format!("(() => {{ const expected={expected}; if(location.href!==expected || window.top!==window) return; const payload={payload}; let tries=0; const send=()=>{{ if(location.href!==expected || window.top!==window) return; if(window.__sparkSetSettingsPopover) window.__sparkSetSettingsPopover(payload); else if(++tries<120) setTimeout(send,16); }}; send(); }})();")).map_err(|e| e.to_string())
}

pub fn handle(app: &AppHandle, tab_id: &str, label: &str, generation: &str, url: &Url) {
    if url.path() != "/action"
        || !super::navigation_state::is_current_generation(app, label, generation)
    {
        return;
    }
    let Some(values) =
        crate::security::bridge::query(url, &["token", "document", "sequence", "payload"])
    else {
        return;
    };
    let Some((token, document, sequence, payload)) = values
        .get("token")
        .zip(values.get("document"))
        .zip(values.get("sequence").and_then(|s| s.parse::<u64>().ok()))
        .zip(values.get("payload"))
        .map(|(((a, b), c), d)| (a, b, c, d))
    else {
        return;
    };
    let state = app.state::<crate::state::browser_state::BrowserState>();
    if !state.session().tabs().lock().is_ok_and(|tabs| {
        tabs.iter()
            .any(|tab| tab.id == tab_id && is_settings_url(&tab.url))
    }) || !app
        .get_webview(label)
        .and_then(|view| view.url().ok())
        .is_some_and(|url| is_document(app, &url))
    {
        return;
    }
    let service = app.state::<SettingsTabSessions>();
    let action = service.accept(label, token, document, generation, sequence, payload);
    if let Some((action, session_id)) = action {
        crate::error_log::ignored(crate::security::emit_shell_for_tab(
            app,
            tab_id,
            "settings-tab-action",
            serde_json::json!({ "sessionId": session_id, "tabId": tab_id, "sequence": sequence, "action": action }),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_rejects_stale_documents_tokens_and_untyped_actions() {
        let sessions = SettingsTabSessions::default();
        assert!(sessions
            .register(
                "tab-1",
                String::new(),
                "doc-1".into(),
                "generation-1".into()
            )
            .is_err());
        sessions
            .register(
                "tab-1",
                "secret".into(),
                "doc-1".into(),
                "generation-1".into(),
            )
            .unwrap();
        let good = r#"{"kind":"click","target":1}"#;
        assert!(sessions
            .accept("tab-2", "secret", "doc-1", "generation-1", 1, good)
            .is_none());
        assert!(sessions
            .accept("tab-1", "wrong", "doc-1", "generation-1", 1, good)
            .is_none());
        assert!(sessions
            .accept("tab-1", "secret", "old", "generation-1", 1, good)
            .is_none());
        assert!(sessions
            .accept("tab-1", "secret", "doc-1", "old", 1, good)
            .is_none());
        assert!(sessions
            .accept(
                "tab-1",
                "secret",
                "doc-1",
                "generation-1",
                1,
                r#"{"kind":"invoke","command":"close_tab"}"#
            )
            .is_none());
        assert!(sessions
            .accept("tab-1", "secret", "doc-1", "generation-1", 0, good)
            .is_none());
        assert!(sessions
            .accept("tab-1", "secret", "doc-1", "generation-1", 1, good)
            .is_some());
        assert!(sessions
            .accept("tab-1", "secret", "doc-1", "generation-1", 1, good)
            .is_none());
        sessions
            .register(
                "tab-1",
                "new-secret".into(),
                "doc-2".into(),
                "generation-2".into(),
            )
            .unwrap();
        assert!(sessions
            .accept("tab-1", "secret", "doc-1", "generation-1", 2, good)
            .is_none());
        assert!(sessions
            .accept("tab-1", "new-secret", "doc-2", "generation-2", 1, good)
            .is_some());
        sessions.revoke("tab-1");
        assert!(sessions
            .accept("tab-1", "new-secret", "doc-2", "generation-2", 2, good)
            .is_none());
    }

    #[test]
    fn settings_address_is_exact() {
        assert!(is_settings_url(URL));
        for value in [
            "spark://settings/",
            "spark://settings?x=1",
            "spark://settings#x",
            "https://settings",
        ] {
            assert!(!is_settings_url(value));
        }
    }

    #[test]
    fn newer_sessions_rotate_tokens_and_reject_old_revisions() {
        let mut session = Session {
            token: "old".into(),
            document: "old-doc".into(),
            generation: "generation".into(),
            sequence: 3,
            session_id: 4,
            revision: 5,
        };
        assert_eq!(session.advance("generation", 4, 4), Ok(false));
        assert!(session.advance("old-generation", 5, 1).is_err());
        assert_eq!(session.advance("generation", 5, 1), Ok(true));
        assert_ne!(session.token, "old");
        assert_ne!(session.document, "old-doc");
        assert_eq!(session.sequence, 0);
        assert!(session.advance("generation", 4, 9).is_err());
    }
}
