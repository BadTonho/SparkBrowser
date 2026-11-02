use crate::{
    backup::{self, model::*},
    database::backup_repo,
    state::browser_state::BrowserState,
};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn export_backup(app: AppHandle, preferences: BackupPreferences) -> Result<bool, String> {
    let service = app.state::<backup::service::BackupService>();
    let _operation = service.operation.lock().await;
    let document = backup::service::capture(&app, preferences).await?;
    let dialog_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(path) = dialog_app
            .dialog()
            .file()
            .add_filter("Veloce backup", &["json"])
            .set_file_name("browser.veloce-backup.json")
            .blocking_save_file()
        else {
            return Ok(false);
        };
        let path = path.into_path().map_err(|e| e.to_string())?;
        if !path
            .to_string_lossy()
            .to_lowercase()
            .ends_with(".veloce-backup.json")
        {
            // Do not append after selection: that could overwrite a path the dialog never confirmed.
            return Err("backup.invalid_extension".into());
        }
        backup::files::write(&path, &document)?;
        Ok(true)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn choose_backup_file(app: AppHandle) -> Result<Option<String>, String> {
    let service = app.state::<backup::service::BackupService>();
    let _operation = service.operation.lock().await;
    let dialog_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(path) = dialog_app
            .dialog()
            .file()
            .add_filter("Veloce backup", &["json"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let doc = backup::files::read(&path.into_path().map_err(|e| e.to_string())?)?;
        dialog_app
            .state::<backup::service::BackupService>()
            .select(doc)
            .map(Some)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn preview_backup_import(
    app: AppHandle,
    document: Option<BackupDocument>,
    source_token: Option<String>,
    current_preferences: BackupPreferences,
) -> Result<BackupPreview, String> {
    backup::service::preview(&app, document, source_token, current_preferences).await
}
#[tauri::command]
pub async fn commit_backup_import(
    app: AppHandle,
    token: String,
    apply_preferences: bool,
    current_preferences: BackupPreferences,
) -> Result<BackupImportResult, String> {
    backup::service::import(&app, token, apply_preferences, current_preferences).await
}
#[tauri::command]
pub async fn get_backup_pending_preferences(
    state: State<'_, BrowserState>,
) -> Result<Option<PreferenceTask>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    backup_repo::pending(db.as_ref().ok_or("backup.database_unavailable")?)
}
#[tauri::command]
pub async fn finish_backup_preferences(
    state: State<'_, BrowserState>,
    id: String,
    error: Option<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    backup_repo::acknowledge(
        db.as_ref().ok_or("backup.database_unavailable")?,
        &id,
        error.as_deref(),
    )
}

#[tauri::command]
pub async fn discard_backup_import(app: AppHandle, source_token: String) -> Result<(), String> {
    let service = app.state::<backup::service::BackupService>();
    let _operation = service.operation.lock().await;
    service.discard(&source_token)
}
