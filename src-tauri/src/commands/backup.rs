use crate::{
    backup::{self, model::*},
    database::backup_repo,
    state::browser_state::BrowserState,
};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

/// Password for encrypted writes; `None` when encryption is off.
async fn stored_write_password(app: &AppHandle) -> Result<Option<String>, String> {
    let worker = app.clone();
    crate::database::executor::run(app.clone(), move |conn| {
        let service = worker.state::<backup::service::BackupService>();
        backup::encryption::write_password(conn, &service.sync.credentials)
    })
    .await
}

#[tauri::command]
pub async fn export_backup(app: AppHandle, preferences: BackupPreferences) -> Result<bool, String> {
    let service = app.state::<backup::service::BackupService>();
    let _operation = service.operation.lock().await;
    let document = backup::service::capture(&app, preferences).await?;
    let password = stored_write_password(&app).await?;
    let dialog_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(path) = dialog_app
            .dialog()
            .file()
            .add_filter("Spark backup", &["json"])
            .set_file_name("browser.spark-backup.json")
            .blocking_save_file()
        else {
            return Ok(false);
        };
        let path = path.into_path().map_err(|e| e.to_string())?;
        if !path
            .to_string_lossy()
            .to_lowercase()
            .ends_with(".spark-backup.json")
            && !path
                .to_string_lossy()
                .to_lowercase()
                .ends_with(".veloce-backup.json")
        {
            // Do not append after selection: that could overwrite a path the dialog never confirmed.
            return Err("backup.invalid_extension".into());
        }
        backup::files::write_with(&path, &document, password.as_deref())?;
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
            .add_filter("Spark backup", &["json"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let bytes = backup::files::read_bytes(&path.into_path().map_err(|e| e.to_string())?)?;
        let service = dialog_app.state::<backup::service::BackupService>();
        let password = {
            let state = dialog_app.state::<BrowserState>();
            let db = state
                .infrastructure()
                .db()
                .lock()
                .map_err(|e| e.to_string())?;
            db.as_ref()
                .and_then(|conn| backup::encryption::read_password(conn, &service.sync.credentials))
        };
        match backup::files::decode_with(&bytes, password.as_deref()) {
            Ok(doc) => service.select(doc).map(Some),
            // Keep the file so the interface can ask for its password and retry.
            Err(error)
                if error == "backup.password_required" || error == "backup.password_wrong" =>
            {
                service.hold_locked(bytes)?;
                Err(error)
            }
            Err(error) => Err(error),
        }
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
    options: Option<RestoreOptions>,
) -> Result<BackupPreview, String> {
    backup::service::preview_with(
        &app,
        document,
        source_token,
        current_preferences,
        options.unwrap_or_default(),
    )
    .await
}
#[tauri::command]
pub async fn commit_backup_import(
    app: AppHandle,
    token: String,
    apply_preferences: bool,
    current_preferences: BackupPreferences,
) -> Result<BackupImportResult, String> {
    let service = app.state::<backup::service::BackupService>();
    // Replace mode deletes data: keep a restorable copy first, and abort if it cannot be written.
    let snapshot = if service.deletes_local_data(&token)? {
        backup::local::create(
            &app,
            current_preferences.clone(),
            backup::local::Reason::PreRestore,
            false,
        )
        .await?
        .snapshot_id
    } else {
        None
    };
    let mut result =
        backup::service::import(&app, token, apply_preferences, current_preferences).await?;
    result.safety_snapshot = snapshot;
    Ok(result)
}
#[tauri::command]
pub async fn get_backup_pending_preferences(
    state: State<'_, BrowserState>,
) -> Result<Option<PreferenceTask>, String> {
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    backup_repo::pending(db.as_ref().ok_or("backup.database_unavailable")?)
}
#[tauri::command]
pub async fn finish_backup_preferences(
    state: State<'_, BrowserState>,
    id: String,
    error: Option<String>,
) -> Result<(), String> {
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
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

#[tauri::command]
pub async fn get_backup_sync_state(
    app: AppHandle,
) -> Result<backup::sync::SyncStateSnapshot, String> {
    let worker_app = app.clone();
    crate::database::executor::run(app, move |conn| {
        let service = worker_app.state::<backup::service::BackupService>();
        backup::sync::snapshot(conn, &service.sync)
    })
    .await
}

#[tauri::command]
pub async fn choose_backup_sync_folder(
    app: AppHandle,
    auto_save: bool,
    apply_preferences_on_direct_pull: bool,
) -> Result<Option<backup::sync::SyncStateSnapshot>, String> {
    let service = app.state::<backup::service::BackupService>();
    let _operation = service.operation.lock().await;
    let dialog_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(folder) = dialog_app.dialog().file().blocking_pick_folder() else {
            return Ok(None);
        };
        let path = folder.into_path().map_err(|e| e.to_string())?;
        let directory_path = path.to_string_lossy().into_owned();
        let state = dialog_app.state::<BrowserState>();
        let service = dialog_app.state::<backup::service::BackupService>();
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("backup.database_unavailable")?;
        let snap = backup::sync::save_config(
            conn,
            &service.sync,
            backup::sync::SyncTargetConfig::Folder {
                directory_path,
                auto_save,
                apply_preferences_on_direct_pull,
            },
            None,
        )?;
        Ok(Some(snap))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_backup_sync_config(
    app: AppHandle,
    state: State<'_, BrowserState>,
    config: backup::sync::SyncTargetConfig,
    secret: Option<String>,
    google_tab_id: Option<String>,
) -> Result<backup::sync::SyncStateSnapshot, String> {
    let service = app.state::<backup::service::BackupService>();
    let _operation = service.operation.lock().await;
    if let Some(id) = google_tab_id {
        if !matches!(
            config,
            backup::sync::SyncTargetConfig::GoogleDriveWeb { .. }
        ) {
            return Err("backup.sync_invalid_config".into());
        }
        let _tabs = state.session().operation().await;
        backup::gdrive_webview::session::bind(&app, &id)?;
    }
    let worker_app = app.clone();
    crate::database::executor::run(app.clone(), move |conn| {
        let service = worker_app.state::<backup::service::BackupService>();
        backup::sync::save_config(conn, &service.sync, config, secret)
    })
    .await
}

/// Saves how often the automatic sync runs and whether it is paused, for every sync target.
#[tauri::command]
pub async fn set_backup_sync_options(
    app: AppHandle,
    options: backup::sync_options::SyncOptions,
) -> Result<backup::sync::SyncStateSnapshot, String> {
    let worker_app = app.clone();
    crate::database::executor::run(app, move |conn| {
        let service = worker_app.state::<backup::service::BackupService>();
        backup::sync_options::save(conn, &options)?;
        backup::sync::snapshot(conn, &service.sync)
    })
    .await
}

/// When Spark opens: applies the file in the cloud if only the cloud changed, or prepares a review.
#[tauri::command]
pub async fn sync_backup_on_start(
    app: AppHandle,
    current_preferences: BackupPreferences,
) -> Result<backup::sync::SyncStartResponse, String> {
    backup::sync::sync_on_start(&app, current_preferences).await
}

/// The earlier versions of the file in the cloud.
#[tauri::command]
pub async fn list_backup_sync_versions(
    app: AppHandle,
) -> Result<Vec<backup::sync_versions::SyncVersion>, String> {
    backup::sync::list_versions(&app).await
}

/// Opens an earlier version of the file in the cloud in the review panel.
#[tauri::command]
pub async fn review_backup_sync_version(
    app: AppHandle,
    id: String,
    current_preferences: BackupPreferences,
) -> Result<backup::sync::SyncPullResponse, String> {
    backup::sync::review_version(&app, id, current_preferences).await
}

#[tauri::command]
pub async fn push_backup_sync(
    app: AppHandle,
    preferences: BackupPreferences,
    only_if_changed: bool,
) -> Result<backup::sync::SyncPushResult, String> {
    backup::sync::push_sync(&app, preferences, only_if_changed).await
}

#[tauri::command]
pub async fn pull_backup_sync(
    app: AppHandle,
    current_preferences: BackupPreferences,
    direct_apply: bool,
    apply_preferences_override: Option<bool>,
) -> Result<backup::sync::SyncPullResponse, String> {
    backup::sync::pull_sync(
        &app,
        current_preferences,
        direct_apply,
        apply_preferences_override,
    )
    .await
}

#[tauri::command]
pub async fn unlock_backup_file(app: AppHandle, password: String) -> Result<String, String> {
    let service = app.state::<backup::service::BackupService>();
    let _operation = service.operation.lock().await;
    let worker = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        worker
            .state::<backup::service::BackupService>()
            .unlock(&password)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn cancel_backup_unlock(app: AppHandle) -> Result<(), String> {
    app.state::<backup::service::BackupService>()
        .cancel_unlock()
}

#[tauri::command]
pub async fn get_local_backup_state(app: AppHandle) -> Result<backup::local::LocalState, String> {
    backup::local::state(&app).await
}

#[tauri::command]
pub async fn set_local_backup_schedule(
    app: AppHandle,
    schedule: backup::local::Schedule,
) -> Result<backup::local::LocalState, String> {
    backup::local::set_schedule(&app, schedule).await
}

#[tauri::command]
pub async fn set_backup_encryption(
    app: AppHandle,
    enabled: bool,
    password: Option<String>,
) -> Result<backup::local::LocalState, String> {
    backup::local::set_encryption(&app, enabled, password).await
}

#[tauri::command]
pub async fn create_local_backup(
    app: AppHandle,
    preferences: BackupPreferences,
    reason: backup::local::Reason,
    only_if_changed: bool,
) -> Result<backup::local::CreateOutcome, String> {
    backup::local::create(&app, preferences, reason, only_if_changed).await
}

#[tauri::command]
pub async fn delete_local_backup(
    app: AppHandle,
    id: String,
) -> Result<backup::local::LocalState, String> {
    backup::local::delete(&app, id).await
}

#[tauri::command]
pub async fn restore_local_backup(app: AppHandle, id: String) -> Result<String, String> {
    backup::local::restore(&app, id).await
}
