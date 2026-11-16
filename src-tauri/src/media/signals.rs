//! Runtime events report only media state; they cannot invoke browser commands.
use super::signal_message::Signal;
use super::{volume::FrameMedia, MediaService};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager};

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
    if let (Some(position), Some(duration)) = (value.position, value.duration) {
        app.state::<MediaService>()
            .record_progress(id, position, duration, value.playing);
    }
    sync_activity(app, id);
}
pub fn remove(app: &AppHandle, id: &str, context: Option<&str>) {
    #[cfg(any(windows, target_os = "linux"))]
    super::pip_window::source_removed(app, id, context);
    if !app
        .state::<BrowserState>()
        .snapshot_tabs()
        .iter()
        .any(|t| t.id == id)
    {
        if let Ok(snapshot) = app.state::<MediaService>().snapshot(app) {
            crate::error_log::ignored(crate::security::emit_shell(
                app,
                "tab-media-state",
                snapshot,
            ));
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
        crate::error_log::ignored(crate::security::emit_shell_for_tab(
            app,
            id,
            "tab-navigation-updated",
            payload,
        ));
    }
}
