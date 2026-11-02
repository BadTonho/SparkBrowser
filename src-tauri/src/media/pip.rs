#[cfg(windows)]
use super::MediaService;
use super::MediaSnapshot;
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use serde_json::json;
use tauri::AppHandle;
#[cfg(windows)]
use tauri::{Emitter, Manager};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MediaVideoCandidate {
    pub id: String,
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub eligible: bool,
}
#[derive(Clone, Default, Serialize)]
pub struct MediaCapabilities {
    pub mute: bool,
    pub volume: bool,
    pub picture_in_picture: bool,
}
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Presentation {
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub labels: std::collections::HashMap<String, String>,
}
pub fn player_script() -> String {
    include_str!("pip_player.js")
        .replace(
            "__PIP_PLAYER_STYLE__",
            &serde_json::to_string(include_str!("player/style.css")).expect("static player CSS"),
        )
        .replace("__PIP_ICONS__", include_str!("player/icons.js"))
}
#[cfg(windows)]
async fn exit_known(app: &AppHandle) -> Result<(), String> {
    if let Some((session, tab, token, context, document)) = super::pip_window::owner() {
        let _ = super::protocol::evaluate(
            app,
            &tab,
            &token,
            &context,
            json!({"kind":"pip-exit","document":document}),
            false,
        )
        .await;
        super::pip_window::discard(app, &session);
    }
    for tab in app.state::<MediaService>().snapshot(app)?.tabs {
        for frame in tab.frames.iter().filter(|f| f.picture_in_picture) {
            let (token, contexts) = super::protocol::contexts(app, &tab.tab_id).await?;
            if !contexts.contains(&frame.context) {
                continue;
            }
            let value = super::protocol::evaluate(
                app,
                &tab.tab_id,
                &token,
                &frame.context,
                json!({"kind":"pip-exit","document":frame.document}),
                false,
            )
            .await?;
            if let Some(code) = value["error"].as_str() {
                return Err(code.into());
            }
        }
    }
    Ok(())
}
pub async fn enter(
    app: &AppHandle,
    id: &str,
    context: &str,
    document: &str,
    video: &str,
    presentation: Option<Presentation>,
) -> Result<MediaSnapshot, String> {
    #[cfg(windows)]
    {
        let snapshot = super::volume::refresh(app, id).await?;
        let tab = snapshot
            .tabs
            .iter()
            .find(|t| t.tab_id == id)
            .ok_or("media.tab_missing")?;
        let frame = tab
            .frames
            .iter()
            .find(|f| f.context == context && f.document == document)
            .ok_or("media.stale")?;
        if !frame.videos.iter().any(|v| v.id == video && v.eligible) {
            return Err("media.pip_unavailable".into());
        }
        exit_known(app).await?;
        let (token, contexts) = super::protocol::contexts(app, id).await?;
        if !contexts.iter().any(|c| c == context) {
            return Err("media.stale".into());
        }
        let session = super::pip_window::prepare(id, &token, context, document, &tab.document_key);
        let result = super::protocol::evaluate(
            app,
            id,
            &token,
            context,
            json!({"kind":"pip-enter","document":document,"video":video,"session":session,"presentation":presentation.unwrap_or_default()}),
            true,
        )
        .await;
        if result.as_ref().is_err() || result.as_ref().is_ok_and(|v| v["custom"] != true) {
            super::pip_window::discard(app, &session);
        }
        let result = result?;
        if result["custom"] == false && result["ok"] == true {
            let _ = app.emit_to("main", "media-pip-fallback", ());
        }
        if let Some(code) = result["error"].as_str() {
            return Err(code.into());
        }
        super::volume::refresh(app, id).await
    }
    #[cfg(not(windows))]
    {
        let _ = (app, id, context, document, video, presentation);
        Err("media.unavailable".into())
    }
}
pub async fn exit(app: &AppHandle) -> Result<MediaSnapshot, String> {
    #[cfg(windows)]
    {
        exit_known(app).await?;
        app.state::<MediaService>().snapshot(app)
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("media.unavailable".into())
    }
}
