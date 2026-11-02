use super::{
    http::now,
    model::{UpdateChannel, UpdateState},
};
use crate::{database::updates_repo, state::browser_state::BrowserState};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{AppHandle, Emitter, Manager};

pub struct Inner {
    pub snapshot: UpdateState,
    pub generation: u64,
    #[cfg(windows)]
    pub candidate: Option<tauri_plugin_updater::Update>,
    #[cfg(windows)]
    pub bytes: Option<std::sync::Arc<Vec<u8>>>,
}
impl Inner {
    pub fn invalidate(&mut self) {
        self.generation += 1;
        self.snapshot.update = None;
        self.snapshot.received = 0;
        self.snapshot.total = None;
        self.snapshot.error = None;
        #[cfg(windows)]
        {
            self.candidate = None;
            self.bytes = None;
        }
    }
}
pub struct UpdateService {
    pub inner: Mutex<Inner>,
    pub operation: tokio::sync::Mutex<()>,
    pub startup_checked: AtomicBool,
    pub changed: tokio::sync::Notify,
}
impl UpdateService {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let version = app.package_info().version.to_string();
        let browser = app.state::<BrowserState>();
        let db = browser.db.lock().map_err(|e| e.to_string())?;
        let (channel, last_checked) = updates_repo::load(
            db.as_ref().ok_or("Database is not ready")?,
            UpdateChannel::initial(&version),
        )?;
        let supported = cfg!(all(windows, target_arch = "x86_64", not(debug_assertions)))
            && app.config().identifier == "org.veloce.browser";
        Ok(Self {
            inner: Mutex::new(Inner {
                snapshot: UpdateState {
                    revision: 0,
                    supported,
                    message: if supported {
                        String::new()
                    } else {
                        "Updates require the installed Windows x64 release. Development, preview and test builds do not update.".into()
                    },
                    current_version: version,
                    channel,
                    last_checked,
                    status: if supported { "idle" } else { "unavailable" }.into(),
                    update: None,
                    received: 0,
                    total: None,
                    error: None,
                    retry_after: None,
                },
                generation: 0,
                #[cfg(windows)]
                candidate: None,
                #[cfg(windows)]
                bytes: None,
            }),
            operation: tokio::sync::Mutex::new(()),
            startup_checked: AtomicBool::new(false),
            changed: tokio::sync::Notify::new(),
        })
    }
    pub fn snapshot(&self) -> Result<UpdateState, String> {
        Ok(self
            .inner
            .lock()
            .map_err(|e| e.to_string())?
            .snapshot
            .clone())
    }
    /// Off the UI thread: cancel a background request, then wait for its operation to settle.
    pub fn settle_for_exit(&self) -> Result<(), String> {
        {
            let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
            if matches!(inner.snapshot.status.as_str(), "checking" | "downloading") {
                inner.invalidate();
                inner.snapshot.status = "idle".into();
                self.changed.notify_waiters();
            }
        }
        let _operation = self.operation.blocking_lock();
        Ok(())
    }
    pub fn publish(&self, app: &AppHandle) -> Result<UpdateState, String> {
        let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
        inner.snapshot.revision += 1;
        let snapshot = inner.snapshot.clone();
        drop(inner);
        let _ = app.emit_to(
            tauri::EventTarget::webview("main"),
            "update-state",
            &snapshot,
        );
        Ok(snapshot)
    }
    pub fn set_channel(
        &self,
        app: &AppHandle,
        channel: UpdateChannel,
    ) -> Result<UpdateState, String> {
        let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
        if inner.snapshot.status == "installing" {
            return Err("Installation is already starting".into());
        }
        let browser = app.state::<BrowserState>();
        let db = browser.db.lock().map_err(|e| e.to_string())?;
        updates_repo::set_channel(db.as_ref().ok_or("Database is not ready")?, channel)?;
        inner.invalidate();
        inner.snapshot.channel = channel;
        inner.snapshot.status = if inner.snapshot.supported {
            "idle"
        } else {
            "unavailable"
        }
        .into();
        drop(inner);
        self.changed.notify_waiters();
        self.publish(app)
    }
    pub async fn check(&self, app: &AppHandle, startup: bool) -> Result<UpdateState, String> {
        if startup && self.startup_checked.swap(true, Ordering::Relaxed) {
            return self.snapshot();
        }
        if !self.snapshot()?.supported {
            return self.snapshot();
        }
        let _operation = self
            .operation
            .try_lock()
            .map_err(|_| "An update operation is already running")?;
        let changed = self.changed.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        let (generation, channel, version) = {
            let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
            if inner
                .snapshot
                .retry_after
                .is_some_and(|deadline| deadline > now())
            {
                return Ok(inner.snapshot.clone());
            }
            inner.invalidate();
            inner.snapshot.status = "checking".into();
            (
                inner.generation,
                inner.snapshot.channel,
                inner.snapshot.current_version.clone(),
            )
        };
        self.publish(app)?;
        #[cfg(windows)]
        let result = tokio::select! {
            result = super::windows::find(app, &version, channel) => result,
            _ = changed => return self.snapshot(),
        };
        #[cfg(not(windows))]
        let result: Result<Option<()>, super::http::CheckError> = {
            let _ = (version, channel);
            Err(super::http::CheckError::plain(
                "Updates are unavailable on this platform",
            ))
        };
        {
            let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
            if generation != inner.generation {
                return Ok(inner.snapshot.clone());
            }
            let timestamp = now();
            let browser = app.state::<BrowserState>();
            let db = browser.db.lock().map_err(|e| e.to_string())?;
            let saved =
                updates_repo::record_check(db.as_ref().ok_or("Database is not ready")?, timestamp);
            inner.snapshot.last_checked = Some(timestamp);
            match result {
                #[cfg(windows)]
                Ok(Some((candidate, info))) => {
                    inner.candidate = Some(candidate);
                    inner.snapshot.update = Some(info);
                    inner.snapshot.status = "available".into();
                    inner.snapshot.retry_after = None;
                }
                Ok(None) => {
                    inner.snapshot.status = "up_to_date".into();
                    inner.snapshot.retry_after = None;
                }
                Err(error) => {
                    inner.snapshot.status = "error".into();
                    inner.snapshot.error = Some(error.message);
                    inner.snapshot.retry_after = error.retry_after;
                }
                #[cfg(not(windows))]
                Ok(Some(())) => unreachable!(),
            }
            if let Err(error) = saved {
                inner.snapshot.status = "error".into();
                inner.snapshot.error = Some(format!("Could not save update preferences: {error}"));
            }
        }
        self.publish(app)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn channel_changes_invalidate_in_flight_results_and_cached_bytes() {
        let mut inner = Inner {
            snapshot: UpdateState {
                revision: 1,
                supported: true,
                message: String::new(),
                current_version: "1.0.0".into(),
                channel: UpdateChannel::Stable,
                last_checked: None,
                status: "downloading".into(),
                update: None,
                received: 10,
                total: Some(20),
                error: None,
                retry_after: None,
            },
            generation: 1,
            #[cfg(windows)]
            candidate: None,
            #[cfg(windows)]
            bytes: Some(std::sync::Arc::new(vec![1, 2])),
        };
        let old_generation = inner.generation;
        inner.invalidate();
        assert_ne!(old_generation, inner.generation);
        assert_eq!(inner.snapshot.received, 0);
        #[cfg(windows)]
        assert!(inner.bytes.is_none());
    }
}
