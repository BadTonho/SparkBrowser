pub mod profile_lock;
pub mod startup;
use crate::{database::recovery_repo, state::browser_state::BrowserState};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct RecoveryStatus {
    pub recovered: bool,
    pub restored_tabs: usize,
}
pub struct RecoveryService {
    pub status: RecoveryStatus,
}
impl RecoveryService {
    pub fn new(status: RecoveryStatus) -> Self {
        Self { status }
    }
}
/// If an installer returns, this process is running again, even if its pre-exit hook ran.
pub fn resume_after_installer(state: &BrowserState) -> Result<(), String> {
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    recovery_repo::resume_run(db.as_ref().ok_or("recovery.database_unavailable")?)
}
