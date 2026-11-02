pub mod profile_lock;
pub mod startup;
use crate::{database::recovery_repo, state::browser_state::BrowserState};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, Serialize)]
pub struct RecoveryStatus {
    pub recovered: bool,
    pub restored_tabs: usize,
}
pub struct RecoveryService {
    pub status: RecoveryStatus,
    pub shutting_down: AtomicBool,
}
impl RecoveryService {
    pub fn new(status: RecoveryStatus) -> Self {
        Self {
            status,
            shutting_down: AtomicBool::new(false),
        }
    }
}
/// Runs off the UI thread. Freeze before acquiring tab_operations: the writer needs that lock.
pub fn prepare_clean_exit(app: &AppHandle) -> Result<(), String> {
    let writer = app.state::<crate::database::navigation_writer::NavigationWriter>();
    writer.freeze()?;
    let state = app.state::<BrowserState>();
    let _tabs = state.tab_operations.blocking_lock();
    state.startup_ready.store(false, Ordering::Release);
    state.persist_run_checkpoint(recovery_repo::RunCheckpoint::CleanExit)
}
/// If an installer returns, this process is running again, even if its pre-exit hook ran.
pub fn resume_after_installer(state: &BrowserState) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    recovery_repo::resume_run(db.as_ref().ok_or("recovery.database_unavailable")?)
}
