use super::{
    encryption, files, gdrive_webview,
    model::{BackupDocument, BackupImportResult, BackupPreferences, BackupPreview},
    service,
};
use crate::sync::Mutex;
use crate::{database::preferences_repo, state::browser_state::BrowserState};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
use tauri::{AppHandle, Manager};

pub const SYNC_FILE_NAME: &str = "browser.spark-backup.json";
/// Name used before the rename. Pulls fall back to it so an existing backup is not orphaned.
pub const LEGACY_SYNC_FILE_NAME: &str = "browser.veloce-backup.json";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum SyncTargetConfig {
    #[default]
    Disabled,
    GoogleDriveWeb {
        auto_save: bool,
        #[serde(default)]
        apply_preferences_on_direct_pull: bool,
    },
    Folder {
        directory_path: String,
        auto_save: bool,
        #[serde(default)]
        apply_preferences_on_direct_pull: bool,
    },
    DirectWebdav {
        endpoint_url: String,
        username: String,
        remember_secret: bool,
        auto_save: bool,
        #[serde(default)]
        apply_preferences_on_direct_pull: bool,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct SyncStateSnapshot {
    pub config: SyncTargetConfig,
    pub has_secret: bool,
    pub connected_account: Option<String>,
    pub last_synced_at: Option<i64>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SyncPushResult {
    pub synced: bool,
    pub skipped_unchanged: bool,
    pub last_synced_at: Option<i64>,
    pub state: SyncStateSnapshot,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncPullResponse {
    Preview {
        source_token: String,
        preview: Box<BackupPreview>,
        state: SyncStateSnapshot,
    },
    Imported {
        result: Box<BackupImportResult>,
        state: SyncStateSnapshot,
    },
}

#[derive(Default)]
pub struct SyncRuntime {
    pub credentials: std::sync::Arc<super::credentials::Credentials>,
    pub session_secret: Mutex<Option<String>>,
    pub last_error: Mutex<Option<String>>,
    pub gdrive: gdrive_webview::GDriveWebRuntime,
}

pub fn fingerprint_hex(doc: &BackupDocument) -> Result<String, String> {
    let bytes = service::fingerprint(&doc.data, &doc.preferences)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

pub fn validate_webdav_url(raw: &str) -> Result<url::Url, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 4096 {
        return Err("backup.sync_invalid_url".into());
    }
    let parsed = url::Url::parse(trimmed).map_err(|_| "backup.sync_invalid_url".to_string())?;
    if !parsed.username().is_empty() || parsed.password().is_some() || parsed.host_str().is_none() {
        return Err("backup.sync_invalid_url".into());
    }
    match parsed.scheme() {
        "https" => Ok(parsed),
        "http" => {
            let host = parsed.host_str().unwrap_or_default();
            if matches!(host, "127.0.0.1" | "localhost" | "[::1]") {
                Ok(parsed)
            } else {
                Err("backup.sync_https_required".into())
            }
        }
        _ => Err("backup.sync_invalid_url".into()),
    }
}

pub fn resolve_webdav_file_url(raw: &str) -> Result<url::Url, String> {
    let mut url = validate_webdav_url(raw)?;
    let path = url.path().to_string();
    let lower = path.to_lowercase();
    if lower.ends_with(".spark-backup.json") || lower.ends_with(".veloce-backup.json") {
        return Ok(url);
    }
    let normalized = if path.ends_with('/') {
        format!("{path}{SYNC_FILE_NAME}")
    } else {
        format!("{path}/{SYNC_FILE_NAME}")
    };
    url.set_path(&normalized);
    Ok(url)
}

pub fn resolve_folder_file_path(directory_path: &str) -> Result<PathBuf, String> {
    let trimmed = directory_path.trim();
    if trimmed.is_empty() {
        return Err("backup.sync_folder_missing".into());
    }
    let dir = Path::new(trimmed);
    if !dir.is_dir() {
        return Err("backup.sync_folder_missing".into());
    }
    Ok(dir.join(SYNC_FILE_NAME))
}

/// The folder file to read: the current name, or the old one when only that exists.
pub fn resolve_folder_pull_path(directory_path: &str) -> Result<PathBuf, String> {
    let current = resolve_folder_file_path(directory_path)?;
    let legacy = current.with_file_name(LEGACY_SYNC_FILE_NAME);
    Ok(if !current.is_file() && legacy.is_file() {
        legacy
    } else {
        current
    })
}

/// The pre-rename twin of a directory-derived WebDAV file URL, if it is one.
fn legacy_webdav_file_url(url: &url::Url) -> Option<url::Url> {
    let path = url.path();
    let base = path.strip_suffix(SYNC_FILE_NAME)?;
    let mut legacy = url.clone();
    legacy.set_path(&format!("{base}{LEGACY_SYNC_FILE_NAME}"));
    Some(legacy)
}

#[cfg(test)]
pub(super) fn legacy_webdav_file_url_for_test(url: &url::Url) -> Option<url::Url> {
    legacy_webdav_file_url(url)
}

pub fn load_config(conn: &Connection) -> Result<SyncTargetConfig, String> {
    let Some(raw) = preferences_repo::load(conn, preferences_repo::BACKUP_SYNC_CONFIG_KEY)? else {
        return Ok(SyncTargetConfig::Disabled);
    };
    serde_json::from_str(&raw).map_err(|_| "backup.sync_invalid_config".into())
}

pub fn resolve_secret(conn: &Connection, runtime: &SyncRuntime) -> Result<Option<String>, String> {
    // Remove legacy plaintext even if the OS store is unavailable. Keep it only
    // in this process so a locked keyring never causes another plaintext write.
    if let Some(legacy) = preferences_repo::load(conn, preferences_repo::BACKUP_SYNC_SECRET_KEY)? {
        *runtime.session_secret.lock().map_err(|e| e.to_string())? = Some(legacy.clone());
        remove_legacy_secret(conn)?;
        runtime
            .credentials
            .write(&super::credentials::key(conn), &legacy)?;
        preferences_repo::store(conn, preferences_repo::BACKUP_SYNC_CREDENTIAL_KEY, "true")?;
    }
    if let Some(secret) = runtime
        .session_secret
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
    {
        if !secret.is_empty() {
            return Ok(Some(secret));
        }
    }
    if preferences_repo::load_bool(conn, preferences_repo::BACKUP_SYNC_CREDENTIAL_KEY)? {
        runtime.credentials.read(&super::credentials::key(conn))
    } else {
        Ok(None)
    }
}

fn forget_secret(conn: &Connection, runtime: &SyncRuntime) -> Result<(), String> {
    if preferences_repo::load_bool(conn, preferences_repo::BACKUP_SYNC_CREDENTIAL_KEY)? {
        runtime.credentials.delete(&super::credentials::key(conn))?;
    }
    preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_CREDENTIAL_KEY)?;
    remove_legacy_secret(conn)
}

fn remove_legacy_secret(conn: &Connection) -> Result<(), String> {
    if preferences_repo::load(conn, preferences_repo::BACKUP_SYNC_SECRET_KEY)?.is_none() {
        return Ok(());
    }
    conn.pragma_update(None, "secure_delete", true)
        .map_err(|e| e.to_string())?;
    preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_SECRET_KEY)?;
    // Startup migration owns the connection: overwrite the deleted cell in the
    // main database and discard historical WAL frames before accepting tabs.
    if conn.is_autocommit() {
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn snapshot(conn: &Connection, runtime: &SyncRuntime) -> Result<SyncStateSnapshot, String> {
    let config = load_config(conn).unwrap_or(SyncTargetConfig::Disabled);
    let has_secret = match &config {
        SyncTargetConfig::DirectWebdav { .. } => {
            runtime
                .session_secret
                .lock()
                .map_err(|e| e.to_string())?
                .is_some()
                || preferences_repo::load_bool(conn, preferences_repo::BACKUP_SYNC_CREDENTIAL_KEY)?
        }
        _ => false,
    };
    let connected_account = match &config {
        // A remembered e-mail without the link (a profile saved by an older build) is not a
        // connection: showing it would hide that the account still has to be chosen once.
        SyncTargetConfig::GoogleDriveWeb { .. } if !gdrive_webview::session::is_bound(conn) => None,
        SyncTargetConfig::GoogleDriveWeb { .. } => runtime
            .gdrive
            .connected_email
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .or_else(|| {
                preferences_repo::load(conn, preferences_repo::BACKUP_SYNC_GDRIVE_EMAIL_KEY)
                    .ok()
                    .flatten()
            })
            .filter(|s| !s.is_empty()),
        _ => None,
    };
    let last_synced_at = preferences_repo::load(conn, preferences_repo::BACKUP_SYNC_LAST_AT_KEY)?
        .and_then(|v| v.parse::<i64>().ok());
    let last_error = runtime
        .last_error
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    Ok(SyncStateSnapshot {
        config,
        has_secret,
        connected_account,
        last_synced_at,
        last_error,
    })
}

pub fn save_config(
    conn: &Connection,
    runtime: &SyncRuntime,
    config: SyncTargetConfig,
    secret: Option<String>,
) -> Result<SyncStateSnapshot, String> {
    match &config {
        SyncTargetConfig::Disabled => {
            gdrive_webview::session::clear(conn)?;
            *runtime.session_secret.lock().map_err(|e| e.to_string())? = None;
            if let Ok(mut email) = runtime.gdrive.connected_email.lock() {
                *email = None;
            }
            forget_secret(conn, runtime)?;
            preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_FINGERPRINT_KEY)?;
            preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_GDRIVE_EMAIL_KEY)?;
            preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_GDRIVE_AUTHUSER_KEY)?;
        }
        SyncTargetConfig::GoogleDriveWeb { .. } => {
            *runtime.session_secret.lock().map_err(|e| e.to_string())? = None;
            forget_secret(conn, runtime)?;
            preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_FINGERPRINT_KEY)?;
        }
        SyncTargetConfig::Folder { directory_path, .. } => {
            let _ = resolve_folder_file_path(directory_path)?;
            *runtime.session_secret.lock().map_err(|e| e.to_string())? = None;
            forget_secret(conn, runtime)?;
            preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_FINGERPRINT_KEY)?;
        }
        SyncTargetConfig::DirectWebdav {
            endpoint_url,
            username,
            remember_secret,
            ..
        } => {
            let _ = resolve_webdav_file_url(endpoint_url)?;
            if username.chars().count() > 256 {
                return Err("backup.sync_invalid_config".into());
            }
            let same_target = matches!(load_config(conn)?, SyncTargetConfig::DirectWebdav { endpoint_url: old_url, username: old_user, .. } if old_url == *endpoint_url && old_user == *username);
            let effective_secret = match secret {
                Some(value) if !value.is_empty() => Some(value),
                _ if same_target => resolve_secret(conn, runtime)?,
                _ => None,
            };
            if let Some(secret_value) = effective_secret {
                if secret_value.len() > 4096 {
                    return Err("backup.sync_invalid_config".into());
                }
                if *remember_secret {
                    runtime
                        .credentials
                        .write(&super::credentials::key(conn), &secret_value)?;
                    preferences_repo::store(
                        conn,
                        preferences_repo::BACKUP_SYNC_CREDENTIAL_KEY,
                        "true",
                    )?;
                } else {
                    forget_secret(conn, runtime)?;
                }
                *runtime.session_secret.lock().map_err(|e| e.to_string())? = Some(secret_value);
            } else {
                forget_secret(conn, runtime)?;
                *runtime.session_secret.lock().map_err(|e| e.to_string())? = None;
            }
            preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_FINGERPRINT_KEY)?;
        }
    }

    let serialized = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    preferences_repo::store(conn, preferences_repo::BACKUP_SYNC_CONFIG_KEY, &serialized)?;
    *runtime.last_error.lock().map_err(|e| e.to_string())? = None;
    snapshot(conn, runtime)
}

