use super::transfer::{download_with_control, validate_download_url};
use crate::{database::downloads_repo::record_download, state::browser_state::BrowserState};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize)]
pub struct DownloadTask {
    pub id: String,
    pub revision: u64,
    pub file_name: String,
    pub url: String,
    pub status: String,
    pub received: u64,
    pub total: Option<u64>,
    pub error: Option<String>,
    pub cancellable: bool,
    pub save_path: Option<String>,
}
impl DownloadTask {
    pub fn pending(&self) -> bool {
        matches!(
            self.status.as_str(),
            "Queued" | "Downloading" | "Cancelling"
        )
    }
}
struct TaskRecord {
    task: DownloadTask,
    tab_id: String,
    cancel: Arc<AtomicBool>,
    cancelled: Arc<tokio::sync::Notify>,
}
pub struct DownloadManager {
    tasks: Mutex<HashMap<String, TaskRecord>>,
    counter: AtomicU64,
    pub slots: Arc<tokio::sync::Semaphore>,
}
impl Default for DownloadManager {
    fn default() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            counter: AtomicU64::new(1),
            slots: Arc::new(tokio::sync::Semaphore::new(2)),
        }
    }
}
impl DownloadManager {
    /// Runs off the UI thread during normal exit so partial manual files can be cleaned up.
    pub fn shutdown(&self) {
        if let Ok(tasks) = self.tasks.lock() {
            for record in tasks
                .values()
                .filter(|record| record.task.cancellable && record.task.pending())
            {
                record.cancel.store(true, Ordering::Relaxed);
                record.cancelled.notify_one();
            }
        }
        let started = Instant::now();
        while self.tasks.lock().is_ok_and(|tasks| {
            tasks
                .values()
                .any(|record| record.task.cancellable && record.task.pending())
        }) {
            if started.elapsed() > Duration::from_secs(20) {
                break;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
    }
    pub fn list(&self) -> Result<Vec<DownloadTask>, String> {
        let mut tasks: Vec<_> = self
            .tasks
            .lock()
            .map_err(|e| e.to_string())?
            .values()
            .map(|r| r.task.clone())
            .collect();
        tasks.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(tasks)
    }
    pub fn register(
        &self,
        tab_id: String,
        url: String,
        file_name: String,
        native: bool,
    ) -> Result<DownloadTask, String> {
        let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        if tasks.values().filter(|r| r.task.pending()).count() >= 64 {
            return Err("Download queue is full (64 tasks)".into());
        }
        if tasks.len() >= 164 {
            tasks.retain(|_, r| r.task.pending());
        }
        let task = DownloadTask {
            id: format!(
                "download-{:010}",
                self.counter.fetch_add(1, Ordering::Relaxed)
            ),
            revision: 1,
            url,
            file_name,
            status: if native { "Downloading" } else { "Queued" }.into(),
            received: 0,
            total: None,
            error: None,
            cancellable: !native,
            save_path: None,
        };
        tasks.insert(
            task.id.clone(),
            TaskRecord {
                task: task.clone(),
                tab_id,
                cancel: Arc::new(AtomicBool::new(false)),
                cancelled: Arc::new(tokio::sync::Notify::new()),
            },
        );
        Ok(task)
    }
    pub fn update(&self, app: &AppHandle, id: &str, update: impl FnOnce(&mut DownloadTask)) {
        let payload = self.tasks.lock().ok().and_then(|mut tasks| {
            tasks.get_mut(id).map(|r| {
                update(&mut r.task);
                r.task.revision += 1;
                r.task.clone()
            })
        });
        if let Some(task) = payload {
            let _ = app.emit("download-task-updated", task);
        }
    }
    pub fn cancel(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        let payload = {
            let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
            let record = tasks.get_mut(id).ok_or("Download task was not found")?;
            if !record.task.pending() {
                return Ok(());
            }
            if !record.task.cancellable {
                return Err("This native download cannot be cancelled from the manager".into());
            }
            record.cancel.store(true, Ordering::Relaxed);
            record.cancelled.notify_one();
            record.task.status = "Cancelling".into();
            record.task.revision += 1;
            record.task.clone()
        };
        let _ = app.emit("download-task-updated", payload);
        Ok(())
    }
    pub fn refresh_protection(&self, app: &AppHandle, tab_id: &str) {
        let downloading = self
            .tasks
            .lock()
            .map(|tasks| {
                tasks
                    .values()
                    .any(|r| r.tab_id == tab_id && r.task.pending())
            })
            .unwrap_or(true);
        let state = app.state::<BrowserState>();
        if let Ok(mut tabs) = state.tabs.lock() {
            if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == tab_id) {
                tab.activity.downloading = downloading;
            }
        };
    }
    pub fn enqueue(&self, app: &AppHandle, raw_url: &str) -> Result<DownloadTask, String> {
        let url = validate_download_url(raw_url)?;
        let state = app.state::<BrowserState>();
        let shield = *state.shield_enabled.lock().map_err(|e| e.to_string())?;
        if crate::privacy::adblock_engine::should_block_url(url.as_str(), shield) {
            return Err("Download blocked by the privacy shield".into());
        }
        let destination = app.path().download_dir().map_err(|e| e.to_string())?;
        let tab_id = state
            .active_tab_id
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        let task = self.register(
            tab_id.clone(),
            url.to_string(),
            super::transfer::file_name(&url),
            false,
        )?;
        let (cancel, cancelled) = {
            let tasks = self.tasks.lock().map_err(|e| e.to_string())?;
            let record = tasks.get(&task.id).ok_or("Task was not found")?;
            (record.cancel.clone(), record.cancelled.clone())
        };
        self.refresh_protection(app, &tab_id);
        let app = app.clone();
        let id = task.id.clone();
        let slots = self.slots.clone();
        let original_url = url.to_string();
        tauri::async_runtime::spawn(async move {
            let permit = tokio::select! {
                biased;
                _ = cancelled.notified() => {
                    let manager = app.state::<DownloadManager>();
                    manager.update(&app, &id, |task| task.status = "Cancelled".into());
                    manager.refresh_protection(&app, &tab_id);
                    return;
                }
                result = slots.acquire_owned() => match result { Ok(permit) => permit, Err(_) => return },
            };
            let transfer_app = app.clone();
            let transfer_id = id.clone();
            let transfer_cancel = cancel.clone();
            let result = tauri::async_runtime::spawn_blocking(move || {
                let manager = transfer_app.state::<DownloadManager>();
                manager.update(&transfer_app, &transfer_id, |task| {
                    task.status = if transfer_cancel.load(Ordering::Relaxed) {
                        "Cancelling"
                    } else {
                        "Downloading"
                    }
                    .into()
                });
                let mut last = Instant::now() - Duration::from_secs(1);
                download_with_control(
                    &url,
                    &destination,
                    shield,
                    &transfer_cancel,
                    |received, total| {
                        if last.elapsed() >= Duration::from_millis(100) || total == Some(received) {
                            manager.update(&transfer_app, &transfer_id, |task| {
                                task.received = received;
                                task.total = total;
                            });
                            last = Instant::now();
                        }
                    },
                )
            })
            .await
            .map_err(|_| "Download worker failed".to_string())
            .and_then(|result| result);
            let manager = app.state::<DownloadManager>();
            match result {
                Ok(saved) => {
                    let path = saved.path.to_string_lossy().to_string();
                    manager.update(&app, &id, |task| {
                        task.status = "Completed".into();
                        task.received = saved.bytes;
                        task.total = Some(saved.bytes);
                        task.file_name = saved.file_name.clone();
                        task.save_path = Some(path.clone());
                        task.error = None;
                    });
                    let state = app.state::<BrowserState>();
                    let recorded = state.db.lock().map_err(|e| e.to_string()).and_then(|db| {
                        let conn = db.as_ref().ok_or("Download database is not ready")?;
                        record_download(conn, &saved.file_name, &original_url, &path, "Completed")
                    });
                    match recorded {
                        Ok(items) => {
                            let _ = app.emit("downloads-updated", items);
                        }
                        Err(_) => {
                            let _ = app.emit(
                                "browser-error",
                                "File saved, but its download record could not be persisted",
                            );
                        }
                    }
                }
                Err(error) => manager.update(&app, &id, |task| {
                    task.status = if cancel.load(Ordering::Relaxed) {
                        "Cancelled"
                    } else {
                        "Failed"
                    }
                    .into();
                    task.error = if cancel.load(Ordering::Relaxed) {
                        None
                    } else {
                        Some(error)
                    };
                }),
            }
            manager.refresh_protection(&app, &tab_id);
            drop(permit);
        });
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backend_bounds_the_queue_and_shared_transfer_slots() {
        let manager = DownloadManager::default();
        let first = manager.slots.clone().try_acquire_owned().unwrap();
        let second = manager.slots.clone().try_acquire_owned().unwrap();
        assert!(manager.slots.clone().try_acquire_owned().is_err());
        drop(first);
        assert!(manager.slots.clone().try_acquire_owned().is_ok());
        drop(second);
        for _ in 0..64 {
            manager
                .register(
                    "tab-1".into(),
                    "https://example.test".into(),
                    "file".into(),
                    false,
                )
                .unwrap();
        }
        assert!(manager
            .register(
                "tab-1".into(),
                "https://example.test".into(),
                "file".into(),
                false
            )
            .is_err());
        assert_eq!(manager.list().unwrap().len(), 64);
    }
}
