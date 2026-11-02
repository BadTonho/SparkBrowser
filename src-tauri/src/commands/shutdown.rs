use crate::shutdown::{model::ShutdownState, ShutdownService};
use tauri::{AppHandle, State};
#[tauri::command]
pub fn get_shutdown_state(service: State<'_, ShutdownService>) -> Result<ShutdownState, String> {
    service.snapshot()
}
#[tauri::command]
pub fn prepare_shutdown(
    app: AppHandle,
    service: State<'_, ShutdownService>,
    attempt: String,
) -> Result<ShutdownState, String> {
    service.prepare(&app, &attempt)
}
#[tauri::command]
pub async fn confirm_shutdown(
    app: AppHandle,
    service: State<'_, ShutdownService>,
    attempt: String,
    revision: u64,
) -> Result<ShutdownState, String> {
    service.confirm(app, attempt, revision).await
}
#[tauri::command]
pub fn cancel_shutdown(
    app: AppHandle,
    service: State<'_, ShutdownService>,
    attempt: String,
) -> Result<ShutdownState, String> {
    service.cancel(&app, &attempt)
}
