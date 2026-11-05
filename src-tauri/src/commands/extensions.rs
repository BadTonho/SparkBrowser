use crate::extensions::types::{ExtensionAction, ExtensionDetails};
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn get_installed_extensions(app: AppHandle) -> Result<Vec<ExtensionDetails>, String> {
    crate::extensions::get_installed_extensions(&app)
}

#[tauri::command]
pub async fn load_unpacked_extension(
    app: AppHandle,
    path: Option<String>,
) -> Result<Option<ExtensionDetails>, String> {
    let folder_path = match path {
        Some(p) => PathBuf::from(p),
        None => {
            let dialog_app = app.clone();
            let selected = tauri::async_runtime::spawn_blocking(move || {
                dialog_app.dialog().file().blocking_pick_folder()
            })
            .await
            .map_err(|e| e.to_string())?;

            match selected {
                Some(p) => p.into_path().map_err(|e| e.to_string())?,
                None => return Ok(None),
            }
        }
    };

    let details = crate::extensions::load_unpacked_extension(&app, &folder_path).await?;
    Ok(Some(details))
}

#[tauri::command]
pub async fn toggle_extension(app: AppHandle, id: String, enabled: bool) -> Result<(), String> {
    crate::extensions::toggle_extension(&app, &id, enabled).await
}

#[tauri::command]
pub async fn remove_extension(app: AppHandle, id: String) -> Result<(), String> {
    crate::extensions::remove_extension(&app, &id).await
}

#[tauri::command]
pub async fn get_extension_action(
    app: AppHandle,
    id: String,
) -> Result<Option<ExtensionAction>, String> {
    crate::extensions::get_extension_action(&app, &id)
}

