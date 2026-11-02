//! Runtime events report only media state; they cannot invoke browser commands.
use super::{volume::FrameMedia, MediaService};
use crate::state::browser_state::BrowserState;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

pub fn receive(app: &AppHandle, id: &str, context: &str, value: &Value) {
    let Some(document) = value["document"].as_str().filter(|v| v.len() <= 128) else {
        return;
    };
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
        f.playing = value["playing"].as_bool().unwrap_or(false);
        f.picture_in_picture = value["picture_in_picture"].as_bool().unwrap_or(false);
        f.pip_video = value["pip_video"]
            .as_str()
            .filter(|v| v.len() <= 128)
            .map(str::to_string);
    });
    sync_activity(app, id);
}
pub fn remove(app: &AppHandle, id: &str, context: Option<&str>) {
    if !app
        .state::<BrowserState>()
        .snapshot_tabs()
        .iter()
        .any(|t| t.id == id)
    {
        if let Ok(snapshot) = app.state::<MediaService>().snapshot(app) {
            let _ = app.emit_to("main", "tab-media-state", snapshot);
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
    let Ok(mut tabs) = browser.tabs.lock() else {
        return;
    };
    let Some(tab) = tabs.iter_mut().find(|t| t.id == id) else {
        return;
    };
    let before = tab.activity.clone();
    tab.activity.native_audio = media.audible;
    tab.activity.frame_media = media.frames.iter().any(|f| f.playing);
    tab.activity.picture_in_picture = media.frames.iter().any(|f| f.picture_in_picture);
    tab.activity.refresh_media();
    if before == tab.activity {
        return;
    }
    tab.navigation.revision += 1;
    let payload = serde_json::json!({"id":tab.id,"url":tab.url,"title":tab.title,"navigation":tab.navigation,"activity":tab.activity});
    drop(tabs);
    let _ = app.emit_to("main", "tab-navigation-updated", payload);
}
