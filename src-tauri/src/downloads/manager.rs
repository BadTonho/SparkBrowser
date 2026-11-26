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
use tauri::{AppHandle, Manager};

pub struct Control {
    pub requested: AtomicU8,
    pub changed: tokio::sync::Notify,
}
pub struct Registration {
    pub tab_id: String,
    pub url: String,
    pub name: String,
    pub native: bool,
    pub selected_target: Option<std::path::PathBuf>,
    pub expected: Option<String>,
    /// The page a video came from.
    pub referer: Option<String>,
    /// An HLS stream instead of a single file.
    pub stream: bool,
}
pub struct VideoRequest {
    pub tab_id: String,
    pub url: tauri::Url,
    pub name: String,
    pub referer: Option<String>,
    pub stream: bool,
    pub target: Option<std::path::PathBuf>,
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
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        repo::save(db.as_ref().ok_or("downloads.database_unavailable")?, record)
    }
    pub fn initialize(&self, app: &AppHandle) -> Result<(), String> {
        let records = {
            let state = app.state::<BrowserState>();
            let db = state
                .infrastructure()
                .db()
                .lock()
                .map_err(|e| e.to_string())?;
            let conn = db.as_ref().ok_or("downloads.database_unavailable")?;
            conn.execute("DELETE FROM download_jobs WHERE json_extract(document,'$.task.status') IN('Completed','Cancelled') AND id NOT IN(SELECT id FROM download_jobs ORDER BY rowid DESC LIMIT 100)",[]).map_err(|e|e.to_string())?;
            repo::load(conn)?
        };
        for mut record in records {
            if record.task.status == "Finalizing" {
                match files::publish(&record) {
                    Ok(path) => {
                        record.task.file_name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        record.task.save_path = Some(path.to_string_lossy().into());
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
        registration: Registration,
    ) -> Result<DownloadTask, String> {
        let Registration {
            tab_id,
            url,
            name,
            native,
            selected_target,
            expected,
            referer,
            stream,
        } = registration;
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
        let (directory, save_path, file_name) = if let Some(target) = selected_target {
            let target = files::available_target(&target)?;
            let parent = target.parent().ok_or("downloads.invalid_target")?;
            let directory =
                std::fs::canonicalize(parent).map_err(|_| "downloads.invalid_target")?;
            let target = directory.join(target.file_name().ok_or("downloads.invalid_target")?);
            let file_name = target
                .file_name()
                .ok_or("downloads.invalid_target")?
                .to_string_lossy()
                .to_string();
            (directory, target, file_name)
        } else {
            let directory = files::directory(app)?;
            let target = directory.join(format!("{id}-{name}"));
            (directory, target, name)
        };
        let page = app
            .state::<BrowserState>()
            .snapshot_tabs()
            .into_iter()
            .find(|t| t.id == tab_id)
            .map(|t| t.url);
        let task = DownloadTask {
            id: id.clone(),
            revision: 1,
            file_name: file_name.clone(),
            url,
            source: if native {
                "native"
            } else if stream {
                "hls"
            } else {
                "url"
            }
            .into(),
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
            save_path: Some(save_path.to_string_lossy().into()),
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
            referer,
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
        crate::error_log::ignored(crate::security::emit_shell(
            app,
            "download-task-updated",
            &task,
        ));
        self.refresh_protection(app, &self.record(id)?.tab_id);
        Ok(task)
    }
    pub fn update(&self, app: &AppHandle, id: &str, f: impl FnOnce(&mut DownloadTask)) {
        crate::error_log::ignored(self.edit(app, id, false, |r| f(&mut r.task)));
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
        if let Ok(Some(payload)) = app.state::<BrowserState>().observe_document(tab_id, |tab| {
            tab.activity.downloading = protected;
        }) {
            let _ =
                crate::security::emit_shell_for_tab(app, tab_id, "tab-navigation-updated", payload);
        }
    }
    pub fn enqueue(
        &self,
        app: &AppHandle,
        window_id: &str,
        url: &str,
        expected: Option<String>,
        target: Option<std::path::PathBuf>,
    ) -> Result<DownloadTask, String> {
        let url = super::transfer::validate_download_url(url)?;
        let id = app
            .state::<BrowserState>()
            .session()
            .window(window_id)
            .ok_or("windows.not_found")?
            .active_tab_id()
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        self.check_policy(app, url.as_str(), Some(&id))?;
        let task = self.register(
            app,
            Registration {
                tab_id: id,
                url: url.to_string(),
                name: super::transfer::file_name(&url),
                native: false,
                selected_target: target,
                expected,
                referer: None,
                stream: false,
            },
        )?;
        self.start(app, &task.id, false)?;
        Ok(self.record(&task.id)?.task)
    }
    /// A video found in a page, which may not be the active tab by the time it is saved.
    pub fn enqueue_video(
        &self,
        app: &AppHandle,
        video: VideoRequest,
    ) -> Result<DownloadTask, String> {
        self.check_policy(app, video.url.as_str(), Some(&video.tab_id))?;
        // Named after the page title; `register` adds a number when the name is taken.
        let target = match video.target {
            Some(target) => target,
            None => files::directory(app)?.join(&video.name),
        };
        let task = self.register(
            app,
            Registration {
                tab_id: video.tab_id,
                url: video.url.to_string(),
                name: video.name,
                native: false,
                selected_target: Some(target),
                expected: None,
                referer: video.referer,
                stream: video.stream,
            },
        )?;
        self.start(app, &task.id, false)?;
        Ok(self.record(&task.id)?.task)
    }
    fn check_policy(&self, app: &AppHandle, url: &str, tab_id: Option<&str>) -> Result<(), String> {
        super::transfer::validate_download_url(url)?;
        if crate::privacy::adblock_engine::should_block_url(
            url,
            crate::privacy::adblock_engine::download_shield_enabled(
                &app.state::<BrowserState>(),
                tab_id,
                url,
            ),
        ) {
            return Err("downloads.blocked".into());
        }
        Ok(())
    }
    fn commit_completed(&self, app: &AppHandle, record: &DownloadRecord) -> Result<(), String> {
        let state = app.state::<BrowserState>();
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        repo::complete(db.as_ref().ok_or("downloads.database_unavailable")?, record)
    }
    pub fn complete(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        let mut record = self.record(id)?;
        let published_path = files::publish(&record)?;
        record.task.file_name = published_path
            .file_name()
            .ok_or("downloads.invalid_target")?
            .to_string_lossy()
            .to_string();
        record.task.save_path = Some(published_path.to_string_lossy().into());
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
        crate::error_log::ignored(crate::security::emit_shell(
            app,
            "download-task-updated",
            record.task,
        ));
        crate::error_log::ignored(crate::security::emit_shell(app, "downloads-updated", ()));
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
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
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
