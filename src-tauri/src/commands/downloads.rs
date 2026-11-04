use crate::database::downloads_repo::{list_downloads, DownloadItem};
use crate::downloads::manager::{DownloadManager, DownloadTask};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn get_download_visual(
    app: AppHandle,
    source: crate::downloads::visual::VisualSource,
) -> Result<crate::downloads::visual::DownloadVisual, String> {
    crate::downloads::visual::get(app, source).await
}

#[tauri::command]
pub async fn get_downloads(state: State<'_, BrowserState>) -> Result<Vec<DownloadItem>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    list_downloads(db.as_ref().ok_or("Download database is not ready")?)
}
#[tauri::command]
pub async fn get_download_tasks(
    manager: State<'_, DownloadManager>,
) -> Result<Vec<DownloadTask>, String> {
    manager.list()
}
#[tauri::command]
pub async fn enqueue_download(
    app: AppHandle,
    manager: State<'_, DownloadManager>,
    url: String,
    expected_sha256: Option<String>,
) -> Result<DownloadTask, String> {
    manager.enqueue(&app, &url, expected_sha256)
}
#[tauri::command]
pub async fn cancel_download(
    app: AppHandle,
    manager: State<'_, DownloadManager>,
    task_id: String,
) -> Result<(), String> {
    manager.control(&app, &task_id, "cancel").await
}
#[tauri::command]
pub async fn clear_all_downloads(
    app: AppHandle,
    manager: State<'_, DownloadManager>,
) -> Result<Vec<String>, String> {
    manager.clear_finished(&app).await
}

#[tauri::command]
pub async fn control_download(
    app: AppHandle,
    manager: State<'_, DownloadManager>,
    task_id: String,
    action: String,
) -> Result<(), String> {
    manager.control(&app, &task_id, &action).await
}
#[tauri::command]
pub async fn set_download_checksum(
    app: AppHandle,
    manager: State<'_, DownloadManager>,
    task_id: String,
    checksum: Option<String>,
) -> Result<(), String> {
    manager.set_checksum(&app, &task_id, checksum).await
}
