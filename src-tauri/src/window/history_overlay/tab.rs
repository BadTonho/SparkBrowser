//! The history panel in its own tab (`spark://history`), chosen in Settings → Appearance & Layout.
//!
//! It is the same page as the panel (`history-overlay.html`, with `?surface=tab`), served from the
//! app's own assets. Like the settings tab, the shell stays the only caller of commands: it sends the
//! rows to the page, and the page only sends typed actions back through the content bridge, accepted
//! for the document, session and tokens that received the rows, and only for visits it shows.
use super::{bridge, ActionPayload, Presentation};
use crate::sync::Mutex;
use std::collections::HashMap;
use tauri::{AppHandle, Manager, Url};

pub const URL: &str = "spark://history";

pub fn is_history_url(raw: &str) -> bool {
    raw == URL
}

pub fn document_url(app: &AppHandle) -> Result<Url, String> {
    let mut url = crate::security::documents::asset_base(app)
        .join("/history-overlay.html")
        .map_err(|e| e.to_string())?;
    url.set_query(Some("surface=tab"));
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
    presentation: Presentation,
}

/// Sessions by content web view label.
#[derive(Default)]
pub struct HistoryTabSessions(Mutex<HashMap<String, Session>>);

impl HistoryTabSessions {
    /// Forgets the session of a web view that navigated or closed.
    pub fn revoke(&self, label: &str) {
        if let Ok(mut sessions) = self.0.lock() {
            sessions.remove(label);
        }
    }
}

/// Sends the rows to the history tab `tab_id` of `window_id`.
pub fn present(
    app: &AppHandle,
    window_id: &str,
    tab_id: &str,
    session_id: u64,
    presentation: Presentation,
) -> Result<(), String> {
    if !presentation.is_valid()
        || presentation.tab_id != tab_id
        || !is_history_url(&presentation.url)
    {
        return Err("history_tab.invalid_presentation".into());
    }
    let state = app.state::<crate::state::browser_state::BrowserState>();
    let tab = state
        .session()
        .tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|tab| tab.id == tab_id)
        .cloned()
        .ok_or("history_tab.invalid_tab")?;
    if tab.window_id != window_id || !is_history_url(&tab.url) {
        return Err("history_tab.invalid_tab".into());
    }
    if tab.is_hibernated || tab.navigation.loading {
        return Err("history_tab.not_ready".into());
    }
    let label = crate::state::tab_model::webview_label_for_tab(tab_id);
    let view = app.get_webview(&label).ok_or("history_tab.not_ready")?;
    if !view.url().is_ok_and(|url| is_document(app, &url)) {
        return Err("history_tab.not_ready".into());
    }
    let generation = state
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .map_err(|e| e.to_string())?
        .get(&label)
        .cloned()
        .ok_or("history_tab.not_ready")?;
    let service = app.state::<HistoryTabSessions>();
    let mut sessions = service.0.lock().map_err(|e| e.to_string())?;
    let renew = sessions
        .get(&label)
        .is_none_or(|session| session.generation != generation || session.session_id < session_id);
    if !renew
        && sessions
            .get(&label)
            .is_some_and(|session| session.session_id > session_id)
    {
        return Ok(());
    }
    let session = if renew {
        sessions.insert(
            label.clone(),
            Session {
                token: super::create_internal_bridge_token(),
                document: super::create_internal_bridge_token(),
                generation,
                sequence: 0,
                session_id,
                presentation,
            },
        );
        sessions.get(&label)
    } else {
        let session = sessions.get_mut(&label).ok_or("history_tab.not_ready")?;
        session.presentation = presentation;
        Some(&*session)
    }
    .ok_or("history_tab.not_ready")?;
    let mut payload = serde_json::to_value(&session.presentation).map_err(|e| e.to_string())?;
    payload["visible"] = true.into();
    payload["sessionId"] = session_id.into();
    payload["token"] = session.token.clone().into();
    payload["document"] = session.document.clone().into();
    drop(sessions);
    let expected = serde_json::to_string(document_url(app)?.as_str()).map_err(|e| e.to_string())?;
    view.eval(format!("(() => {{ const expected={expected}; if(location.href!==expected || window.top!==window) return; const payload={payload}; let tries=0; const send=()=>{{ if(location.href!==expected || window.top!==window) return; if(window.__sparkSetHistoryOverlay) window.__sparkSetHistoryOverlay(payload); else if(++tries<120) setTimeout(send,16); }}; send(); }})();")).map_err(|e| e.to_string())
}

/// An action from the history tab, sent to the shell of the tab's window.
pub fn handle(app: &AppHandle, tab_id: &str, label: &str, generation: &str, url: &Url) {
    if url.path() != "/action"
        || !super::super::navigation_state::is_current_generation(app, label, generation)
        || !app
            .get_webview(label)
            .and_then(|view| view.url().ok())
            .is_some_and(|url| is_document(app, &url))
    {
        return;
    }
    let Some(values) =
        crate::security::bridge::query(url, &["token", "document", "sequence", "payload"])
    else {
        return;
    };
    let (Some(token), Some(document), Some(sequence), Some(payload)) = (
        values.get("token"),
        values.get("document"),
        values.get("sequence").and_then(|s| s.parse::<u64>().ok()),
        values
            .get("payload")
            .and_then(|value| serde_json::from_str::<ActionPayload>(value).ok()),
    ) else {
        return;
    };
    // The tab has no panel to close; everything else is the panel's actions.
    if payload.action == "close" || !bridge::allowed(&payload) {
        return;
    }
    let service = app.state::<HistoryTabSessions>();
    let Ok(mut sessions) = service.0.lock() else {
        return;
    };
    let Some(session) = sessions.get_mut(label) else {
        return;
    };
    if session.generation != generation
        || &session.token != token
        || &session.document != document
        || sequence <= session.sequence
        || session.presentation.tab_id != tab_id
    {
        return;
    }
    let Some(event) = bridge::action_event(session.session_id, &session.presentation, payload)
    else {
        return;
    };
    session.sequence = sequence;
    drop(sessions);
    crate::error_log::ignored(crate::security::emit_shell_for_tab(
        app,
        tab_id,
        "history-tab-action",
        event,
    ));
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_the_exact_internal_address_is_the_history_tab() {
        assert!(super::is_history_url("spark://history"));
        for raw in [
            "spark://history/",
            "spark://history?x=1",
            "spark://settings",
            "https://history",
        ] {
            assert!(!super::is_history_url(raw), "{raw}");
        }
    }
}
