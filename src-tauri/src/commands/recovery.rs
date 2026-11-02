use crate::recovery::{RecoveryService, RecoveryStatus};
use tauri::State;

#[tauri::command]
pub fn get_recovery_status(service: State<'_, RecoveryService>) -> RecoveryStatus {
    service.status.clone()
}
