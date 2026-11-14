use super::{valid_context, ActionEvent, PermissionPopover};
use tauri::{AppHandle, Manager, Url};

fn valid_action(
    mode: &str,
    origin: &str,
    has_request: bool,
    payload: &super::ActionPayload,
) -> bool {
    let is_request = mode == "request" && has_request;
    match payload.action.as_str() {
        "dismiss" => {
            !is_request && matches!(payload.reason.as_str(), "escape" | "outside" | "trigger")
        }
        "refresh" | "reload" | "confirm-reload" | "cancel-reload" => !is_request,
        "set-permission" => {
            !is_request
                && payload.origin == origin
                && matches!(
                    payload.kind.as_str(),
                    "camera" | "microphone" | "location" | "notifications"
                )
                && matches!(payload.value.as_str(), "ask" | "allow" | "block")
        }
        "respond" => {
            is_request
                && matches!(
                    payload.decision.as_str(),
                    "block" | "block_always" | "allow_once" | "allow_always"
                )
        }
        _ => false,
    }
}

pub(super) fn navigation(app: &AppHandle, url: &Url) {
    if url.scheme() != "spark-internal"
        || url.host_str() != Some("permission-popover")
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
        .and_then(|value| serde_json::from_str::<super::ActionPayload>(value).ok())
    else {
        return;
    };
    let service = app.state::<PermissionPopover>();
    let Ok(mut inner) = service.0.lock() else {
        return;
    };
    let Some(session) = inner.session.as_mut() else {
        return;
    };
    if session.closing
        || parameters.get("token") != Some(&session.token)
        || sequence <= session.sequence
        || !valid_context(
            app,
            &app.state::<crate::state::browser_state::BrowserState>(),
            session,
        )
    {
        return;
    }
    let request = session.presentation.request.as_ref();
    let valid = valid_action(
        &session.presentation.mode,
        &session.presentation.origin,
        request.is_some(),
        &payload,
    );
    if !valid {
        return;
    }
    session.sequence = sequence;
    session.closing = payload.action == "dismiss" || payload.action == "respond";
    let event = ActionEvent {
        session_id: session.id,
        tab_id: session.presentation.tab_id.clone(),
        url: session.presentation.url.clone(),
        origin: session.presentation.origin.clone(),
        request_id: request.map(|value| value.id.clone()),
        action: payload.action,
        kind: payload.kind,
        value: payload.value,
        decision: payload.decision,
        reason: payload.reason,
    };
    drop(inner);
    let _ = crate::security::emit_shell(app, "site-permission-popover-action", event);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn action(value: &str) -> super::super::ActionPayload {
        serde_json::from_str(value).unwrap()
    }
    #[test]
    fn permission_controls_are_limited_to_the_presented_site_and_actions() {
        assert!(valid_action(
            "settings",
            "https://site.test",
            false,
            &action(
                r#"{"action":"set-permission","origin":"https://site.test","kind":"camera","value":"allow"}"#
            )
        ));
        assert!(!valid_action(
            "settings",
            "https://site.test",
            false,
            &action(
                r#"{"action":"set-permission","origin":"https://other.test","kind":"camera","value":"allow"}"#
            )
        ));
        assert!(!valid_action(
            "settings",
            "https://site.test",
            false,
            &action(
                r#"{"action":"set-permission","origin":"https://site.test","kind":"unknown","value":"allow"}"#
            )
        ));
    }
    #[test]
    fn request_actions_require_a_pending_request_and_fixed_decisions() {
        assert!(valid_action(
            "request",
            "https://site.test",
            true,
            &action(r#"{"action":"respond","decision":"allow_once"}"#)
        ));
        assert!(!valid_action(
            "request",
            "https://site.test",
            false,
            &action(r#"{"action":"respond","decision":"allow_once"}"#)
        ));
        assert!(!valid_action(
            "request",
            "https://site.test",
            true,
            &action(r#"{"action":"respond","decision":"close_tab"}"#)
        ));
    }
}
