//! Authenticated, document-bound consumers for the native audio visualizer.
use super::*;
use serde::Serialize;

#[derive(Clone)]
pub(crate) struct Consumer {
    pub id: String,
    pub label: String,
    pub token: String,
    pub document: String,
}
impl Consumer {
    pub fn current(&self, app: &AppHandle) -> bool {
        bridge::valid(app, &self.id, &self.label, &self.token)
            && app.state::<NewTabService>().sessions.lock().is_ok_and(|s| {
                s.get(&self.label)
                    .is_some_and(|s| s.document == self.document)
            })
    }
    pub fn visible(&self, app: &AppHandle) -> bool {
        self.current(app)
            && app.state::<BrowserState>().snapshot_tabs().iter().any(|t| {
                t.id == self.id
                    && t.is_visible
                    && app.get_window(&t.window_id).is_some_and(|w| {
                        w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(true)
                    })
            })
    }
    pub fn deliver(&self, app: &AppHandle, frame: &impl Serialize) {
        if !self.visible(app) {
            return;
        }
        if let (Some(view), Ok(payload)) =
            (app.get_webview(&self.label), serde_json::to_string(frame))
        {
            let _ = view.eval(format!(
                "window.__sparkAudioFrame?.({},{});",
                serde_json::to_string(&self.document).unwrap(),
                payload
            ));
        }
    }
}
