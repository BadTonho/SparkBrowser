#[cfg(windows)]
use super::MediaService;
use super::MediaSnapshot;
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use serde_json::json;
use tauri::AppHandle;
#[cfg(windows)]
use tauri::Manager;

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FrameMedia {
    #[serde(default)]
    pub context: String,
    pub document: String,
    pub count: u32,
    pub applied_volume: u16,
    pub status: String,
    #[serde(default)]
    pub videos: Vec<super::pip::MediaVideoCandidate>,
    #[serde(default)]
    pub playing: bool,
    #[serde(default)]
    pub picture_in_picture: bool,
    #[serde(default)]
    pub pip_video: Option<String>,
    #[serde(default)]
    pub pip_mode: Option<String>,
    #[serde(default)]
    pub pip_session: Option<String>,
}
pub fn validate_volume(value: u16) -> Result<(), String> {
    if value > 200 {
        Err("media.invalid_volume".into())
    } else {
        Ok(())
    }
}
#[cfg(any(windows, test))]
fn document_key(frames: &[FrameMedia]) -> String {
    let mut keys: Vec<_> = frames
        .iter()
        .map(|f| format!("{}:{}", f.context, f.document))
        .collect();
    keys.sort();
    keys.join("|")
}
#[cfg(any(windows, test))]
fn validate_document(expected: &str, actual: &str, owned_pip: bool) -> Result<(), String> {
    // Removing an unrelated iframe clears the aggregate key. A PiP edit may
    // still proceed only after its source session matches the fresh reports.
    if !owned_pip && (expected.is_empty() || expected != actual) {
        return Err("media.stale".into());
    }
    Ok(())
}
#[cfg(windows)]
async fn reports(app: &AppHandle, id: &str) -> Result<(String, Vec<FrameMedia>), String> {
    let (token, contexts) = super::protocol::contexts(app, id).await?;
    let mut frames = Vec::new();
    for context in contexts {
        match super::protocol::evaluate(app, id, &token, &context, json!({"kind":"report"}), false)
            .await
        {
            Ok(value) => {
                if let Ok(mut f) = serde_json::from_value::<FrameMedia>(value) {
                    f.context = context;
                    frames.push(f);
                }
            }
            Err(e) if e == "media.unavailable" || e == "media.stale" => {}
            Err(e) => return Err(e),
        }
    }
    if frames.is_empty() {
        return Err("media.unavailable".into());
    }
    Ok((token, frames))
}
pub async fn refresh(app: &AppHandle, id: &str) -> Result<MediaSnapshot, String> {
    #[cfg(windows)]
    {
        let (_, frames) = reports(app, id).await?;
        let relevant: Vec<_> = frames.iter().filter(|f| f.count > 0).collect();
        let previous = app.state::<MediaService>().tab(id);
        let desired = previous.volume;
        let applied = relevant
            .iter()
            .map(|f| f.applied_volume)
            .min()
            .unwrap_or(100);
        let status = if relevant.is_empty() {
            "unavailable"
        } else if relevant.iter().any(|f| f.status == "limited") {
            "limited"
        } else if relevant.iter().any(|f| f.status == "pending") {
            "pending"
        } else if relevant.iter().any(|f| f.status == "unavailable") {
            "unavailable"
        } else {
            "ready"
        };
        let status = if previous.document_key == document_key(&frames)
            && desired == 100
            && previous.volume_status == "limited"
        {
            "limited"
        } else {
            status
        };
        app.state::<MediaService>().update(app, id, |t| {
            t.applied_volume = if desired == 0 { 0 } else { applied };
            t.volume_status = status.into();
            t.document_key = document_key(&frames);
            t.frames = frames;
            if status == "limited" {
                t.volume = 100;
            }
        });
        super::signals::sync_activity(app, id);
        app.state::<MediaService>().snapshot(app)
    }
    #[cfg(not(windows))]
    {
        let _ = (app, id);
        Err("media.unavailable".into())
    }
}
pub async fn set(
    app: &AppHandle,
    id: &str,
    value: u16,
    expected: &str,
    pip_session: Option<&str>,
) -> Result<MediaSnapshot, String> {
    validate_volume(value)?;
    #[cfg(windows)]
    {
        let (token, frames) = reports(app, id).await?;
        let actual = document_key(&frames);
        let owned = pip_session
            .is_some_and(|session| super::pip_window::matches(session, id, &token, &frames));
        validate_document(expected, &actual, owned)?;
        let expected = if owned { actual.as_str() } else { expected };
        let previous = app.state::<MediaService>().tab(id);
        let mut result = Ok(());
        let mut status = "ready";
        let mut applied = value;
        for f in &frames {
            match super::protocol::evaluate(
                app,
                id,
                &token,
                &f.context,
                json!({"kind":"volume","document":f.document,"value":value}),
                true,
            )
            .await
            {
                Ok(v) if v["error"].is_string() => {
                    result = Err("media.stale".into());
                    break;
                }
                Ok(v) => {
                    if f.count > 0 && v["status"] != "ready" {
                        status = if v["status"] == "pending" {
                            "pending"
                        } else if value > 100 {
                            "limited"
                        } else {
                            "unavailable"
                        };
                        applied = 100;
                    }
                }
                Err(e) => {
                    result = Err(e);
                    break;
                }
            }
        }
        if !frames.iter().any(|f| f.count > 0) && !matches!(value, 0 | 100) {
            status = "unavailable";
            applied = 100;
        }
        let fallback = if result.is_err() {
            previous.volume
        } else {
            applied
        };
        if result.is_err() || applied != value {
            for f in &frames {
                let _ = super::protocol::evaluate(
                    app,
                    id,
                    &token,
                    &f.context,
                    json!({"kind":"volume","document":f.document,"value":fallback}),
                    false,
                )
                .await;
            }
        }
        result?;
        if let Some(view) = app.get_webview(&crate::state::tab_model::webview_label_for_tab(id)) {
            super::windows::set_muted(app, &view, previous.muted || applied == 0).await?;
        }
        app.state::<MediaService>().update(app, id, |t| {
            t.muted = previous.muted;
            t.volume = applied;
            t.applied_volume = applied;
            t.volume_status = status.into();
            t.document_key = expected.into();
            t.frames = frames;
        });
        app.state::<MediaService>().snapshot(app)
    }
    #[cfg(not(windows))]
    {
        let _ = (app, id, value, expected, pip_session);
        Err("media.unavailable".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_values_above_the_supported_range() {
        for n in [0, 50, 100, 150, 200] {
            assert!(validate_volume(n).is_ok());
        }
        assert!(validate_volume(201).is_err());
    }
    #[test]
    fn document_keys_ignore_context_order_but_detect_navigation() {
        let a = FrameMedia {
            context: "a".into(),
            document: "first".into(),
            ..Default::default()
        };
        let b = FrameMedia {
            context: "b".into(),
            document: "second".into(),
            ..Default::default()
        };
        assert_eq!(
            document_key(&[a.clone(), b.clone()]),
            document_key(&[b.clone(), a.clone()])
        );
        assert_ne!(document_key(&[a, b]), document_key(&[]));
    }
    #[test]
    fn only_a_revalidated_pip_session_can_replace_an_invalidated_aggregate_key() {
        assert!(validate_document("current", "current", false).is_ok());
        assert!(validate_document("", "current", false).is_err());
        assert!(validate_document("old", "current", false).is_err());
        assert!(validate_document("", "", false).is_err());
        assert!(validate_document("", "current", true).is_ok());
        assert!(validate_document("old", "current", true).is_ok());
    }
}
