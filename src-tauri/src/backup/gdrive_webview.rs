#[path = "gdrive/auth.rs"]
mod auth;
#[path = "gdrive/session.rs"]
pub(crate) mod session;
pub use auth::{
    build_folder_metadata_json, build_sapisidhash_authorization, extract_api_keys_from_html,
    extract_email_from_text, sha1_hex,
};
use session::resolve_gdrive_session_context;

use super::{files, model::BackupDocument, service::BackupService};
use crate::sync::Mutex;
use crate::{database::preferences_repo, state::browser_state::BrowserState};
use base64::Engine;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tauri::{AppHandle, Manager, Url};
use tauri::{LogicalPosition, LogicalSize, WebviewUrl};

pub const GDRIVE_SYNC_WEBVIEW_LABEL: &str = "google-drive-sync-webview";
pub const SYNC_FOLDER_NAME: &str = "Spark Backup";
pub const LEGACY_SYNC_FOLDER_NAME: &str = "Veloce Browser Backup";
pub const FOLDER_MIME_TYPE: &str = "application/vnd.google-apps.folder";

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_request_id() -> String {
    format!("req-{}", NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed))
}

pub struct PendingGDriveRequest {
    pub chunks: Vec<String>,
    pub sender: Option<tokio::sync::oneshot::Sender<Result<String, String>>>,
}

#[derive(Default)]
pub struct GDriveWebRuntime {
    pub bridge_token: Mutex<Option<String>>,
    pub page_status: Mutex<Option<String>>,
    pub connected_email: Mutex<Option<String>>,
    pub active_authuser: Mutex<Option<String>>,
    pub cached_api_key: Mutex<Option<String>>,
    pub cached_folder_id: Mutex<Option<String>>,
    pub pending: Mutex<HashMap<String, PendingGDriveRequest>>,
}

pub fn initializer_script(token: &str) -> String {
    let encoded_token = serde_json::to_string(token).expect("token serializable");
    include_str!("gdrive_bridge.js").replace("__GDRIVE_BRIDGE_TOKEN__", &encoded_token)
}

