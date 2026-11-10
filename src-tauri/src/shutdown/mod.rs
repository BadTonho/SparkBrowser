pub mod model;
use crate::{
    database::navigation_writer::NavigationWriter, downloads::manager::DownloadManager,
    state::browser_state::BrowserState,
};
use model::{ShutdownPurpose, ShutdownReview, ShutdownRisk, ShutdownState};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct ShutdownService(Mutex<ShutdownState>);
impl Default for ShutdownService {
    fn default() -> Self {
        Self(Mutex::new(ShutdownState {
            phase: "idle".into(),
            ..Default::default()
        }))
    }
}
impl ShutdownService {
    pub fn snapshot(&self) -> Result<ShutdownState, String> {
        Ok(self.0.lock().map_err(|e| e.to_string())?.clone())
    }
    pub fn blocked(&self) -> bool {
        self.0.lock().map_or(true, |s| s.phase != "idle")
    }
    fn publish(&self, app: &AppHandle) -> Result<ShutdownState, String> {
        let state = self.snapshot()?;
        let _ = crate::security::emit_shell(app, "shutdown-state", &state);
        Ok(state)
    }
    pub fn request(
        &self,
        app: &AppHandle,
        update_version: Option<String>,
    ) -> Result<ShutdownState, String> {
        let purpose = if update_version.is_some() {
            ShutdownPurpose::InstallUpdate
        } else {
            ShutdownPurpose::Exit
        };
        self.request_purpose(app, purpose, update_version)
    }
    pub fn request_restart(&self, app: &AppHandle) -> Result<ShutdownState, String> {
        self.request_purpose(app, ShutdownPurpose::RestartRuntime, None)
    }
    fn request_purpose(
        &self,
        app: &AppHandle,
        purpose: ShutdownPurpose,
        update_version: Option<String>,
    ) -> Result<ShutdownState, String> {
        let mut state = self.0.lock().map_err(|e| e.to_string())?;
        if state.phase != "idle" {
            return Ok(state.clone());
        }
        *state = ShutdownState {
            purpose,
            revision: state.revision + 1,
            phase: "flushing".into(),
            attempt: Some(crate::window::webview_manager::create_internal_bridge_token()),
            update_version,
            review: None,
            error: None,
        };
        drop(state);
        self.publish(app)
    }
    pub fn cancel(&self, app: &AppHandle, attempt: &str) -> Result<ShutdownState, String> {
        let mut state = self.0.lock().map_err(|e| e.to_string())?;
        if state.attempt.as_deref() != Some(attempt) || state.phase == "committing" {
            return Err("shutdown.stale_attempt".into());
        }
        if state.phase == "error" {
            crate::recovery::resume_after_installer(&app.state::<BrowserState>())?;
        }
        *state = ShutdownState {
            revision: state.revision + 1,
            phase: "idle".into(),
            ..Default::default()
        };
        drop(state);
        self.publish(app)
    }
    pub fn failed(&self, app: &AppHandle, error: String) -> Result<ShutdownState, String> {
        let mut state = self.0.lock().map_err(|e| e.to_string())?;
        state.phase = "error".into();
        state.error = Some(error);
        state.revision += 1;
        drop(state);
        self.publish(app)
    }
    pub fn prepare(&self, app: &AppHandle, attempt: &str) -> Result<ShutdownState, String> {
        let mut state = self.0.lock().map_err(|e| e.to_string())?;
        if state.attempt.as_deref() != Some(attempt)
            || !matches!(state.phase.as_str(), "flushing" | "error")
        {
            return Err("shutdown.stale_attempt".into());
        }
        let mut review = review(app)?;
        state.revision += 1;
        review.revision = state.revision;
        state.review = Some(review);
        state.phase = "review".into();
        state.error = None;
        drop(state);
        self.publish(app)
    }
    pub async fn confirm(
        &self,
        app: AppHandle,
        attempt: String,
        revision: u64,
    ) -> Result<ShutdownState, String> {
        let approved = {
            let mut state = self.0.lock().map_err(|e| e.to_string())?;
            if state.attempt.as_deref() != Some(&attempt)
                || state.phase != "review"
                || state.review.as_ref().map(|r| r.revision) != Some(revision)
            {
                return Err("shutdown.stale_attempt".into());
            }
            let current = review(&app)?;
            if !state
                .review
                .as_ref()
                .is_some_and(|r| r.same_activity(&current))
            {
                state.revision += 1;
                state.review = Some(ShutdownReview {
                    revision: state.revision,
                    ..current
                });
                drop(state);
                return self.publish(&app);
            }
            state.phase = "committing".into();
            state.revision += 1;
            state.clone()
        };
        self.publish(&app)?;
        let purpose = approved.purpose;
        let a = app.clone();
        let result = tauri::async_runtime::spawn_blocking(move || persist(&a, &approved))
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r);
        match result {
            Ok(Some(changed)) => {
                let mut state = self.0.lock().map_err(|e| e.to_string())?;
                state.revision += 1;
                state.phase = "review".into();
                state.review = Some(ShutdownReview {
                    revision: state.revision,
                    ..changed
                });
                drop(state);
                self.publish(&app)
            }
            Ok(None) => {
                if purpose == ShutdownPurpose::RestartRuntime {
                    if let Err(error) = crate::webview_runtime::restart(&app) {
                        let _ = resume(&app);
                        return self.failed(&app, error);
                    }
                    return self.snapshot();
                }
                let version = self.snapshot()?.update_version;
                if let Some(version) = version {
                    #[cfg(windows)]
                    let result = crate::updates::install(app.clone(), version, true).await;
                    #[cfg(not(windows))]
                    let result: Result<(), String> = {
                        let _ = version;
                        Err("shutdown.update_unavailable".into())
                    };
                    let _ = resume(&app);
                    self.failed(
                        &app,
                        result
                            .err()
                            .unwrap_or_else(|| "shutdown.installer_returned".into()),
                    )
                } else {
                    app.exit(0);
                    self.snapshot()
                }
            }
            Err(error) => {
                let _ = resume(&app);
                self.failed(&app, format!("shutdown.save_failed: {error}"))
            }
        }
    }
}
pub fn review(app: &AppHandle) -> Result<ShutdownReview, String> {
    let mut tabs: Vec<_> = app
        .state::<BrowserState>()
        .snapshot_tabs()
        .into_iter()
        .filter(|t| {
            t.activity.dirty_form
                || (t.activity.playing_media || t.activity.picture_in_picture)
                || t.activity.downloading
        })
        .map(|t| ShutdownRisk {
            id: t.id,
            title: t.title,
            url: t.url,
            form: t.activity.dirty_form,
            media: t.activity.playing_media || t.activity.picture_in_picture,
            native_download: t.activity.downloading,
        })
        .collect();
    tabs.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = ShutdownReview {
        tabs,
        ..Default::default()
    };
    for task in app.state::<DownloadManager>().list()? {
        if task.source == "native" && task.unfinished() {
            out.native_downloads.push(task.id);
        } else if task.pending() {
            out.downloads.push(task.id);
        }
    }
    out.downloads.sort();
    out.native_downloads.sort();
    if app
        .state::<crate::updates::UpdateService>()
        .snapshot()?
        .status
        == "downloading"
    {
        out.downloads.push("application-update".into());
    }
    Ok(out)
}
/// Off the UI thread: drain writers before taking the lock they also acquire.
fn persist(app: &AppHandle, approved: &ShutdownState) -> Result<Option<ShutdownReview>, String> {
    let writer = app.state::<NavigationWriter>();
    writer.freeze()?;
    let current = review(app)?;
    if !approved
        .review
        .as_ref()
        .is_some_and(|r| r.same_activity(&current))
    {
        writer.resume();
        return Ok(Some(current));
    }
    let downloads = app.state::<DownloadManager>();
    if approved.update_version.is_some() {
        crate::updates::validate_restart(
            &app.state::<BrowserState>().snapshot_tabs(),
            downloads.list()?.iter().any(|d| d.pending()),
            true,
        )?;
    } else {
        app.state::<crate::updates::UpdateService>()
            .settle_for_exit()?;
        downloads.shutdown();
        crate::downloads::native::stop_for_exit(app)?;
    }
    downloads.flush(app)?;
    let browser = app.state::<BrowserState>();
    let _tabs = browser.session().blocking_operation();
    let final_review = review(app)?;
    if final_review.tabs.iter().any(|t| {
        (t.form || t.media)
            && !approved.review.as_ref().is_some_and(|r| {
                r.tabs.iter().any(|a| {
                    a.id == t.id && a.url == t.url && (!t.form || a.form) && (!t.media || a.media)
                })
            })
    }) {
        writer.resume();
        return Ok(Some(final_review));
    }
    browser.persist_run_checkpoint(crate::database::recovery_repo::RunCheckpoint::CleanExit)?;
    Ok(None)
}
fn resume(app: &AppHandle) -> Result<(), String> {
    let result = crate::recovery::resume_after_installer(&app.state::<BrowserState>());
    app.state::<NavigationWriter>().resume();
    result
}