fn record_error(runtime: &SyncRuntime, err: &str) {
    if let Ok(mut guard) = runtime.last_error.lock() {
        *guard = Some(err.to_string());
    }
}

fn clear_error_and_record_sync(
    conn: &Connection,
    runtime: &SyncRuntime,
    fingerprint: &str,
    timestamp: i64,
) -> Result<(), String> {
    preferences_repo::store(
        conn,
        preferences_repo::BACKUP_SYNC_FINGERPRINT_KEY,
        fingerprint,
    )?;
    preferences_repo::store(
        conn,
        preferences_repo::BACKUP_SYNC_LAST_AT_KEY,
        &timestamp.to_string(),
    )?;
    if let Ok(mut guard) = runtime.last_error.lock() {
        *guard = None;
    }
    Ok(())
}

pub fn push_document_to_target(
    conn: &Connection,
    runtime: &SyncRuntime,
    document: &BackupDocument,
    only_if_changed: bool,
) -> Result<SyncPushResult, String> {
    let config = load_config(conn)?;
    if config == SyncTargetConfig::Disabled {
        return Err("backup.sync_not_configured".into());
    }

    let current_fp = fingerprint_hex(document)?;
    let previous_fp = preferences_repo::load(conn, preferences_repo::BACKUP_SYNC_FINGERPRINT_KEY)?;

    let target_exists = match &config {
        SyncTargetConfig::Folder { directory_path, .. } => resolve_folder_file_path(directory_path)
            .map(|p| p.is_file())
            .unwrap_or(false),
        SyncTargetConfig::DirectWebdav { .. } | SyncTargetConfig::GoogleDriveWeb { .. } => true,
        SyncTargetConfig::Disabled => false,
    };

    if only_if_changed && target_exists && previous_fp.as_deref() == Some(current_fp.as_str()) {
        let state = snapshot(conn, runtime)?;
        return Ok(SyncPushResult {
            synced: false,
            skipped_unchanged: true,
            last_synced_at: state.last_synced_at,
            state,
        });
    }

    let password = encryption::write_password(conn, &runtime.credentials)?;
    let write_result = match &config {
        SyncTargetConfig::Disabled | SyncTargetConfig::GoogleDriveWeb { .. } => {
            Err("backup.sync_not_configured".to_string())
        }
        SyncTargetConfig::Folder { directory_path, .. } => resolve_folder_file_path(directory_path)
            .and_then(|file_path| files::write_with(&file_path, document, password.as_deref())),
        SyncTargetConfig::DirectWebdav {
            endpoint_url,
            username,
            ..
        } => {
            let secret = resolve_secret(conn, runtime)?;
            webdav_put(
                endpoint_url,
                username,
                secret.as_deref(),
                document,
                password.as_deref(),
            )
        }
    };

    if let Err(err) = write_result {
        record_error(runtime, &err);
        return Err(err);
    }

    let now = crate::state::tab_model::TabInfo::current_epoch_secs() as i64;
    clear_error_and_record_sync(conn, runtime, &current_fp, now)?;
    let state = snapshot(conn, runtime)?;
    Ok(SyncPushResult {
        synced: true,
        skipped_unchanged: false,
        last_synced_at: Some(now),
        state,
    })
}

