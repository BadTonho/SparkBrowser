use super::{action, current, MediaPopover};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager, Url};

/// Child content pages have no IPC grants. Only this WebView's authenticated
/// navigation callback can forward the two fixed popover actions to the shell.
pub(super) fn navigation(app: &AppHandle, url: &Url) {
    if url.scheme() != "spark-internal" || url.host_str() != Some("media-popover") {
        return;
    }
    let Some(parameters) =
        crate::security::bridge::query(url, &["token", "sequence", "value", "reason"])
    else {
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
    let _ =
        crate::window::overlay_host::emit_to_host(app, super::LABEL, "media-popover-action", event);
}
