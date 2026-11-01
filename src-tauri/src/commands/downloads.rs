use crate::database::downloads_repo::{clear_downloads, list_downloads, DownloadItem};
use crate::downloads::manager::{DownloadManager, DownloadTask};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, State};

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
) -> Result<DownloadTask, String> {
    manager.enqueue(&app, &url)
}
#[tauri::command]
pub async fn cancel_download(
    app: AppHandle,
    manager: State<'_, DownloadManager>,
    task_id: String,
) -> Result<(), String> {
    manager.cancel(&app, &task_id)
}
#[tauri::command]
pub async fn clear_all_downloads(state: State<'_, BrowserState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    clear_downloads(db.as_ref().ok_or("Download database is not ready")?)
}
