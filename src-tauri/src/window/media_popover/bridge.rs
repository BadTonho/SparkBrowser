use super::{action, current, MediaPopover};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager, Url};

/// Child content pages have no IPC grants. Only this WebView's authenticated
/// navigation callback can forward the two fixed popover actions to the shell.
pub(super) fn navigation(app: &AppHandle, url: &Url) {
    if url.scheme() != "veloce-internal" || url.host_str() != Some("media-popover") {
        return;
    }
    let Some(parameters) = crate::security::bridge::query(
        url,
        &["token", "sequence", "value", "reason", "targetTabId"],
    ) else {
        return;
    };
    let query = |name: &str| parameters.get(name).cloned();
    let Some(sequence) = query("sequence").and_then(|value| value.parse::<u64>().ok()) else {
        return;
    };
    let value = query("value")
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|value| *value <= 200);
    if parameters.contains_key("value") && value.is_none() {
        return;
    }
    let service = app.state::<MediaPopover>();
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
    let name = url.path().trim_matches('/');
    let reason = query("reason").unwrap_or_default();
    if !matches!(name, "volume" | "dismiss" | "select" | "mute")
        || (name == "volume" && (session.presentation.disabled || value.is_none()))
    {
        return;
    }
    if name == "dismiss" && !matches!(reason.as_str(), "escape" | "tab" | "outside" | "trigger") {
        return;
    }
    let target_tab_id = if matches!(name, "select" | "mute") {
        let Some(target) = query("targetTabId") else {
            return;
        };
        let Some(item) = session
            .presentation
            .items
            .iter()
            .find(|item| item.tab_id == target)
        else {
            return;
        };
        if name == "mute" && !item.mute_available {
            return;
        }
        Some(target)
    } else {
        None
    };
    session.sequence = sequence;
    session.closing = name == "dismiss";
    let mut event = action(
        session,
        name,
        value.filter(|_| !session.presentation.disabled),
        &reason,
    );
    event.target_tab_id = target_tab_id;
    drop(inner);
    // The shell hides the WebView after delivery, rather than interrupting its
    // own navigation callback, and flushes the last slider value on dismissal.
    let _ = crate::security::emit_shell(app, "media-popover-action", event);
}
