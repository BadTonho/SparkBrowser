//! Fixed, bounded document reports. These messages cannot name native commands.
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Signal {
    token: String,
    pub document: String,
    pub sequence: u64,
    pub playing: bool,
    pub picture_in_picture: bool,
    pub pip_video: Option<String>,
    pub pip_mode: Option<String>,
    pub pip_session: Option<String>,
    pub position: Option<f64>,
    pub duration: Option<f64>,
}
pub(super) fn parse(raw: &str, token: &str) -> Option<Signal> {
    if raw.len() > 2048 {
        return None;
    }
    let signal: Signal = serde_json::from_str(raw).ok()?;
    let bounded = |s: &str| !s.is_empty() && s.len() <= 128;
    if signal.token != token
        || !bounded(&signal.document)
        || signal.sequence == 0
        || signal
            .position
            .is_some_and(|v| !v.is_finite() || !(0.0..=1.0e7).contains(&v))
        || signal
            .duration
            .is_some_and(|v| !v.is_finite() || !(0.0..=1.0e7).contains(&v))
        || signal.pip_video.as_deref().is_some_and(|s| !bounded(s))
        || signal.pip_session.as_deref().is_some_and(|s| !bounded(s))
        || signal
            .pip_mode
            .as_deref()
            .is_some_and(|s| !matches!(s, "native" | "custom"))
    {
        return None;
    }
    Some(signal)
}

#[cfg(test)]
mod tests {
    use super::*;
    const VALID: &str = r#"{"token":"secret","document":"doc","sequence":1,"playing":true,"picture_in_picture":false,"pip_video":null,"pip_mode":null,"pip_session":null}"#;
    #[test]
    fn invalid_reports_cannot_clear_media_activity() {
        assert!(parse(VALID, "secret").is_some());
        assert!(parse(VALID, "wrong").is_none());
        for (from, to) in [
            ("\"playing\":true", "\"playing\":\"false\""),
            ("\"playing\":true", "\"playing\":true,\"playing\":false"),
            ("\"sequence\":1", "\"sequence\":0"),
            ("\"pip_mode\":null", "\"pip_mode\":\"arbitrary\""),
            ("\"pip_video\":null", "\"command\":\"close_tab\""),
            ("\"pip_mode\":null", "\"pip_mode\":null,\"position\":-1"),
            ("\"pip_mode\":null", "\"pip_mode\":null,\"duration\":1e9"),
            ("\"pip_mode\":null", "\"pip_mode\":null,\"position\":\"7\""),
        ] {
            assert!(parse(&VALID.replace(from, to), "secret").is_none());
        }
        assert!(parse(&"a".repeat(2049), "secret").is_none());
        let with_progress = VALID.replace(
            "\"pip_mode\":null",
            "\"pip_mode\":null,\"position\":12.5,\"duration\":200.0",
        );
        let signal = parse(&with_progress, "secret").unwrap();
        assert_eq!(
            (signal.position, signal.duration),
            (Some(12.5), Some(200.0))
        );
    }
}