pub fn is_allowed_gdrive_host(url: &Url) -> bool {
    if url.scheme() == "about" && url.path() == "blank" {
        return true;
    }
    if url.scheme() != "https" {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    let lower = host.to_lowercase();
    lower == "drive.google.com"
        || lower == "accounts.google.com"
        || lower.ends_with(".google.com")
        || lower.ends_with(".googleapis.com")
        || lower.ends_with(".gstatic.com")
        || lower.ends_with(".googleusercontent.com")
}

pub fn extract_authuser_from_url(raw_url: &str) -> Option<String> {
    let parsed = Url::parse(raw_url).ok()?;
    let host = parsed.host_str()?.to_lowercase();
    if host != "drive.google.com" {
        return None;
    }
    for (k, v) in parsed.query_pairs() {
        if k == "authuser" && !v.is_empty() && v.len() <= 4 && v.bytes().all(|b| b.is_ascii_digit())
        {
            return Some(v.into_owned());
        }
    }
    let segments: Vec<&str> = parsed.path().split('/').filter(|s| !s.is_empty()).collect();
    for window in segments.windows(2) {
        if window[0] == "u"
            && !window[1].is_empty()
            && window[1].len() <= 4
            && window[1].bytes().all(|b| b.is_ascii_digit())
        {
            return Some(window[1].to_string());
        }
    }
    if segments.first().copied() == Some("drive") {
        return Some("0".to_string());
    }
    None
}

fn sanitize_email(raw: Option<&String>) -> Option<String> {
    let trimmed = raw?.trim();
    if trimmed.len() >= 3 && trimmed.len() <= 254 && trimmed.contains('@') && !trimmed.contains(' ')
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

pub fn apply_bridge_navigation(runtime: &GDriveWebRuntime, nav_url: &Url) {
    if nav_url.scheme() != "spark-internal" || nav_url.host_str() != Some("google-drive-sync") {
        return;
    }
    let Some(params) = crate::security::bridge::query(
        nav_url,
        &["token", "request", "reason", "value", "data", "payload"],
    ) else {
        return;
    };
    let token_valid = runtime
        .bridge_token
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .is_some_and(|expected| params.get("token") == Some(&expected));
    if !token_valid {
        return;
    }

    if let Some(email) = sanitize_email(params.get("data")) {
        if let Ok(mut guard) = runtime.connected_email.lock() {
            *guard = Some(email);
        }
    }

    let action = nav_url.path().trim_matches('/');
    match action {
        "status" => {
            if let Some(reason) = params.get("reason") {
                if let Ok(mut status) = runtime.page_status.lock() {
                    *status = Some(reason.clone());
                }
            }
        }
        "chunk" => {
            let Some(req_id) = params.get("request") else {
                return;
            };
            let Some(payload) = params.get("payload") else {
                return;
            };
            if let Ok(mut pending) = runtime.pending.lock() {
                if let Some(entry) = pending.get_mut(req_id) {
                    entry.chunks.push(payload.clone());
                }
            }
        }
        "complete" => {
            let Some(req_id) = params.get("request") else {
                return;
            };
            if let Ok(mut pending) = runtime.pending.lock() {
                if let Some(mut entry) = pending.remove(req_id) {
                    let final_payload = if entry.chunks.is_empty() {
                        params.get("payload").cloned().unwrap_or_default()
                    } else {
                        if let Some(tail) = params.get("payload").filter(|s| !s.is_empty()) {
                            entry.chunks.push(tail.clone());
                        }
                        entry.chunks.concat()
                    };
                    if let Some(tx) = entry.sender.take() {
                        crate::error_log::ignored(tx.send(Ok(final_payload)));
                    }
                }
            }
        }
        "error" => {
            let Some(req_id) = params.get("request") else {
                return;
            };
            let err_msg = params
                .get("data")
                .cloned()
                .unwrap_or_else(|| "backup.sync_network_failed".into());
            if let Ok(mut pending) = runtime.pending.lock() {
                if let Some(mut entry) = pending.remove(req_id) {
                    if let Some(tx) = entry.sender.take() {
                        crate::error_log::ignored(tx.send(Err(err_msg)));
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn persist_detected_gdrive_email(app: &AppHandle) {
    let Some(service) = app.try_state::<BackupService>() else {
        return;
    };
    let email = service
        .sync
        .gdrive
        .connected_email
        .lock()
        .ok()
        .and_then(|g| g.clone());
    let Some(email) = email else {
        return;
    };
    let Some(state) = app.try_state::<BrowserState>() else {
        return;
    };
    let Ok(db) = state.infrastructure().db().lock() else {
        return;
    };
    if let Some(conn) = db.as_ref() {
        let _ =
            preferences_repo::store(conn, preferences_repo::BACKUP_SYNC_GDRIVE_EMAIL_KEY, &email);
    }
}

pub fn handle_bridge_navigation(app: &AppHandle, nav_url: &Url) {
    let Some(service) = app.try_state::<BackupService>() else {
        return;
    };
    apply_bridge_navigation(&service.sync.gdrive, nav_url);
    persist_detected_gdrive_email(app);
}

pub fn close_gdrive_sync_webview(app: &AppHandle) {
    if let Some(service) = app.try_state::<BackupService>() {
        if let Ok(mut status) = service.sync.gdrive.page_status.lock() {
            *status = None;
        }
        if let Ok(mut token) = service.sync.gdrive.bridge_token.lock() {
            *token = None;
        }
    }
    if let Some(webview) = app.get_webview(GDRIVE_SYNC_WEBVIEW_LABEL) {
        crate::error_log::ignored(webview.close());
    }
}

fn reset_gdrive_webview(app: &AppHandle, webview: &tauri::Webview) {
    if let Some(service) = app.try_state::<BackupService>() {
        if let Ok(mut status) = service.sync.gdrive.page_status.lock() {
            *status = None;
        }
        if let Ok(mut token) = service.sync.gdrive.bridge_token.lock() {
            *token = None;
        }
    }
    crate::error_log::ignored(webview.close());
}

fn ensure_hidden_gdrive_webview(app: &AppHandle) -> Result<(tauri::Webview, String), String> {
    let service = app.state::<BackupService>();
    let (data_dir, authuser, _) = resolve_gdrive_session_context(app)?;

    if let Some(existing) = app.get_webview(GDRIVE_SYNC_WEBVIEW_LABEL) {
        crate::error_log::ignored(existing.close());
    }

    let main_window = app
        .get_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;
    let token = crate::window::webview_manager::create_internal_bridge_token();
    *service
        .sync
        .gdrive
        .bridge_token
        .lock()
        .map_err(|e| e.to_string())? = Some(token.clone());
    *service
        .sync
        .gdrive
        .page_status
        .lock()
        .map_err(|e| e.to_string())? = None;
    *service
        .sync
        .gdrive
        .active_authuser
        .lock()
        .map_err(|e| e.to_string())? = Some(authuser.clone());

    let entry_url = format!("https://drive.google.com/drive/u/{authuser}/my-drive");
    let target_url: Url = entry_url
        .parse()
        .map_err(|e: url::ParseError| e.to_string())?;
    let app_for_nav = app.clone();
    let builder = crate::window::create_webview_builder(
        GDRIVE_SYNC_WEBVIEW_LABEL,
        WebviewUrl::External(target_url),
    )
    .focused(false)
    .initialization_script_for_all_frames(initializer_script(&token))
    .on_navigation(move |nav_url| {
        if nav_url.scheme() == "spark-internal" {
            handle_bridge_navigation(&app_for_nav, nav_url);
            return false;
        }
        is_allowed_gdrive_host(nav_url)
    });

    let builder = match data_dir {
        Some(dir) => builder.data_directory(dir),
        None => builder,
    };

    let webview = main_window
        .add_child(
            builder,
            LogicalPosition::new(-10000.0, -10000.0),
            LogicalSize::new(1.0, 1.0),
        )
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    crate::error_log::ignored(crate::security::harden(app, &webview, false));
    Ok((webview, token))
}

async fn wait_until_webview_ready(app: &AppHandle) -> Result<(tauri::Webview, String), String> {
    let (webview, token) = ensure_hidden_gdrive_webview(app)?;
    let service = app.state::<BackupService>();

    for attempt in 0..75 {
        let status = service
            .sync
            .gdrive
            .page_status
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        match status.as_deref() {
            Some("ready") => {
                let email = service
                    .sync
                    .gdrive
                    .connected_email
                    .lock()
                    .map_err(|_| session::login("state-unavailable"))?
                    .clone();
                if let Err(error) = session::verify_email(app, email.as_deref()) {
                    reset_gdrive_webview(app, &webview);
                    return Err(error);
                }
                persist_detected_gdrive_email(app);
                return Ok((webview, token));
            }
            Some("login_required") => {
                reset_gdrive_webview(app, &webview);
                return Err(session::login("hidden-page-shows-sign-in"));
            }
            _ => {
                if attempt > 0 && attempt % 10 == 0 {
                    if let Ok(encoded_token) = serde_json::to_string(&token) {
                        let _ =
                            webview.eval(format!("window.__sparkGDrivePing?.({encoded_token});"));
                    }
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
    reset_gdrive_webview(app, &webview);
    Err("backup.sync_network_failed: Google Drive page did not finish loading".into())
}

pub async fn push_document_via_webview(
    app: &AppHandle,
    document: &BackupDocument,
    password: Option<&str>,
) -> Result<(), String> {
    let json_bytes = files::encode(document, password)?;
    let b64_json = base64::engine::general_purpose::STANDARD.encode(&json_bytes);
    let (webview, token) = wait_until_webview_ready(app).await?;

    let request_id = next_request_id();
    let (tx, rx) = tokio::sync::oneshot::channel();
    {
        let service = app.state::<BackupService>();
        service
            .sync
            .gdrive
            .pending
            .lock()
            .map_err(|e| e.to_string())?
            .insert(
                request_id.clone(),
                PendingGDriveRequest {
                    chunks: Vec::new(),
                    sender: Some(tx),
                },
            );
    }

    let script = format!(
        "window.__sparkGDrivePush?.({}, {}, {});",
        serde_json::to_string(&token).map_err(|e| e.to_string())?,
        serde_json::to_string(&request_id).map_err(|e| e.to_string())?,
        serde_json::to_string(&b64_json).map_err(|e| e.to_string())?,
    );
    webview
        .eval(&script)
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;

    let response = tokio::time::timeout(Duration::from_secs(25), rx)
        .await
        .map_err(|_| {
            if let Some(service) = app.try_state::<BackupService>() {
                if let Ok(mut pending) = service.sync.gdrive.pending.lock() {
                    pending.remove(&request_id);
                }
            }
            reset_gdrive_webview(app, &webview);
            "backup.sync_network_failed: Google Drive web session timed out".to_string()
        })?
        .map_err(|_| "backup.sync_network_failed".to_string())?;

    reset_gdrive_webview(app, &webview);
    response?;
    persist_detected_gdrive_email(app);
    Ok(())
}

/// Runs one bridge call in the hidden Drive page and returns its payload (base64 text). `call` builds
/// the script from the encoded token and request id.
async fn bridge_payload(
    app: &AppHandle,
    call: impl FnOnce(&str, &str) -> String,
) -> Result<String, String> {
    let (webview, token) = wait_until_webview_ready(app).await?;
    let request_id = next_request_id();
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.state::<BackupService>()
        .sync
        .gdrive
        .pending
        .lock()
        .map_err(|e| e.to_string())?
        .insert(
            request_id.clone(),
            PendingGDriveRequest {
                chunks: Vec::new(),
                sender: Some(tx),
            },
        );
    let script = call(
        &serde_json::to_string(&token).map_err(|e| e.to_string())?,
        &serde_json::to_string(&request_id).map_err(|e| e.to_string())?,
    );
    webview
        .eval(&script)
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    let response = tokio::time::timeout(Duration::from_secs(25), rx)
        .await
        .map_err(|_| {
            if let Some(service) = app.try_state::<BackupService>() {
                if let Ok(mut pending) = service.sync.gdrive.pending.lock() {
                    pending.remove(&request_id);
                }
            }
            reset_gdrive_webview(app, &webview);
            "backup.sync_network_failed: Google Drive web session timed out".to_string()
        })?
        .map_err(|_| "backup.sync_network_failed".to_string())?;
    reset_gdrive_webview(app, &webview);
    let payload = response?;
    persist_detected_gdrive_email(app);
    Ok(payload)
}

fn decode_payload(b64_payload: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(b64_payload.as_bytes())
        .map_err(|e| format!("backup.invalid_document: {e}"))
}

/// The earlier versions Drive keeps of the sync file, newest first.
pub async fn list_versions_via_webview(
    app: &AppHandle,
) -> Result<Vec<super::sync_versions::SyncVersion>, String> {
    let payload = bridge_payload(app, |token, request| {
        format!("window.__sparkGDriveVersions?.({token}, {request});")
    })
    .await?;
    #[derive(serde::Deserialize)]
    struct Revision {
        id: String,
        created_at: i64,
        size: Option<u64>,
    }
    let revisions: Vec<Revision> = serde_json::from_slice(&decode_payload(&payload)?)
        .map_err(|e| format!("backup.sync_versions_failed: {e}"))?;
    Ok(revisions
        .into_iter()
        .filter(|rev| super::sync_versions::valid_revision_id(&rev.id))
        .map(|rev| super::sync_versions::SyncVersion {
            id: rev.id,
            created_at: rev.created_at,
            size: rev.size,
        })
        .collect())
}

/// Downloads one earlier version (a Drive revision) of the sync file.
pub async fn pull_version_via_webview(
    app: &AppHandle,
    revision_id: &str,
    password: Option<&str>,
) -> Result<BackupDocument, String> {
    if !super::sync_versions::valid_revision_id(revision_id) {
        return Err("backup.sync_version_not_found".into());
    }
    let revision = serde_json::to_string(revision_id).map_err(|e| e.to_string())?;
    let payload = bridge_payload(app, |token, request| {
        format!("window.__sparkGDriveVersion?.({token}, {request}, {revision});")
    })
    .await?;
    files::decode_with(&decode_payload(&payload)?, password)
}

pub async fn pull_document_via_webview(
    app: &AppHandle,
    password: Option<&str>,
) -> Result<BackupDocument, String> {
    let (webview, token) = wait_until_webview_ready(app).await?;

    let request_id = next_request_id();
    let (tx, rx) = tokio::sync::oneshot::channel();
    {
        let service = app.state::<BackupService>();
        service
            .sync
            .gdrive
            .pending
            .lock()
            .map_err(|e| e.to_string())?
            .insert(
                request_id.clone(),
                PendingGDriveRequest {
                    chunks: Vec::new(),
                    sender: Some(tx),
                },
            );
    }

    let script = format!(
        "window.__sparkGDrivePull?.({}, {});",
        serde_json::to_string(&token).map_err(|e| e.to_string())?,
        serde_json::to_string(&request_id).map_err(|e| e.to_string())?,
    );
    webview
        .eval(&script)
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;

    let response = tokio::time::timeout(Duration::from_secs(25), rx)
        .await
        .map_err(|_| {
            if let Some(service) = app.try_state::<BackupService>() {
                if let Ok(mut pending) = service.sync.gdrive.pending.lock() {
                    pending.remove(&request_id);
                }
            }
            reset_gdrive_webview(app, &webview);
            "backup.sync_network_failed: Google Drive web session timed out".to_string()
        })?
        .map_err(|_| "backup.sync_network_failed".to_string())?;

    reset_gdrive_webview(app, &webview);
    let b64_payload = match response {
        Ok(payload) => payload,
        Err(err) => {
            return Err(err);
        }
    };

    persist_detected_gdrive_email(app);
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(b64_payload.as_bytes())
        .map_err(|e| format!("backup.invalid_document: {e}"))?;
    files::decode_with(&decoded_bytes, password)
}
