#[cfg(windows)]
pub mod contexts;
pub mod pip;
#[cfg(windows)]
pub mod protocol;
pub mod signals;
pub mod volume;
#[cfg(windows)]
pub mod windows;

use crate::state::{browser_state::BrowserState, tab_model::webview_label_for_tab};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Serialize)]
pub struct TabMediaState {
    pub tab_id: String,
    pub muted: bool,
    pub audible: bool,
    pub mute_available: bool,
    pub volume: u16,
    pub applied_volume: u16,
    pub volume_status: String,
    pub document_key: String,
    pub frames: Vec<volume::FrameMedia>,
    pub capabilities: pip::MediaCapabilities,
}
impl Default for TabMediaState {
    fn default() -> Self {
        Self {
            tab_id: String::new(),
            muted: false,
            audible: false,
            mute_available: false,
            volume: 100,
            applied_volume: 100,
            volume_status: "unavailable".into(),
            document_key: String::new(),
            frames: Vec::new(),
            capabilities: Default::default(),
        }
    }
}
#[derive(Clone, Default, Serialize)]
pub struct MediaSnapshot {
    pub revision: u64,
    pub tabs: Vec<TabMediaState>,
}
#[derive(Default)]
pub struct MediaService {
    pub operations: tokio::sync::Mutex<()>,
    inner: Mutex<MediaSnapshot>,
}
impl MediaService {
    pub fn snapshot(&self, app: &AppHandle) -> Result<MediaSnapshot, String> {
        let ids: Vec<_> = app
            .state::<BrowserState>()
            .snapshot_tabs()
            .into_iter()
            .map(|t| t.id)
            .collect();
        let mut inner = self.inner.lock().map_err(|_| "media.error")?;
        if inner.tabs.iter().any(|t| !ids.contains(&t.tab_id)) {
            inner.tabs.retain(|t| ids.contains(&t.tab_id));
            inner.revision += 1;
        }
        Ok(inner.clone())
    }
    pub fn tab(&self, id: &str) -> TabMediaState {
        self.inner
            .lock()
            .ok()
            .and_then(|s| s.tabs.iter().find(|t| t.tab_id == id).cloned())
            .unwrap_or_else(|| TabMediaState {
                tab_id: id.into(),
                ..Default::default()
            })
    }
    pub fn update(&self, app: &AppHandle, id: &str, change: impl FnOnce(&mut TabMediaState)) {
        let live = app.state::<BrowserState>().snapshot_tabs();
        if !live.iter().any(|t| t.id == id) {
            return;
        }
        if let Ok(mut inner) = self.inner.lock() {
            inner
                .tabs
                .retain(|t| live.iter().any(|tab| tab.id == t.tab_id));
            let index = inner
                .tabs
                .iter()
                .position(|t| t.tab_id == id)
                .unwrap_or_else(|| {
                    inner.tabs.push(TabMediaState {
                        tab_id: id.into(),
                        ..Default::default()
                    });
                    inner.tabs.len() - 1
                });
            change(&mut inner.tabs[index]);
            let t = &mut inner.tabs[index];
            t.capabilities = pip::MediaCapabilities {
                mute: t.mute_available,
                volume: !t.document_key.is_empty() && t.volume_status != "unavailable",
                picture_in_picture: t.frames.iter().any(|f| f.videos.iter().any(|v| v.eligible)),
            };
            inner.revision += 1;
            let _ = app.emit_to("main", "tab-media-state", inner.clone());
        }
    }
    pub async fn mute(
        &self,
        app: &AppHandle,
        id: &str,
        muted: bool,
    ) -> Result<MediaSnapshot, String> {
        if !app
            .state::<BrowserState>()
            .snapshot_tabs()
            .iter()
            .any(|t| t.id == id)
        {
            return Err("media.tab_missing".into());
        }
        #[cfg(windows)]
        {
            if let Some(view) = app.get_webview(&webview_label_for_tab(id)) {
                windows::set_muted(app, &view, muted || self.tab(id).volume == 0).await?;
            }
            self.update(app, id, |t| {
                t.muted = muted;
                t.mute_available = true;
            });
            self.snapshot(app)
        }
        #[cfg(not(windows))]
        {
            let _ = (muted, webview_label_for_tab(id));
            Err("media.unavailable".into())
        }
    }
}
pub fn current(app: &AppHandle, label: &str, token: &str) -> bool {
    app.state::<BrowserState>()
        .keyboard_bridge_tokens
        .lock()
        .is_ok_and(|m| m.get(label).is_some_and(|t| t == token))
}
