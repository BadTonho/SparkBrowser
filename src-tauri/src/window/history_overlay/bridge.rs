use super::{valid_context, ActionEvent, ActionPayload, HistoryOverlay};
use tauri::{AppHandle, Manager, Url};

fn action_event(session: &super::Session, payload: ActionPayload) -> Option<ActionEvent> {
    let entry_url = if payload.action == "open" {
        let id = payload.entry_id?;
        Some(
            session
                .presentation
                .entries
                .iter()
                .find(|entry| entry.id == id)?
                .url
                .clone(),
        )
    } else {
        None
    };
    Some(ActionEvent {
        session_id: session.id,
        tab_id: session.presentation.tab_id.clone(),
        url: session.presentation.url.clone(),
        action: payload.action,
        query: payload.query,
        entry_id: payload.entry_id,
        entry_url,
    })
}

pub(super) fn navigation(app: &AppHandle, url: &Url) {
    if url.scheme() != "spark-internal"
        || url.host_str() != Some("history-overlay")
        || url.path() != "/action"
    {
        return;
    }
    let Some(parameters) = crate::security::bridge::query(url, &["token", "sequence", "payload"])
    else {
        return;
    };
    let Some(sequence) = parameters
        .get("sequence")
        .and_then(|value| value.parse::<u64>().ok())
    else {
        return;
    };
    let Some(payload) = parameters
        .get("payload")
        .and_then(|value| serde_json::from_str::<ActionPayload>(value).ok())
    else {
        return;
    };
    if !matches!(
        payload.action.as_str(),
        "close" | "search" | "clear" | "load-more" | "open"
    ) {
        return;
    }
    if payload.action == "search" && payload.query.len() > 2048 {
        return;
    }
    let service = app.state::<HistoryOverlay>();
    let Ok(mut inner) = service.0.lock() else {
        return;
    };
    let Some(session) = inner.session.as_mut() else {
        return;
    };
    let Some(token) = parameters.get("token") else {
        return;
    };
    if session.closing
        || token != &session.token
        || sequence <= session.sequence
        || !valid_context(
            &app.state::<crate::state::browser_state::BrowserState>(),
            session,
        )
    {
        return;
    }
    let Some(event) = action_event(session, payload) else {
        return;
    };
    session.sequence = sequence;
    session.closing = event.action == "close" || event.action == "open";
    drop(inner);
    let _ = crate::window::overlay_host::emit_to_host(
        app,
        super::LABEL,
        "history-overlay-action",
        event,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_action_only_uses_presented_entries() {
        let presentation = super::super::Presentation {
            tab_id: "tab-a".into(),
            url: "https://example.test".into(),
            locale: "en".into(),
            labels: super::super::Labels {
                title: "History".into(),
                clear: "Clear".into(),
                close: "Close".into(),
                search: "Search".into(),
                empty: "Empty".into(),
                load_more: "More".into(),
            },
            theme: super::super::Theme {
                surface: "#fff".into(),
                border: "#ddd".into(),
                text: "#111".into(),
                muted: "#555".into(),
                accent: "#555".into(),
                hover: "#eee".into(),
                field: "#fff".into(),
            },
            query: String::new(),
            entries: vec![super::super::Entry {
                id: 1,
                title: "Example".into(),
                url: "https://entry.test".into(),
                time: "12:00".into(),
            }],
            busy: false,
            error: String::new(),
            has_more: false,
        };
        let session = super::super::Session {
            id: 2,
            token: "token".into(),
            workspace: "ws".into(),
            presentation,
            sequence: 0,
            closing: false,
        };
        let valid = action_event(
            &session,
            ActionPayload {
                action: "open".into(),
                query: String::new(),
                entry_id: Some(1),
            },
        )
        .unwrap();
        assert_eq!(valid.entry_url.as_deref(), Some("https://entry.test"));
        assert!(action_event(
            &session,
            ActionPayload {
                action: "open".into(),
                query: String::new(),
                entry_id: Some(2)
            }
        )
        .is_none());
    }
}
