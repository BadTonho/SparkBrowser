use super::{
    files,
    model::{BackupDocument, MAX_BYTES},
    service::BackupService,
};
use base64::Engine;
use std::{
    collections::HashMap,
    sync::Mutex,
    time::Duration,
};
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Url, WebviewUrl};

pub const GDRIVE_SYNC_WEBVIEW_LABEL: &str = "google-drive-sync-webview";
const GDRIVE_ENTRY_URL: &str = "https://drive.google.com/drive/my-drive";

pub struct PendingGDriveRequest {
    pub chunks: Vec<String>,
    pub sender: Option<tokio::sync::oneshot::Sender<Result<String, String>>>,
}

#[derive(Default)]
pub struct GDriveWebRuntime {
    pub bridge_token: Mutex<Option<String>>,
    pub page_status: Mutex<Option<String>>,
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

pub fn handle_bridge_navigation(app: &AppHandle, nav_url: &Url) {
    if nav_url.scheme() != "veloce-internal" || nav_url.host_str() != Some("google-drive-sync") {
        return;
    }
    let Some(params) = crate::security::bridge::query(
        nav_url,
        &["token", "request", "reason", "value", "data", "payload"],
    ) else {
        return;
    };
    let Some(service) = app.try_state::<BackupService>() else {
        return;
    };
    let token_valid = service
        .sync
        .gdrive
        .bridge_token
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .is_some_and(|expected| params.get("token") == Some(&expected));
    if !token_valid {
        return;
    }

    let action = nav_url.path().trim_matches('/');
    match action {
        "status" => {
            if let Some(reason) = params.get("reason") {
                if let Ok(mut status) = service.sync.gdrive.page_status.lock() {
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
            if let Ok(mut pending) = service.sync.gdrive.pending.lock() {
                if let Some(entry) = pending.get_mut(req_id) {
                    entry.chunks.push(payload.clone());
                }
            }
        }
        "complete" => {
            let Some(req_id) = params.get("request") else {
                return;
            };
            if let Ok(mut pending) = service.sync.gdrive.pending.lock() {
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
                        let _ = tx.send(Ok(final_payload));
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
            if let Ok(mut pending) = service.sync.gdrive.pending.lock() {
                if let Some(mut entry) = pending.remove(req_id) {
                    if let Some(tx) = entry.sender.take() {
                        let _ = tx.send(Err(err_msg));
                    }
                }
            }
        }
        _ => {}
    }
}

fn ensure_hidden_gdrive_webview(app: &AppHandle) -> Result<(tauri::Webview, String), String> {
    let service = app.state::<BackupService>();
    if let Some(existing) = app.get_webview(GDRIVE_SYNC_WEBVIEW_LABEL) {
        let token = service
            .sync
            .gdrive
            .bridge_token
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
            .unwrap_or_default();
        if !token.is_empty() {
            return Ok((existing, token));
        }
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

    let target_url: Url = GDRIVE_ENTRY_URL
        .parse()
        .map_err(|e: url::ParseError| e.to_string())?;
    let app_for_nav = app.clone();
    let builder = crate::window::create_webview_builder(
        GDRIVE_SYNC_WEBVIEW_LABEL,
        WebviewUrl::External(target_url),
    )
    .focused(false)
    .initialization_script(initializer_script(&token))
    .on_navigation(move |nav_url| {
        if nav_url.scheme() == "veloce-internal" {
            handle_bridge_navigation(&app_for_nav, nav_url);
            return false;
        }
        if nav_url.host_str() == Some("accounts.google.com") {
            if let Some(svc) = app_for_nav.try_state::<BackupService>() {
                if let Ok(mut status) = svc.sync.gdrive.page_status.lock() {
                    *status = Some("login_required".into());
                }
            }
        }
        is_allowed_gdrive_host(nav_url)
    });

    let webview = main_window
        .add_child(
            builder,
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(10.0, 10.0),
        )
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    let _ = webview.hide();
    Ok((webview, token))
}

async fn wait_until_webview_ready(app: &AppHandle) -> Result<(tauri::Webview, String), String> {
    let (webview, token) = ensure_hidden_gdrive_webview(app)?;
    let service = app.state::<BackupService>();

    for _ in 0..50 {
        let status = service
            .sync
            .gdrive
            .page_status
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        match status.as_deref() {
            Some("ready") => return Ok((webview, token)),
            Some("login_required") => {
                // Reload on next attempt in case user logs into Gmail in a tab
                let _ = webview.close();
                return Err("backup.sync_gdrive_login_required".into());
            }
            _ => tokio::time::sleep(Duration::from_millis(200)).await,
        }
    }
    Ok((webview, token))
}

pub async fn push_document_via_webview(
    app: &AppHandle,
    document: &BackupDocument,
) -> Result<(), String> {
    super::validation::document(document)?;
    let json_bytes = serde_json::to_vec_pretty(document).map_err(|e| e.to_string())?;
    if json_bytes.len() as u64 > MAX_BYTES {
        return Err("backup.file_too_large".into());
    }
    let b64_json = base64::engine::general_purpose::STANDARD.encode(&json_bytes);
    let (webview, token) = wait_until_webview_ready(app).await?;

    let request_id = crate::window::webview_manager::create_internal_bridge_token();
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
        "window.__veloceGDrivePush?.({}, {}, {});",
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
            "backup.sync_network_failed: Google Drive web session timed out".to_string()
        })?
        .map_err(|_| "backup.sync_network_failed".to_string())?;

    if let Err(err) = response {
        if err.starts_with("backup.sync_gdrive_login_required") {
            let _ = webview.close();
        }
        return Err(err);
    }
    Ok(())
}

pub async fn pull_document_via_webview(app: &AppHandle) -> Result<BackupDocument, String> {
    let (webview, token) = wait_until_webview_ready(app).await?;

    let request_id = crate::window::webview_manager::create_internal_bridge_token();
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
        "window.__veloceGDrivePull?.({}, {});",
        serde_json::to_string(&token).map_err(|e| e.to_string())?,
        serde_json::to_string(&request_id).map_err(|e| e.to_string())?,
    );
    webview
        .eval(&script)
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;

    let b64_payload = tokio::time::timeout(Duration::from_secs(25), rx)
        .await
        .map_err(|_| {
            if let Some(service) = app.try_state::<BackupService>() {
                if let Ok(mut pending) = service.sync.gdrive.pending.lock() {
                    pending.remove(&request_id);
                }
            }
            "backup.sync_network_failed: Google Drive web session timed out".to_string()
        })?
        .map_err(|_| "backup.sync_network_failed".to_string())??;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(b64_payload.as_bytes())
        .map_err(|e| format!("backup.invalid_document: {e}"))?;
    files::decode(&decoded_bytes)
}
