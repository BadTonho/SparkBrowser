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

fn widget(operation: &str, data: &str) -> bool {
    match operation {
        "save" => serde_json::from_str::<Save>(data).is_ok_and(|value| {
            let _ = value.revision;
            value.settings.validate().is_ok()
        }),
        "cities" => serde_json::from_str::<Cities>(data)
            .is_ok_and(|value| value.query.trim().chars().count() >= 2 && value.query.len() <= 200),
        "read" | "panel" | "open_gmail" | "locate" | "refresh_weather" | "refresh_gmail"
        | "connect" | "cancel_connect" | "disconnect" => {
            serde_json::from_str::<Empty>(data).is_ok()
        }
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
    fn required_fields_and_cross_action_parameters_are_checked_before_session_changes() {
        let mut values = HashMap::from([("request".into(), "1".into())]);
        assert!(valid("/read", &values));
        values.insert("input".into(), "https://example.test/".into());
        assert!(!valid("/read", &values));
        assert!(!valid("/navigate", &values));
        assert!(!valid("/unknown", &values));
    }
}
