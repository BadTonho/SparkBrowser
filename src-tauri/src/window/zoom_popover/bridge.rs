use super::{action, current, ZoomPopover};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Emitter, Manager, Url};

/// The auxiliary page has no IPC grants; only its current authenticated session can act.
pub(super) fn navigation(app: &AppHandle, url: &Url) {
    if url.scheme() != "veloce-internal" || url.host_str() != Some("zoom-popover") {
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
    let raw_value = query("value");
    let value = raw_value
        .as_ref()
        .and_then(|value| value.parse::<f64>().ok());
    let service = app.state::<ZoomPopover>();
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
    if !matches!(name, "zoom" | "dismiss")
        || (name == "zoom" && value.is_none())
        || (raw_value.is_some()
            && !value.is_some_and(|v| valid_value(v, &session.presentation.levels)))
        || (name == "dismiss"
            && !matches!(reason.as_str(), "escape" | "tab" | "outside" | "trigger"))
    {
        return;
    }
    session.sequence = sequence;
    session.closing = name == "dismiss";
    let event = action(session, name, value, &reason);
    drop(inner);
    let _ = app.emit_to("main", "zoom-popover-action", event);
}

fn valid_value(value: f64, levels: &[f64]) -> bool {
    value.is_finite() && (0.25..=5.0).contains(&value) && levels.contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adjustments_require_finite_supported_levels() {
        let levels = [0.25, 1.0, 1.25, 5.0];
        assert!(valid_value(1.25, &levels));
        for value in [0.0, 5.1, 1.37, f64::NAN, f64::INFINITY] {
            assert!(!valid_value(value, &levels));
        }
    }
}
