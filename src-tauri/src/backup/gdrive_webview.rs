use super::{
    files,
    model::{BackupDocument, MAX_BYTES},
    service::BackupService,
};
use base64::Engine;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Url, WebviewUrl};

pub const GDRIVE_SYNC_WEBVIEW_LABEL: &str = "google-drive-sync-webview";
const GDRIVE_ENTRY_URL: &str = "https://drive.google.com/drive/my-drive";

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

pub fn apply_bridge_navigation(runtime: &GDriveWebRuntime, nav_url: &Url) {
    if nav_url.scheme() != "veloce-internal" || nav_url.host_str() != Some("google-drive-sync") {
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
            if let Ok(mut pending) = runtime.pending.lock() {
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

pub fn handle_bridge_navigation(app: &AppHandle, nav_url: &Url) {
    let Some(service) = app.try_state::<BackupService>() else {
        return;
    };
    apply_bridge_navigation(&service.sync.gdrive, nav_url);
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
    let _ = webview.close();
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
        let _ = existing.close();
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
        is_allowed_gdrive_host(nav_url)
    });

    // Position offscreen instead of calling hide() so WebView2 does not throttle background load
    let webview = main_window
        .add_child(
            builder,
            LogicalPosition::new(-10000.0, -10000.0),
            LogicalSize::new(200.0, 200.0),
        )
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    let _ = crate::security::harden(app, &webview, false);
    crate::window::send_webview_to_back(&webview);
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
            Some("ready") => return Ok((webview, token)),
            Some("login_required") => {
                reset_gdrive_webview(app, &webview);
                return Err("backup.sync_gdrive_login_required".into());
            }
            _ => {
                if attempt > 0 && attempt % 10 == 0 {
                    if let Ok(encoded_token) = serde_json::to_string(&token) {
                        let _ = webview.eval(&format!("window.__veloceGDrivePing?.({encoded_token});"));
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
) -> Result<(), String> {
    super::validation::document(document)?;
    let json_bytes = serde_json::to_vec_pretty(document).map_err(|e| e.to_string())?;
    if json_bytes.len() as u64 > MAX_BYTES {
        return Err("backup.file_too_large".into());
    }
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
            reset_gdrive_webview(app, &webview);
            "backup.sync_network_failed: Google Drive web session timed out".to_string()
        })?
        .map_err(|_| "backup.sync_network_failed".to_string())?;

    reset_gdrive_webview(app, &webview);
    if let Err(err) = response {
        return Err(err);
    }
    Ok(())
}

pub async fn pull_document_via_webview(app: &AppHandle) -> Result<BackupDocument, String> {
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
        "window.__veloceGDrivePull?.({}, {});",
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

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(b64_payload.as_bytes())
        .map_err(|e| format!("backup.invalid_document: {e}"))?;
    files::decode(&decoded_bytes)
}
