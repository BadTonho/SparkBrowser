use super::{valid_context, ActionEvent, ActionPayload, HistoryOverlay};
use tauri::{AppHandle, Manager, Url};

/// Builds the event for the shell. Visits are only taken from the ones the panel shows, so the page
/// can never ask to open or delete anything else.
fn action_event(session: &super::Session, payload: ActionPayload) -> Option<ActionEvent> {
    let entries = &session.presentation.entries;
    let find = |id: i64| entries.iter().find(|entry| entry.id == id);
    let entry_url = if payload.action == "open" {
        Some(find(payload.entry_id?)?.url.clone())
    } else {
        None
    };
    let entry_host = if payload.action == "more-from-site" {
        Some(find(payload.entry_id?)?.host.clone()).filter(|host| !host.is_empty())
    } else {
        None
    };
    if payload.action == "more-from-site" && entry_host.is_none() {
        return None;
    }
    let mut entry_ids = Vec::new();
    let mut entry_urls = Vec::new();
    if matches!(payload.action.as_str(), "open-many" | "delete") {
        if payload.entry_ids.is_empty() || payload.entry_ids.len() > super::MAX_SELECTED {
            return None;
        }
        for id in &payload.entry_ids {
            let entry = find(*id)?;
            if !entry_ids.contains(id) {
                entry_ids.push(*id);
                entry_urls.push(entry.url.clone());
            }
        }
    }
    Some(ActionEvent {
        session_id: session.id,
        tab_id: session.presentation.tab_id.clone(),
        url: session.presentation.url.clone(),
        action: payload.action,
        query: payload.query,
        entry_id: payload.entry_id,
        entry_url,
        entry_ids,
        entry_urls,
        entry_host,
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
        "close"
            | "search"
            | "clear"
            | "load-more"
            | "open"
            | "open-many"
            | "delete"
            | "more-from-site"
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
    session.closing = matches!(event.action.as_str(), "close" | "open" | "open-many");
    drop(inner);
    crate::error_log::ignored(crate::window::overlay_host::emit_to_host(
        app,
        super::LABEL,
        "history-overlay-action",
        event,
    ));
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
                selected: "{count} selected".into(),
                open: "Open".into(),
                delete: "Delete".into(),
                cancel_selection: "Cancel".into(),
                more_from_site: "More from this site".into(),
                delete_entry: "Delete from history".into(),
                menu: "Actions".into(),
                select: "Select".into(),
            },
            theme: super::super::Theme {
                surface: "#fff".into(),
                surface_border: None,
                surface_filter: None,
                surface_shadow: None,
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
                host: "entry.test".into(),
                day: "Today".into(),
                gap: false,
                site: None,
            }],
            icons: Default::default(),
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
        let payload = |action: &str, entry_id: Option<i64>, entry_ids: Vec<i64>| ActionPayload {
            action: action.into(),
            query: String::new(),
            entry_id,
            entry_ids,
        };
        let valid = action_event(&session, payload("open", Some(1), vec![])).unwrap();
        assert_eq!(valid.entry_url.as_deref(), Some("https://entry.test"));
        assert!(action_event(&session, payload("open", Some(2), vec![])).is_none());
        // A selection may only hold shown visits; repeats are dropped.
        let delete = action_event(&session, payload("delete", None, vec![1, 1])).unwrap();
        assert_eq!(delete.entry_ids, [1]);
        let many = action_event(&session, payload("open-many", None, vec![1])).unwrap();
        assert_eq!(many.entry_urls, ["https://entry.test"]);
        assert!(action_event(&session, payload("delete", None, vec![1, 2])).is_none());
        assert!(action_event(&session, payload("delete", None, vec![])).is_none());
        let site = action_event(&session, payload("more-from-site", Some(1), vec![])).unwrap();
        assert_eq!(site.entry_host.as_deref(), Some("entry.test"));
        assert!(action_event(&session, payload("more-from-site", Some(2), vec![])).is_none());
    }
}
