use crate::database::downloads_repo::{list_downloads, DownloadItem};
use crate::downloads::manager::{DownloadManager, DownloadTask};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn get_download_visual(
    app: AppHandle,
    source: crate::downloads::visual::VisualSource,
) -> Result<crate::downloads::visual::DownloadVisual, String> {
    crate::downloads::visual::get(app, source).await
}

#[tauri::command]
pub async fn get_downloads(state: State<'_, BrowserState>) -> Result<Vec<DownloadItem>, String> {
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    list_downloads(db.as_ref().ok_or("Download database is not ready")?)
}
#[tauri::command]
pub async fn get_download_tasks(
    manager: State<'_, DownloadManager>,
) -> Result<Vec<DownloadTask>, String> {
    manager.list()
}
/// Asks where to save when the setting is on. `None` means the user cancelled the dialog;
/// `Some(None)` means the default folder.
pub(crate) async fn choose_target(
    app: &AppHandle,
    state: &BrowserState,
    name: String,
) -> Result<Option<Option<std::path::PathBuf>>, String> {
    if !state.policies().ask_download_destination() {
        return Ok(Some(None));
    }
    let dialog_app = app.clone();
    let start = crate::downloads::files::directory(app).ok();
    let selected = tauri::async_runtime::spawn_blocking(move || {
        let mut dialog = dialog_app.dialog().file().set_file_name(name);
        if let Some(start) = start {
            dialog = dialog.set_directory(start);
        }
        dialog.blocking_save_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(path) = selected else {
        return Ok(None);
    };
    Ok(Some(Some(
        path.into_path().map_err(|_| "downloads.invalid_target")?,
    )))
}
#[tauri::command]
pub async fn enqueue_download(
    app: AppHandle,
    webview: tauri::Webview,
    manager: State<'_, DownloadManager>,
    state: State<'_, BrowserState>,
    url: String,
    expected_sha256: Option<String>,
) -> Result<Option<DownloadTask>, String> {
    let parsed = crate::downloads::transfer::validate_download_url(&url)?;
    let name = crate::downloads::transfer::file_name(&parsed);
    let Some(target) = choose_target(&app, &state, name).await? else {
        return Ok(None);
    };
    manager
        .enqueue(
            &app,
            &crate::commands::access::caller_window(&webview),
            &url,
            expected_sha256,
            target,
        )
        .map(Some)
}
#[tauri::command]
pub async fn cancel_download(
    app: AppHandle,
    manager: State<'_, DownloadManager>,
    task_id: String,
) -> Result<(), String> {
    manager.control(&app, &task_id, "cancel").await
}
/// Clears the finished downloads from the list; with `older_than_days`, only the older ones.
#[tauri::command]
pub async fn clear_all_downloads(
    app: AppHandle,
    manager: State<'_, DownloadManager>,
    older_than_days: Option<u32>,
) -> Result<Vec<String>, String> {
    manager.clear_finished(&app, older_than_days).await
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

/// Where downloads go when Spark does not ask, for Settings → Downloads.
#[tauri::command]
pub fn get_download_location(
    app: AppHandle,
) -> Result<crate::downloads::location::Location, String> {
    current_location(&app)
}

fn current_location(app: &AppHandle) -> Result<crate::downloads::location::Location, String> {
    use tauri::Manager;
    let custom = app
        .state::<crate::downloads::location::DownloadLocation>()
        .custom();
    let chosen = crate::downloads::location::chosen(custom);
    let is_default = chosen.is_none();
    let path = match chosen {
        Some(path) => path,
        None => app.path().download_dir().map_err(|e| e.to_string())?,
    };
    Ok(crate::downloads::location::Location {
        path: path
            .to_string_lossy()
            .trim_start_matches(r"\?\")
            .to_string(),
        is_default,
    })
}

/// Opens the folder picker and keeps the choice. `None` when the picker was cancelled.
#[tauri::command]
pub async fn choose_download_location(
    app: AppHandle,
) -> Result<Option<crate::downloads::location::Location>, String> {
    use tauri::Manager;
    let start = crate::downloads::files::directory(&app).ok();
    let dialog_app = app.clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        let mut dialog = dialog_app.dialog().file();
        if let Some(start) = start {
            dialog = dialog.set_directory(start);
        }
        dialog.blocking_pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(picked) = picked else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|_| "downloads.invalid_location")?;
    let folder = crate::downloads::location::usable(&path)?;
    let saved = folder.clone();
    crate::database::executor::run(app.clone(), move |conn| {
        crate::downloads::location::store(conn, Some(&saved))
    })
    .await?;
    app.state::<crate::downloads::location::DownloadLocation>()
        .set(Some(folder));
    current_location(&app).map(Some)
}

/// Goes back to the system Downloads folder.
#[tauri::command]
pub async fn reset_download_location(
    app: AppHandle,
) -> Result<crate::downloads::location::Location, String> {
    use tauri::Manager;
    crate::database::executor::run(app.clone(), |conn| {
        crate::downloads::location::store(conn, None)
    })
    .await?;
    app.state::<crate::downloads::location::DownloadLocation>()
        .set(None);
    current_location(&app)
}