pub fn pull_document_from_target(
    conn: &Connection,
    runtime: &SyncRuntime,
) -> Result<BackupDocument, String> {
    let config = load_config(conn)?;
    let password = encryption::read_password(conn, &runtime.credentials);
    let read_result = match &config {
        SyncTargetConfig::Disabled | SyncTargetConfig::GoogleDriveWeb { .. } => {
            Err("backup.sync_not_configured".to_string())
        }
        SyncTargetConfig::Folder { directory_path, .. } => {
            let file_path = resolve_folder_pull_path(directory_path)?;
            if !file_path.is_file() {
                Err("backup.sync_file_not_found".to_string())
            } else {
                files::read_with(&file_path, password.as_deref())
            }
        }
        SyncTargetConfig::DirectWebdav {
            endpoint_url,
            username,
            ..
        } => {
            let secret = resolve_secret(conn, runtime)?;
            webdav_get(
                endpoint_url,
                username,
                secret.as_deref(),
                password.as_deref(),
            )
        }
    };

    match read_result {
        Ok(doc) => {
            if let Ok(mut guard) = runtime.last_error.lock() {
                *guard = None;
            }
            Ok(doc)
        }
        Err(err) => {
            record_error(runtime, &err);
            Err(err)
        }
    }
}

fn build_http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))
}

