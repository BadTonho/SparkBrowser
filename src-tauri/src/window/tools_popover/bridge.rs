use super::{current, Action, ToolsPopover};
use crate::state::browser_state::BrowserState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Url};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum DismissReason {
    Escape,
    Outside,
    Trigger,
    Blur,
}
#[derive(Clone, Deserialize, Serialize)]
pub(super) enum TabKey {
    ArrowLeft,
    ArrowRight,
    Home,
    End,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub(super) enum UiAction {
    Click {
        target: u32,
    },
    Submit {
        target: u32,
    },
    Input {
        target: u32,
        value: String,
        checked: bool,
    },
    Change {
        target: u32,
        value: String,
        checked: bool,
    },
    Key {
        target: u32,
        key: TabKey,
    },
    Dismiss {
        reason: DismissReason,
    },
}
impl UiAction {
    fn valid(&self) -> bool {
        match self {
            Self::Click { target } | Self::Submit { target } | Self::Key { target, .. } => {
                *target > 0
            }
            Self::Input { target, value, .. } | Self::Change { target, value, .. } => {
                *target > 0 && value.len() <= 1_000_000
            }
            Self::Dismiss { .. } => true,
        }
    }
}

/// Only this packaged surface's callback can forward authenticated UI actions.
/// An opaque control handle never selects a native command or an arbitrary DOM selector.
pub(super) fn navigation(app: &AppHandle, url: &Url) {
    if url.scheme() != "veloce-internal"
        || url.host_str() != Some("tools-popover")
        || url.path() != "/action"
    {
        return;
    }
    let query = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    };
    let Some(sequence) = query("sequence").and_then(|value| value.parse::<u64>().ok()) else {
        return;
    };
    let Some(value) =
        query("payload").and_then(|value| serde_json::from_str::<UiAction>(&value).ok())
    else {
        return;
    };
    if !value.valid() {
        return;
    }
    let service = app.state::<ToolsPopover>();
    let Ok(mut inner) = service.0.lock() else {
        return;
    };
    let Some(session) = inner.session.as_mut() else {
        return;
    };
    if session.closing
        || query("token").as_deref() != Some(&session.token)
        || sequence <= session.sequence
        || !current(app, &app.state::<BrowserState>(), session)
    {
        return;
    }
    session.sequence = sequence;
    session.closing = matches!(value, UiAction::Dismiss { .. });
    let event = Action {
        session_id: session.id,
        sequence,
        action: value,
    };
    drop(inner);
    let _ = app.emit_to("main", "tools-popover-action", event);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_typed_control_actions_are_accepted() {
        for value in [
            r#"{"kind":"invoke","command":"close_tab"}"#,
            r#"{"kind":"click","target":0}"#,
            r#"{"kind":"click","target":1,"command":"close_tab"}"#,
        ] {
            assert!(serde_json::from_str::<UiAction>(value).map_or(true, |v| !v.valid()));
        }
        assert!(
            serde_json::from_str::<UiAction>(r#"{"kind":"click","target":1}"#)
                .unwrap()
                .valid()
        );
    }
}
