use crate::database::downloads_repo::{
    clear_downloads, list_downloads, record_download, DownloadItem,
};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn get_downloads(state: State<'_, BrowserState>) -> Result<Vec<DownloadItem>, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return list_downloads(conn);
    }
    Ok(Vec::new())
}

#[tauri::command]
pub async fn enqueue_download(
    app: AppHandle,
    state: State<'_, BrowserState>,
    url: String,
) -> Result<Vec<DownloadItem>, String> {
    let destination = app.path().download_dir().map_err(|e| e.to_string())?;
    let parsed = crate::downloads::transfer::validate_download_url(&url)?;
    let shield_enabled = *state.shield_enabled.lock().map_err(|e| e.to_string())?;
    if crate::privacy::adblock_engine::should_block_url(parsed.as_str(), shield_enabled) {
        return Err("Download blocked by the privacy shield".to_string());
    }
    let saved = tauri::async_runtime::spawn_blocking(move || {
        crate::downloads::transfer::download_to_directory(&parsed, &destination, shield_enabled)
    })
    .await
    .map_err(|e| e.to_string())??;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Download database is not ready")?;
    record_download(
        conn,
        &saved.file_name,
        &url,
        &saved.path.to_string_lossy(),
        "Completed",
    )
}

#[tauri::command]
pub fn clear_all_downloads(state: State<'_, BrowserState>) -> Result<(), String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        clear_downloads(conn)?;
    }
    Ok(())
}