fn apply_auth(
    request: reqwest::blocking::RequestBuilder,
    username: &str,
    secret: Option<&str>,
) -> reqwest::blocking::RequestBuilder {
    let user = username.trim();
    match (user.is_empty(), secret.filter(|s| !s.is_empty())) {
        (false, Some(pass)) => request.basic_auth(user, Some(pass)),
        (false, None) => request.basic_auth(user, None::<&str>),
        (true, Some(token)) => request.bearer_auth(token),
        (true, None) => request,
    }
}

pub fn webdav_put(
    endpoint_url: &str,
    username: &str,
    secret: Option<&str>,
    document: &BackupDocument,
    password: Option<&str>,
) -> Result<(), String> {
    let bytes = files::encode(document, password)?;
    let url = resolve_webdav_file_url(endpoint_url)?;
    let client = build_http_client()?;
    let request = apply_auth(
        client
            .put(url)
            .header("Content-Type", "application/json")
            .body(bytes),
        username,
        secret,
    );
    let response = request
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
        return Err("backup.sync_auth_failed".into());
    }
    if !response.status().is_success() {
        return Err(format!(
            "backup.sync_network_failed: HTTP {}",
            response.status().as_u16()
        ));
    }
    Ok(())
}

pub fn webdav_get(
    endpoint_url: &str,
    username: &str,
    secret: Option<&str>,
    password: Option<&str>,
) -> Result<BackupDocument, String> {
    let url = resolve_webdav_file_url(endpoint_url)?;
    match webdav_get_url(url.clone(), username, secret, password) {
        Err(error) if error == "backup.sync_file_not_found" => match legacy_webdav_file_url(&url) {
            Some(legacy) => webdav_get_url(legacy, username, secret, password),
            None => Err(error),
        },
        other => other,
    }
}

