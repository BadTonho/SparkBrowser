pub mod crx;
pub mod manifest;
pub mod storage;
pub mod types;

#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
pub use windows as platform;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux as platform;

use crate::database::extensions_repo::{self, ExtensionRecord};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use types::{ExtensionAction, ExtensionDetails, ExtensionManifest};

fn generate_extension_id() -> String {
    let mut bytes = [0u8; 16];
    let _ = getrandom::fill(&mut bytes);
    format!(
        "ext-{}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    )
}

/// Loads an unpacked extension folder, validates its manifest, copies it to persistent storage if needed,
/// registers it with the underlying browser engine, and saves it into the SQLite database.
pub async fn load_unpacked_extension(
    app: &AppHandle,
    source_dir: &Path,
) -> Result<ExtensionDetails, String> {
    if !source_dir.exists() || !source_dir.is_dir() {
        return Err(format!(
            "Directory does not exist: {}",
            source_dir.display()
        ));
    }

    let (manifest, manifest_raw) = manifest::read_manifest_from_directory(source_dir)?;

    let extensions_base = storage::extensions_dir(app)?;

    // Store a persistent copy in app_data_dir/extensions/<id> BEFORE registering with the browser engine,
    // so WebView2 never points to a temporary directory that will be deleted.
    let installed_path = if source_dir.starts_with(&extensions_base) {
        source_dir.to_path_buf()
    } else {
        let temp_id = generate_extension_id();
        let target = extensions_base.join(&temp_id);
        if target.exists() {
            let _ = std::fs::remove_dir_all(&target);
        }
        storage::copy_dir_all(source_dir, &target)
            .map_err(|e| format!("Could not create persistent copy of extension: {}", e))?;
        target
    };

    // Register persistent directory with browser engine (WebView2 on Windows, stub/bridge on Linux)
    let engine_id = platform::add_browser_extension(app, &installed_path.to_string_lossy()).await?;
    let extension_id = if engine_id.is_empty() {
        installed_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("ext")
            .to_string()
    } else {
        engine_id
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let record = ExtensionRecord {
        id: extension_id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone().unwrap_or_default(),
        path: installed_path.to_string_lossy().to_string(),
        enabled: true,
        manifest_json: manifest_raw,
        installed_at: now,
    };

    let conn = crate::database::connection::init_database(app)?;
    extensions_repo::save(&conn, &record)?;

    to_extension_details(&installed_path, &record, &manifest)
}

/// Downloads and installs an extension directly from a Chrome Web Store URL or ID.
pub async fn install_from_webstore(
    app: &AppHandle,
    url_or_id: &str,
) -> Result<ExtensionDetails, String> {
    let id = crx::parse_webstore_id(url_or_id)
        .ok_or_else(|| "Invalid Chrome Web Store URL or Extension ID".to_string())?;

    let download_id = id.clone();
    let bytes = tauri::async_runtime::spawn_blocking(move || crx::download_webstore_crx(&download_id))
        .await
        .map_err(|e| e.to_string())??;

    // Extract directly into the permanent extension directory: app_data_dir/extensions/<id>
    let dest_dir = storage::extensions_dir(app)?.join(&id);
    if dest_dir.exists() {
        let _ = std::fs::remove_dir_all(&dest_dir);
    }
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create extension directory: {}", e))?;
    crx::extract_crx_or_zip(&bytes, &dest_dir)?;

    load_unpacked_extension(app, &dest_dir).await
}

/// Installs an extension from a local `.crx` or `.zip` archive.
pub async fn install_from_archive(
    app: &AppHandle,
    archive_path: &Path,
) -> Result<ExtensionDetails, String> {
    let bytes =
        std::fs::read(archive_path).map_err(|e| format!("Failed to read archive: {}", e))?;
    let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    crx::extract_crx_or_zip(&bytes, temp_dir.path())?;

    load_unpacked_extension(app, temp_dir.path()).await
}

/// Retrieves all installed extensions from the database and constructs full details.
pub fn get_installed_extensions(app: &AppHandle) -> Result<Vec<ExtensionDetails>, String> {
    let conn = crate::database::connection::init_database(app)?;
    let records = extensions_repo::get_all(&conn)?;

    let mut details = Vec::new();
    for rec in records {
        let manifest =
            manifest::parse_manifest(&rec.manifest_json).unwrap_or_else(|_| ExtensionManifest {
                manifest_version: 2,
                name: rec.name.clone(),
                version: rec.version.clone(),
                description: Some(rec.description.clone()),
                permissions: Vec::new(),
                host_permissions: Vec::new(),
                action: None,
                options_page: None,
                icons: std::collections::HashMap::new(),
            });

        let path = PathBuf::from(&rec.path);
        if let Ok(item) = to_extension_details(&path, &rec, &manifest) {
            details.push(item);
        }
    }
    Ok(details)
}

/// Toggles extension enabled/disabled state.
pub async fn toggle_extension(app: &AppHandle, id: &str, enabled: bool) -> Result<(), String> {
    let conn = crate::database::connection::init_database(app)?;
    extensions_repo::set_enabled(&conn, id, enabled)?;
    platform::set_browser_extension_enabled(app, id.to_string(), enabled).await
}

/// Removes an extension from database, engine runtime, and disk.
pub async fn remove_extension(app: &AppHandle, id: &str) -> Result<(), String> {
    let conn = crate::database::connection::init_database(app)?;
    extensions_repo::delete(&conn, id)?;
    let _ = platform::remove_browser_extension(app, id.to_string()).await;
    let _ = storage::remove_extension_dir(app, id);
    Ok(())
}

/// Retrieves action/popup info for an installed extension.
pub fn get_extension_action(app: &AppHandle, id: &str) -> Result<Option<ExtensionAction>, String> {
    let conn = crate::database::connection::init_database(app)?;
    let rec =
        extensions_repo::get(&conn, id)?.ok_or_else(|| format!("Extension {} not found", id))?;

    let manifest = manifest::parse_manifest(&rec.manifest_json)?;
    Ok(manifest.action)
}

fn to_extension_details(
    dir: &Path,
    record: &ExtensionRecord,
    manifest: &ExtensionManifest,
) -> Result<ExtensionDetails, String> {
    let icon_data_url = manifest::find_best_icon_url(dir, manifest);

    let popup_url = manifest
        .action
        .as_ref()
        .and_then(|a| a.default_popup.as_ref())
        .map(|popup| {
            let clean = popup.trim_start_matches(['/', '\\']);
            dir.join(clean).to_string_lossy().to_string()
        });

    let options_url = manifest.options_page.as_ref().map(|opt| {
        let clean = opt.trim_start_matches(['/', '\\']);
        dir.join(clean).to_string_lossy().to_string()
    });

    let mut permissions = manifest.permissions.clone();
    permissions.extend(manifest.host_permissions.clone());

    Ok(ExtensionDetails {
        id: record.id.clone(),
        name: record.name.clone(),
        version: record.version.clone(),
        description: record.description.clone(),
        path: record.path.clone(),
        enabled: record.enabled,
        installed_at: record.installed_at,
        icon_data_url,
        popup_url,
        options_url,
        permissions,
    })
}
