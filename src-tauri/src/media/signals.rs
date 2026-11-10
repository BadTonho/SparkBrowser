//! Runtime events report only media state; they cannot invoke browser commands.
use super::signal_message::Signal;
use super::{volume::FrameMedia, MediaService};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager};

#[cfg_attr(not(windows), allow(dead_code))]
pub(super) fn receive(app: &AppHandle, id: &str, context: &str, value: &Signal) {
    let document = value.document.as_str();
    app.state::<MediaService>().update(app, id, |t| {
        let index = t
            .frames
            .iter()
            .position(|f| f.context == context)
            .unwrap_or_else(|| {
                t.frames.push(FrameMedia {
                    context: context.into(),
                    document: document.into(),
                    status: "unavailable".into(),
                    applied_volume: 100,
                    ..Default::default()
                });
                t.frames.len() - 1
            });
        let f = &mut t.frames[index];
        if f.document != document {
            return;
        }
        f.playing = value.playing;
        f.picture_in_picture = value.picture_in_picture;
        f.pip_video = value.pip_video.clone();
        f.pip_mode = value.pip_mode.clone();
        f.pip_session = value.pip_session.clone();
    });
    sync_activity(app, id);
}
pub fn remove(app: &AppHandle, id: &str, context: Option<&str>) {
    #[cfg(windows)]
    super::pip_window::source_removed(app, id, context);
    if !app
        .state::<BrowserState>()
        .snapshot_tabs()
        .iter()
        .any(|t| t.id == id)
    {
        if let Ok(snapshot) = app.state::<MediaService>().snapshot(app) {
            let _ = crate::security::emit_shell(app, "tab-media-state", snapshot);
        }
        return;
    }
    app.state::<MediaService>().update(app, id, |t| {
        if let Some(context) = context {
            t.frames.retain(|f| f.context != context);
        } else {
            t.frames.clear();
            t.audible = false;
        }
        t.document_key.clear();
    });
    sync_activity(app, id);
}
pub fn sync_activity(app: &AppHandle, id: &str) {
    let media = app.state::<MediaService>().tab(id);
    let browser = app.state::<BrowserState>();
    if let Ok(Some(payload)) = browser.observe_document(id, |tab| {
        tab.activity.native_audio = media.audible;
        tab.activity.frame_media = media.frames.iter().any(|f| f.playing);
        tab.activity.picture_in_picture = media.frames.iter().any(|f| f.picture_in_picture);
        tab.activity.refresh_media();
    }) {
        let _ = crate::security::emit_shell(app, "tab-navigation-updated", payload);
    }
}