fn webdav_get_url(
    url: url::Url,
    username: &str,
    secret: Option<&str>,
    password: Option<&str>,
) -> Result<BackupDocument, String> {
    let client = build_http_client()?;
    let request = apply_auth(
        client.get(url).header("Accept", "application/json"),
        username,
        secret,
    );
    let response = request
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
        return Err("backup.sync_auth_failed".into());
    }
    if response.status().as_u16() == 404 {
        return Err("backup.sync_file_not_found".into());
    }
    if !response.status().is_success() {
        return Err(format!(
            "backup.sync_network_failed: HTTP {}",
            response.status().as_u16()
        ));
    }
    let mut bytes = Vec::new();
    response
        .take(files::MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    files::decode_with(&bytes, password)
}

pub async fn push_sync(
    app: &AppHandle,
    preferences: BackupPreferences,
    only_if_changed: bool,
) -> Result<SyncPushResult, String> {
    let service = app.state::<service::BackupService>();
    let _serial = service.operation.lock().await;
    let document = service::capture(app, preferences).await?;
    let current_fp = fingerprint_hex(&document)?;

    let (config, previous_fp, password) = {
        let state = app.state::<BrowserState>();
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("backup.database_unavailable")?;
        let cfg = load_config(conn)?;
        let prev = preferences_repo::load(conn, preferences_repo::BACKUP_SYNC_FINGERPRINT_KEY)?;
        let password = encryption::write_password(conn, &service.sync.credentials)?;
        (cfg, prev, password)
    };

    if config == SyncTargetConfig::Disabled {
        return Err("backup.sync_not_configured".into());
    }

    let target_exists = match &config {
        SyncTargetConfig::Folder { directory_path, .. } => resolve_folder_file_path(directory_path)
            .map(|p| p.is_file())
            .unwrap_or(false),
        SyncTargetConfig::DirectWebdav { .. } | SyncTargetConfig::GoogleDriveWeb { .. } => true,
        SyncTargetConfig::Disabled => false,
    };

    if only_if_changed && target_exists && previous_fp.as_deref() == Some(current_fp.as_str()) {
        let state = app.state::<BrowserState>();
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("backup.database_unavailable")?;
        let snap = snapshot(conn, &service.sync)?;
        return Ok(SyncPushResult {
            synced: false,
            skipped_unchanged: true,
            last_synced_at: snap.last_synced_at,
            state: snap,
        });
    }

    let io_result = match &config {
        SyncTargetConfig::Disabled => Err("backup.sync_not_configured".to_string()),
        SyncTargetConfig::GoogleDriveWeb { .. } => {
            gdrive_webview::push_document_via_webview(app, &document, password.as_deref()).await
        }
        SyncTargetConfig::Folder { directory_path, .. } => {
            let dir = directory_path.clone();
            let doc = document.clone();
            let password = password.clone();
            tauri::async_runtime::spawn_blocking(move || {
                resolve_folder_file_path(&dir)
                    .and_then(|file_path| files::write_with(&file_path, &doc, password.as_deref()))
            })
            .await
            .map_err(|e| e.to_string())?
        }
        SyncTargetConfig::DirectWebdav {
            endpoint_url,
            username,
            ..
        } => {
            let secret = {
                let state = app.state::<BrowserState>();
                let db = state
                    .infrastructure()
                    .db()
                    .lock()
                    .map_err(|e| e.to_string())?;
                let conn = db.as_ref().ok_or("backup.database_unavailable")?;
                resolve_secret(conn, &service.sync)?
            };
            let url = endpoint_url.clone();
            let user = username.clone();
            let doc = document.clone();
            let password = password.clone();
            tauri::async_runtime::spawn_blocking(move || {
                webdav_put(&url, &user, secret.as_deref(), &doc, password.as_deref())
            })
            .await
            .map_err(|e| e.to_string())?
        }
    };

    let state = app.state::<BrowserState>();
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("backup.database_unavailable")?;
    if let Err(err) = io_result {
        record_error(&service.sync, &err);
        return Err(err);
    }
    let now = crate::state::tab_model::TabInfo::current_epoch_secs() as i64;
    clear_error_and_record_sync(conn, &service.sync, &current_fp, now)?;
    let snap = snapshot(conn, &service.sync)?;
    Ok(SyncPushResult {
        synced: true,
        skipped_unchanged: false,
        last_synced_at: Some(now),
        state: snap,
    })
}

