use super::{action, current, MediaPopover};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Emitter, Manager, Url};

/// Child content pages have no IPC grants. Only this WebView's authenticated
/// navigation callback can forward the two fixed popover actions to the shell.
pub(super) fn navigation(app: &AppHandle, url: &Url) {
    if url.scheme() != "veloce-internal" || url.host_str() != Some("media-popover") {
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
    let value = query("value")
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|value| *value <= 200);
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
    if !matches!(name, "volume" | "dismiss")
        || (name == "volume" && (session.presentation.disabled || value.is_none()))
    {
        return;
    }
    if name == "dismiss" && !matches!(reason.as_str(), "escape" | "tab" | "outside" | "trigger") {
        return;
    }
    session.sequence = sequence;
    session.closing = name == "dismiss";
    let event = action(
        session,
        name,
        value.filter(|_| !session.presentation.disabled),
        &reason,
    );
    drop(inner);
    // The shell hides the WebView after delivery, rather than interrupting its
    // own navigation callback, and flushes the last slider value on dismissal.
    let _ = app.emit_to("main", "media-popover-action", event);
}
