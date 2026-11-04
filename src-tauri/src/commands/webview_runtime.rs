use crate::{
    shutdown::{model::ShutdownState, ShutdownService},
    webview_runtime::{RuntimeService, RuntimeStatus},
};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_webview_runtime_status(
    service: State<'_, RuntimeService>,
) -> Result<RuntimeStatus, String> {
    service.snapshot()
}

#[tauri::command]
pub fn restart_webview_runtime(
    app: AppHandle,
    service: State<'_, RuntimeService>,
    shutdown: State<'_, ShutdownService>,
) -> Result<ShutdownState, String> {
    service.ensure_pending()?;
    shutdown.request_restart(&app)
}