pub async fn pull_sync(
    app: &AppHandle,
    current_preferences: BackupPreferences,
    direct_apply: bool,
    apply_preferences_override: Option<bool>,
) -> Result<SyncPullResponse, String> {
    let backup_service = app.state::<service::BackupService>();
    let (config, secret) = {
        let state = app.state::<BrowserState>();
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("backup.database_unavailable")?;
        let cfg = load_config(conn)?;
        let sec = resolve_secret(conn, &backup_service.sync)?;
        (cfg, sec)
    };
    let password = {
        let state = app.state::<BrowserState>();
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("backup.database_unavailable")?;
        encryption::read_password(conn, &backup_service.sync.credentials)
    };

    let config_apply_prefs = match &config {
        SyncTargetConfig::GoogleDriveWeb {
            apply_preferences_on_direct_pull,
            ..
        }
        | SyncTargetConfig::Folder {
            apply_preferences_on_direct_pull,
            ..
        }
        | SyncTargetConfig::DirectWebdav {
            apply_preferences_on_direct_pull,
            ..
        } => *apply_preferences_on_direct_pull,
        SyncTargetConfig::Disabled => false,
    };

    let read_res = match &config {
        SyncTargetConfig::Disabled => Err("backup.sync_not_configured".to_string()),
        SyncTargetConfig::GoogleDriveWeb { .. } => {
            gdrive_webview::pull_document_via_webview(app, password.as_deref()).await
        }
        SyncTargetConfig::Folder { directory_path, .. } => {
            let dir = directory_path.clone();
            let password = password.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let file_path = resolve_folder_file_path(&dir)?;
                if !file_path.is_file() {
                    Err("backup.sync_file_not_found".to_string())
                } else {
                    files::read_with(&file_path, password.as_deref())
                }
            })
            .await
            .map_err(|e| e.to_string())?
        }
        SyncTargetConfig::DirectWebdav {
            endpoint_url,
            username,
            ..
        } => {
            let url = endpoint_url.clone();
            let user = username.clone();
            let password = password.clone();
            tauri::async_runtime::spawn_blocking(move || {
                webdav_get(&url, &user, secret.as_deref(), password.as_deref())
            })
            .await
            .map_err(|e| e.to_string())?
        }
    };

    let doc = match read_res {
        Ok(doc) => {
            if let Ok(mut guard) = backup_service.sync.last_error.lock() {
                *guard = None;
            }
            doc
        }
        Err(err) => {
            record_error(&backup_service.sync, &err);
            return Err(err);
        }
    };

    let source_token = backup_service.select(doc)?;
    let preview = service::preview(
        app,
        None,
        Some(source_token.clone()),
        current_preferences.clone(),
    )
    .await?;

    if !direct_apply {
        let state = app.state::<BrowserState>();
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("backup.database_unavailable")?;
        let snap = snapshot(conn, &backup_service.sync)?;
        return Ok(SyncPullResponse::Preview {
            source_token,
            preview: Box::new(preview),
            state: snap,
        });
    }

    let apply_prefs = apply_preferences_override.unwrap_or(config_apply_prefs);
    let result =
        service::import(app, preview.token, apply_prefs, current_preferences.clone()).await?;
    let synced_doc = service::capture(app, current_preferences).await?;
    let current_fp = fingerprint_hex(&synced_doc)?;
    let now = crate::state::tab_model::TabInfo::current_epoch_secs() as i64;
    let state = app.state::<BrowserState>();
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("backup.database_unavailable")?;
    clear_error_and_record_sync(conn, &backup_service.sync, &current_fp, now)?;
    let snap = snapshot(conn, &backup_service.sync)?;
    Ok(SyncPullResponse::Imported {
        result: Box::new(result),
        state: snap,
    })
}
