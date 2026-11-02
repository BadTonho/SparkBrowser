mod actions;
mod worker;
pub use super::model::DownloadTask;
use super::{files, model::DownloadRecord};
use crate::{database::download_jobs_repo as repo, state::browser_state::BrowserState};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};

pub struct Control {
    pub requested: AtomicU8,
    pub changed: tokio::sync::Notify,
}
impl Default for Control {
    fn default() -> Self {
        Self {
            requested: AtomicU8::new(0),
            changed: Default::default(),
        }
    }
}
struct Live {
    record: DownloadRecord,
    control: Arc<Control>,
    worker: bool,
    generation: u64,
    native_available: bool,
    native_resume: bool,
}
pub struct DownloadManager {
    tasks: Mutex<HashMap<String, Live>>,
    operations: tauri::async_runtime::Mutex<()>,
    pub slots: Arc<tokio::sync::Semaphore>,
}
impl Default for DownloadManager {
    fn default() -> Self {
        Self {
            tasks: Default::default(),
            operations: Default::default(),
            slots: Arc::new(tokio::sync::Semaphore::new(2)),
        }
    }
}
impl DownloadManager {
    fn persist(app: &AppHandle, record: &DownloadRecord) -> Result<(), String> {
        let state = app.state::<BrowserState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        repo::save(db.as_ref().ok_or("downloads.database_unavailable")?, record)
    }
    pub fn initialize(&self, app: &AppHandle) -> Result<(), String> {
        let records = {
            let state = app.state::<BrowserState>();
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let conn = db.as_ref().ok_or("downloads.database_unavailable")?;
            conn.execute("DELETE FROM download_jobs WHERE json_extract(document,'$.task.status') IN('Completed','Cancelled') AND id NOT IN(SELECT id FROM download_jobs ORDER BY rowid DESC LIMIT 100)",[]).map_err(|e|e.to_string())?;
            repo::load(conn)?
        };
        for mut record in records {
            if record.task.status == "Finalizing" {
                match files::publish(&record) {
                    Ok(()) => {
                        record.published = true;
                        record.task.status = "Completed".into();
                        self.commit_completed(app, &record)?;
                    }
                    Err(e) => {
                        record.task.status = "RestartRequired".into();
                        record.task.error = Some(e);
                    }
                }
            } else if record.task.pending() {
                record.task.status =
                    if record.task.status == "Verifying" && !record.partial().exists() {
                        "Completed"
                    } else {
                        "Interrupted"
                    }
                    .into();
            }
            if record.task.source == "native" && !record.body_completed && record.task.unfinished()
            {
                record.task.status = "Interrupted".into();
                record.task.error = Some("downloads.native_unavailable".into());
            }
            record.task.revision += 1;
            record.refresh_capabilities(false);
            Self::persist(app, &record)?;
            self.tasks.lock().map_err(|e| e.to_string())?.insert(
                record.task.id.clone(),
                Live {
                    record,
                    control: Arc::new(Control::default()),
                    worker: false,
                    generation: 0,
                    native_available: false,
                    native_resume: false,
                },
            );
        }
        Ok(())
    }
    pub fn list(&self) -> Result<Vec<DownloadTask>, String> {
        let mut out: Vec<_> = self
            .tasks
            .lock()
            .map_err(|e| e.to_string())?
            .values()
            .map(|v| v.record.task.clone())
            .collect();
        out.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(out)
    }
    pub fn record(&self, id: &str) -> Result<DownloadRecord, String> {
        self.tasks
            .lock()
            .map_err(|e| e.to_string())?
            .get(id)
            .map(|v| v.record.clone())
            .ok_or("downloads.not_found".into())
    }
    pub fn register(
        &self,
        app: &AppHandle,
        tab_id: String,
        url: String,
        name: String,
        native: bool,
        expected: Option<String>,
    ) -> Result<DownloadTask, String> {
        if app
            .try_state::<crate::shutdown::ShutdownService>()
            .is_some_and(|s| s.blocked())
        {
            return Err("shutdown.busy".into());
        }
        let expected = super::model::checksum(expected)?;
        let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        if tasks
            .values()
            .filter(|v| v.record.task.unfinished())
            .count()
            >= 64
        {
            return Err("downloads.queue_full".into());
        }
        if tasks.len() >= 164 {
            tasks.retain(|_, v| v.record.task.unfinished());
        }
        let id = format!(
            "download-{}",
            crate::window::webview_manager::create_internal_bridge_token()
        );
        let directory = files::directory(app)?;
        let page = app
            .state::<BrowserState>()
            .snapshot_tabs()
            .into_iter()
            .find(|t| t.id == tab_id)
            .map(|t| t.url);
        let task = DownloadTask {
            id: id.clone(),
            revision: 1,
            file_name: name.clone(),
            url,
            source: if native { "native" } else { "url" }.into(),
            source_page_url: page,
            status: if native { "Downloading" } else { "Paused" }.into(),
            received: 0,
            total: None,
            error: None,
            cancellable: true,
            can_pause: false,
            can_resume: false,
            can_restart: false,
            can_verify: false,
            save_path: Some(
                directory
                    .join(format!("{id}-{name}"))
                    .to_string_lossy()
                    .into(),
            ),
            expected_sha256: expected,
            actual_sha256: None,
            verification_received: 0,
        };
        let mut record = DownloadRecord {
            task,
            tab_id,
            directory,
            confirmed: 0,
            prefix_sha256: format!("{:x}", sha2::Sha256::default().finalize()),
            final_url: None,
            etag: None,
            body_completed: false,
            published: false,
        };
        record.refresh_capabilities(native);
        Self::persist(app, &record)?;
        let task = record.task.clone();
        tasks.insert(
            id,
            Live {
                record,
                control: Arc::new(Control::default()),
                worker: false,
                generation: 0,
                native_available: native,
                native_resume: false,
            },
        );
        Ok(task)
    }
    pub fn edit(
        &self,
        app: &AppHandle,
        id: &str,
        durable: bool,
        f: impl FnOnce(&mut DownloadRecord),
    ) -> Result<DownloadTask, String> {
        let task = {
            let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
            let live = tasks.get_mut(id).ok_or("downloads.not_found")?;
            let before = live.record.clone();
            f(&mut live.record);
            live.record.task.revision += 1;
            live.record.refresh_capabilities(live.native_available);
            if live.record.task.source == "native" {
                live.record.task.can_resume &= live.native_resume;
            }
            if durable {
                if let Err(e) = Self::persist(app, &live.record) {
                    live.record = before;
                    return Err(e);
                }
            }
            live.record.task.clone()
        };
        let _ = app.emit_to("main", "download-task-updated", &task);
        self.refresh_protection(app, &self.record(id)?.tab_id);
        Ok(task)
    }
    pub fn update(&self, app: &AppHandle, id: &str, f: impl FnOnce(&mut DownloadTask)) {
        let _ = self.edit(app, id, false, |r| f(&mut r.task));
    }
    pub fn update_saved(
        &self,
        app: &AppHandle,
        id: &str,
        f: impl FnOnce(&mut DownloadTask),
    ) -> Result<DownloadTask, String> {
        self.edit(app, id, true, |r| f(&mut r.task))
    }
    pub fn native_capability(&self, app: &AppHandle, id: &str, available: bool, resume: bool) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(v) = tasks.get_mut(id) {
                v.native_available = available;
                v.native_resume = resume;
            }
        }
        self.update(app, id, |_| {});
    }
    pub fn body_completed(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        self.edit(app, id, true, |r| {
            r.body_completed = true;
            r.task.status = "Paused".into();
        })?;
        self.start(app, id, true)
    }
    pub fn refresh_protection(&self, app: &AppHandle, tab_id: &str) {
        let protected = self
            .tasks
            .lock()
            .map(|tasks| {
                tasks.values().any(|v| {
                    v.record.tab_id == tab_id
                        && (v.record.task.pending()
                            || (v.record.task.source == "native"
                                && v.native_available
                                && v.record.task.unfinished()))
                })
            })
            .unwrap_or(true);
        if let Ok(mut tabs) = app.state::<BrowserState>().tabs.lock() {
            if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.activity.downloading = protected;
            }
        }
    }
    pub fn enqueue(
        &self,
        app: &AppHandle,
        url: &str,
        expected: Option<String>,
    ) -> Result<DownloadTask, String> {
        let url = super::transfer::validate_download_url(url)?;
        self.check_policy(app, url.as_str())?;
        let id = app
            .state::<BrowserState>()
            .active_tab_id
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        let task = self.register(
            app,
            id,
            url.to_string(),
            super::transfer::file_name(&url),
            false,
            expected,
        )?;
        self.start(app, &task.id, false)?;
        Ok(self.record(&task.id)?.task)
    }
    fn check_policy(&self, app: &AppHandle, url: &str) -> Result<(), String> {
        super::transfer::validate_download_url(url)?;
        if crate::privacy::adblock_engine::should_block_url(
            url,
            *app.state::<BrowserState>()
                .shield_enabled
                .lock()
                .map_err(|e| e.to_string())?,
        ) {
            return Err("downloads.blocked".into());
        }
        Ok(())
    }
    fn commit_completed(&self, app: &AppHandle, record: &DownloadRecord) -> Result<(), String> {
        let state = app.state::<BrowserState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        repo::complete(db.as_ref().ok_or("downloads.database_unavailable")?, record)
    }
    pub fn complete(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        let mut record = self.record(id)?;
        files::publish(&record)?;
        record.published = true;
        record.task.status = "Completed".into();
        record.task.error = None;
        record.task.revision += 1;
        record.refresh_capabilities(false);
        self.commit_completed(app, &record)?;
        if let Some(v) = self.tasks.lock().map_err(|e| e.to_string())?.get_mut(id) {
            v.record = record.clone();
        }
        self.refresh_protection(app, &record.tab_id);
        let _ = app.emit_to("main", "download-task-updated", record.task);
        let _ = app.emit_to("main", "downloads-updated", ());
        Ok(())
    }
    pub fn shutdown(&self) {
        if let Ok(tasks) = self.tasks.lock() {
            for v in tasks.values().filter(|v| v.worker) {
                v.control.requested.store(1, Ordering::Release);
                v.control.changed.notify_one();
            }
        }
        let started = Instant::now();
        while self
            .tasks
            .lock()
            .is_ok_and(|m| m.values().any(|v| v.worker))
        {
            if started.elapsed() > Duration::from_secs(30) {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    pub fn flush(&self, app: &AppHandle) -> Result<(), String> {
        let records = self.tasks.lock().map_err(|e| e.to_string())?;
        let state = app.state::<BrowserState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let tx = db
            .as_ref()
            .ok_or("downloads.database_unavailable")?
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        for v in records.values() {
            if v.worker {
                return Err("downloads.busy".into());
            }
            repo::save(&tx, &v.record)?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
}
use sha2::Digest;
