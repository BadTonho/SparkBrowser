use crate::database::downloads_repo::{
    clear_downloads, list_downloads, record_download, DownloadItem,
};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager, State, Url};

fn extract_filename_from_url(raw_url: &str) -> String {
    if let Ok(parsed) = Url::parse(raw_url) {
        if let Some(mut segments) = parsed.path_segments() {
            if let Some(last) = segments.next_back() {
                let clean = last.trim();
                if !clean.is_empty() && clean.contains('.') {
                    return clean.to_string();
                }
            }
        }
        if let Some(host) = parsed.host_str() {
            return format!("{}-download.html", host.trim_start_matches("www."));
        }
    }
    "downloaded-file.bin".to_string()
}

#[tauri::command]
pub fn get_downloads(state: State<'_, BrowserState>) -> Result<Vec<DownloadItem>, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return list_downloads(conn);
    }
    Ok(Vec::new())
}

#[tauri::command]
pub fn enqueue_download(
    app: AppHandle,
    state: State<'_, BrowserState>,
    url: String,
) -> Result<Vec<DownloadItem>, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("Download URL cannot be empty".to_string());
    }

    let file_name = extract_filename_from_url(trimmed);
    let downloads_dir = app
        .path()
        .download_dir()
        .unwrap_or_else(|_| std::env::temp_dir());

    let target_path = downloads_dir.join(&file_name);
    let save_path_str = target_path.to_string_lossy().to_string();

    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return record_download(conn, &file_name, trimmed, &save_path_str, "Completed");
    }

    Ok(Vec::new())
}

#[tauri::command]
pub fn clear_all_downloads(state: State<'_, BrowserState>) -> Result<(), String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        clear_downloads(conn)?;
    }
    Ok(())
}

