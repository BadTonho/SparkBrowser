//! Validate action-specific fields before advancing a new-tab session.
use crate::database::new_tab_repo::Settings;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    revision: u64,
    settings: Settings,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cities {
    query: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Seek {
    seconds: f64,
}

fn widget(operation: &str, data: &str) -> bool {
    match operation {
        "save" => serde_json::from_str::<Save>(data).is_ok_and(|value| {
            let _ = value.revision;
            value.settings.validate().is_ok()
        }),
        "cities" => serde_json::from_str::<Cities>(data)
            .is_ok_and(|value| value.query.trim().chars().count() >= 2 && value.query.len() <= 200),
        "read" | "panel" | "open_gmail" | "locate" | "refresh_weather" | "refresh_gmail"
        | "connect" | "cancel_connect" | "disconnect" | "spotify_connect" | "spotify_cancel"
        | "spotify_disconnect" | "spotify_open" | "spotify_play" | "spotify_pause"
        | "spotify_next" | "spotify_previous" | "audio_subscribe" | "audio_unsubscribe"
        | "youtube_toggle" | "youtube_next" | "youtube_previous" | "youtube_open" => {
            serde_json::from_str::<Empty>(data).is_ok()
        }
        "youtube_seek" => serde_json::from_str::<Seek>(data)
            .is_ok_and(|value| value.seconds.is_finite() && (0.0..=86_400.0).contains(&value.seconds)),
        _ => false,
    }
}
pub(super) fn valid(path: &str, values: &HashMap<String, String>) -> bool {
    let required = |key: &str| values.get(key).is_some_and(|v| !v.trim().is_empty());
    let request = || {
        values
            .get("request")
            .and_then(|v| v.parse::<u64>().ok())
            .is_some_and(|v| v > 0)
    };
    let fields: &[&str] = match path {
        "/read" if request() => &["request"],
        "/navigate" if required("input") => &["input"],
        "/visit" if required("input") && required("site_key") => &["input", "site_key"],
        "/widgets"
            if request()
                && values
                    .get("operation")
                    .zip(values.get("data"))
                    .is_some_and(|(op, data)| widget(op, data)) =>
        {
            &["request", "operation", "data"]
        }
        "/error"
            if required("message")
                && values.get("message").is_some_and(|v| v.len() <= 2_000)
                && values.get("details").is_none_or(|v| v.len() <= 6_000)
                && values.get("level").is_none_or(|v| v == "warning") =>
        {
            &["message", "details", "level"]
        }
        _ => return false,
    };
    values.keys().all(|key| {
        matches!(key.as_str(), "token" | "document" | "sequence") || fields.contains(&key.as_str())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn widget_data_rejects_unknown_actions_types_and_duplicate_json_fields() {
        assert!(widget("panel", "{}"));
        assert!(widget("cities", r#"{"query":"São Paulo"}"#));
        for (op, data) in [
            ("invoke", "{}"),
            ("panel", r#"{"command":"close_tab"}"#),
            ("cities", r#"{"query":22}"#),
            ("cities", r#"{"query":"AA","query":"BB"}"#),
            (
                "save",
                r#"{"revision":1,"settings":{"weather_enabled":true,"weather_enabled":false}}"#,
            ),
            ("save", r#"{"settings":{}}"#),
        ] {
            assert!(!widget(op, data));
        }
    }
    #[test]
    fn youtube_controls_are_allowed_but_carry_no_data() {
        for op in [
            "youtube_toggle",
            "youtube_next",
            "youtube_previous",
            "youtube_open",
        ] {
            assert!(widget(op, "{}"), "{op}");
            assert!(!widget(op, r#"{"script":"alert(1)"}"#), "{op}");
        }
        assert!(!widget("youtube_eval", "{}"));
    }
    #[test]
    fn youtube_seek_needs_a_bounded_number_of_seconds() {
        assert!(widget("youtube_seek", r#"{"seconds":42.5}"#));
        for data in [r#"{"seconds":-1}"#, r#"{"seconds":999999}"#, r#"{"seconds":"5"}"#, "{}", r#"{"seconds":1,"x":2}"#] {
            assert!(!widget("youtube_seek", data), "{data}");
        }
    }
    #[test]
    fn required_fields_and_cross_action_parameters_are_checked_before_session_changes() {
        let mut values = HashMap::from([("request".into(), "1".into())]);
        assert!(valid("/read", &values));
        values.insert("input".into(), "https://example.test/".into());
        assert!(!valid("/read", &values));
        assert!(!valid("/navigate", &values));
        assert!(!valid("/unknown", &values));
        let error = HashMap::from([("message".into(), "x is not defined".into())]);
        assert!(valid("/error", &error));
        assert!(!valid("/error", &HashMap::new()));
        let mut warning = error.clone();
        warning.insert("level".into(), "warning".into());
        assert!(valid("/error", &warning));
        warning.insert("level".into(), "critical".into());
        assert!(!valid("/error", &warning));
        assert!(!valid(
            "/error",
            &HashMap::from([("message".into(), "x".repeat(2_001))])
        ));
    }
}
