#[cfg(windows)]
pub mod contexts;
#[cfg(target_os = "linux")]
pub mod linux;
pub mod pip;
#[cfg(any(windows, target_os = "linux"))]
pub mod pip_window;
#[cfg(windows)]
pub mod protocol;
#[cfg(target_os = "linux")]
#[path = "protocol_linux.rs"]
pub mod protocol;
mod signal_message;
pub mod signals;
pub mod visualizer;
pub mod volume;
#[cfg(windows)]
pub mod windows;
#[cfg(target_os = "linux")]
pub(crate) use linux as native_audio;
#[cfg(windows)]
pub(crate) use windows as native_audio;

use crate::state::{browser_state::BrowserState, tab_model::webview_label_for_tab};
use crate::sync::Mutex;
use serde::Serialize;
use std::{collections::HashMap, time::Instant};
use tauri::{AppHandle, Manager};

#[derive(Clone, PartialEq, Serialize)]
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
    pub frame_tracking_supported: bool,
}
impl MediaSnapshot {
    fn update(
        &mut self,
        id: &str,
        live: &[crate::state::tab_model::TabInfo],
        change: impl FnOnce(&mut TabMediaState),
    ) -> bool {
        if !live.iter().any(|t| t.id == id) {
            return false;
        }
        let previous_count = self.tabs.len();
        self.tabs
            .retain(|t| live.iter().any(|tab| tab.id == t.tab_id));
        let previous_tab = self.tabs.iter().find(|t| t.tab_id == id).cloned();
        let index = self
            .tabs
            .iter()
            .position(|t| t.tab_id == id)
            .unwrap_or_else(|| {
                self.tabs.push(TabMediaState {
                    tab_id: id.into(),
                    ..Default::default()
                });
                self.tabs.len() - 1
            });
        let t = &mut self.tabs[index];
        change(t);
        t.capabilities = pip::MediaCapabilities {
            mute: t.mute_available,
            volume: !t.document_key.is_empty() && t.volume_status != "unavailable",
            picture_in_picture: t.frames.iter().any(|f| f.videos.iter().any(|v| v.eligible)),
        };
        if previous_count == self.tabs.len() && previous_tab.as_ref() == Some(&self.tabs[index]) {
            return false;
        }
        self.revision += 1;
        true
    }
}
/// Last playback position a tab reported. Kept outside `MediaSnapshot` so it never wakes the shell.
struct ProgressSample {
    position: f64,
    duration: f64,
    playing: bool,
    at: Instant,
}
#[derive(Default)]
pub struct MediaService {
    progress: Mutex<HashMap<String, ProgressSample>>,
    pub operations: tokio::sync::Mutex<()>,
    #[cfg(any(windows, target_os = "linux"))]
    pub(crate) pip_window_shadow: std::sync::atomic::AtomicBool,
    inner: Mutex<MediaSnapshot>,
}
impl MediaService {
    pub fn record_progress(&self, id: &str, position: f64, duration: f64, playing: bool) {
        if let Ok(mut samples) = self.progress.lock() {
            if samples.len() > 64 {
                samples.clear();
            }
            samples.insert(
                id.into(),
                ProgressSample {
                    position,
                    duration,
                    playing,
                    at: Instant::now(),
                },
            );
        }
    }
    /// `(position, duration)` in seconds, advanced by the time elapsed since the sample while playing.
    pub fn progress(&self, id: &str) -> Option<(f64, f64)> {
        let samples = self.progress.lock().ok()?;
        let sample = samples.get(id)?;
        if sample.duration <= 0.0 {
            return None;
        }
        let elapsed = if sample.playing {
            sample.at.elapsed().as_secs_f64()
        } else {
            0.0
        };
        Some((
            (sample.position + elapsed).min(sample.duration),
            sample.duration,
        ))
    }
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
        let mut snapshot = inner.clone();
        snapshot.frame_tracking_supported = cfg!(any(windows, target_os = "linux"));
        Ok(snapshot)
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
        if let Ok(mut inner) = self.inner.lock() {
            inner.frame_tracking_supported = cfg!(any(windows, target_os = "linux"));
            if !inner.update(id, &live, change) {
                return;
            }
            #[cfg(any(windows, target_os = "linux"))]
            let changed = inner.tabs.iter().find(|t| t.tab_id == id).unwrap().clone();
            crate::error_log::ignored(crate::security::emit_shell(
                app,
                "tab-media-state",
                inner.clone(),
            ));
            drop(inner);
            #[cfg(any(windows, target_os = "linux"))]
            pip_window::sync(app, &changed);
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
        #[cfg(any(windows, target_os = "linux"))]
        {
            if let Some(view) = app.get_webview(&webview_label_for_tab(id)) {
                native_audio::set_muted(app, &view, muted || self.tab(id).volume == 0).await?;
            }
            self.update(app, id, |t| {
                t.muted = muted;
                t.mute_available = true;
            });
            self.snapshot(app)
        }
        #[cfg(not(any(windows, target_os = "linux")))]
        {
            let _ = (muted, webview_label_for_tab(id));
            Err("media.unavailable".into())
        }
    }
}
pub fn current(app: &AppHandle, label: &str, token: &str) -> bool {
    app.state::<BrowserState>()
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .is_ok_and(|m| m.get(label).is_some_and(|t| t == token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::tab_model::TabInfo;

    #[test]
    fn unchanged_reports_do_not_publish_and_real_changes_still_increment_revision() {
        let mut snapshot = MediaSnapshot::default();
        let mut live = vec![TabInfo::new_default(
            "a".into(),
            "https://example.test".into(),
            true,
        )];
        assert!(snapshot.update("a", &live, |t| t.audible = true));
        for _ in 0..100 {
            assert!(!snapshot.update("a", &live, |t| t.audible = true));
        }
        assert_eq!(snapshot.revision, 1);
        assert!(snapshot.update("a", &live, |t| t.audible = false));
        assert!(snapshot.update("a", &live, |t| {
            t.document_key = "new-document".into();
            t.volume_status = "applied".into();
        }));
        assert!(snapshot.tabs[0].capabilities.volume);
        assert_eq!(snapshot.revision, 3);
        live.push(TabInfo::new_default(
            "b".into(),
            "https://example.test/b".into(),
            false,
        ));
        assert!(snapshot.update("b", &live, |_| {}));
        live.remove(0);
        assert!(
            snapshot.update("b", &live, |_| {}),
            "removing a closed tab changes the snapshot"
        );
        assert_eq!(snapshot.tabs.len(), 1);
        assert!(!snapshot.update("a", &live, |_| panic!("closed tab must be ignored")));
    }
}
