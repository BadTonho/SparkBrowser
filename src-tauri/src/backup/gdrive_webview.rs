use super::{
    files,
    model::{BackupDocument, MAX_BYTES},
    service::BackupService,
};
use crate::{database::preferences_repo, state::browser_state::BrowserState};
#[cfg(not(windows))]
use base64::Engine;
use std::{
    collections::HashMap,
    io::Read,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::Duration,
};
#[cfg(not(windows))]
use tauri::{LogicalPosition, LogicalSize, WebviewUrl};
use tauri::{AppHandle, Manager, Url};

pub const GDRIVE_SYNC_WEBVIEW_LABEL: &str = "google-drive-sync-webview";
pub const SYNC_FOLDER_NAME: &str = "Spark Backup";
pub const LEGACY_SYNC_FOLDER_NAME: &str = "Veloce Browser Backup";
pub const FOLDER_MIME_TYPE: &str = "application/vnd.google-apps.folder";
const SYNC_FILE_NAME: &str = "browser.veloce-backup.json";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

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

/// FIPS 180-4 SHA-1 implementation used for Google SAPISIDHASH header generation.
pub fn sha1_hex(data: &[u8]) -> String {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 80];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32),
                _ => (b ^ c ^ d, 0xCA62C1D6u32),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    format!("{h0:08x}{h1:08x}{h2:08x}{h3:08x}{h4:08x}")
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
    let Ok(db) = state.db.lock() else {
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
        let _ = webview.close();
    }
}

fn resolve_gdrive_session_context(app: &AppHandle) -> (Option<PathBuf>, String, Option<String>) {
    let mut detected_tab_id: Option<String> = None;
    let mut detected_authuser: Option<String> = None;

    if let Some(state) = app.try_state::<BrowserState>() {
        let active_id = state.active_tab_id.lock().ok().map(|g| g.clone());
        if let Ok(tabs) = state.tabs.lock() {
            if let Some(active) = active_id.as_deref() {
                if let Some(tab) = tabs.iter().find(|t| t.id == active && !t.is_ephemeral) {
                    if let Some(au) = extract_authuser_from_url(&tab.url) {
                        detected_tab_id = Some(tab.id.clone());
                        detected_authuser = Some(au);
                    }
                }
            }
            if detected_authuser.is_none() {
                for tab in tabs.iter().rev() {
                    if !tab.is_ephemeral {
                        if let Some(au) = extract_authuser_from_url(&tab.url) {
                            detected_tab_id = Some(tab.id.clone());
                            detected_authuser = Some(au);
                            break;
                        }
                    }
                }
            }
            if detected_tab_id.is_none() {
                detected_tab_id = active_id.or_else(|| {
                    tabs.iter()
                        .find(|t| !t.is_ephemeral)
                        .map(|t| t.id.clone())
                });
            }
        }

        if let Ok(db) = state.db.lock() {
            if let Some(conn) = db.as_ref() {
                if let Some(ref au) = detected_authuser {
                    let _ = preferences_repo::store(
                        conn,
                        preferences_repo::BACKUP_SYNC_GDRIVE_AUTHUSER_KEY,
                        au,
                    );
                } else if let Ok(Some(saved_au)) =
                    preferences_repo::load(conn, preferences_repo::BACKUP_SYNC_GDRIVE_AUTHUSER_KEY)
                {
                    if !saved_au.is_empty()
                        && saved_au.len() <= 4
                        && saved_au.bytes().all(|b| b.is_ascii_digit())
                    {
                        detected_authuser = Some(saved_au);
                    }
                }
            }
        }
    }

    let data_dir = detected_tab_id
        .as_deref()
        .and_then(|tab_id| crate::window::workspace_data::content_directory(app, tab_id).ok())
        .flatten();
    let authuser = detected_authuser.unwrap_or_else(|| "0".to_string());
    (data_dir, authuser, detected_tab_id)
}

#[cfg(windows)]
async fn get_webview_cookies_for_url(
    app: &AppHandle,
    preferred_tab_id: Option<&str>,
    url: &str,
) -> Result<Vec<(String, String)>, String> {
    use std::sync::Arc;
    use webview2_com::{
        CoTaskMemPWSTR, GetCookiesCompletedHandler, Microsoft::Web::WebView2::Win32::*,
    };
    use windows_core::{Interface, PCWSTR};

    let view = preferred_tab_id
        .and_then(|id| app.get_webview(&crate::state::tab_model::webview_label_for_tab(id)))
        .or_else(|| crate::window::webview_manager::active_content_webview(app))
        .or_else(|| {
            app.state::<BrowserState>()
                .snapshot_tabs()
                .first()
                .and_then(|t| {
                    app.get_webview(&crate::state::tab_model::webview_label_for_tab(&t.id))
                })
        })
        .or_else(|| app.get_webview("main"))
        .ok_or_else(|| "backup.sync_gdrive_login_required".to_string())?;

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let sender = Arc::new(Mutex::new(Some(sender)));
    let uri: Vec<u16> = url.encode_utf16().chain(Some(0)).collect();

    let sender_for_webview = sender.clone();
    view.with_webview(move |platform| {
        let op = (|| -> windows_core::Result<()> {
            let core = unsafe { platform.controller().CoreWebView2()? };
            let core2 = core.cast::<ICoreWebView2_2>()?;
            let cm = unsafe { core2.CookieManager()? };
            let callback_sender = sender_for_webview.clone();

            let handler = GetCookiesCompletedHandler::create(Box::new(move |hr, list| {
                let parse_result = (|| -> windows_core::Result<Vec<(String, String)>> {
                    hr?;
                    let mut pairs = Vec::new();
                    if let Some(list) = list {
                        let mut count = 0;
                        unsafe {
                            list.Count(&mut count)?;
                            for i in 0..count {
                                let c = list.GetValueAtIndex(i)?;
                                let mut name = Default::default();
                                let mut value = Default::default();
                                c.Name(&mut name)?;
                                c.Value(&mut value)?;
                                let n = CoTaskMemPWSTR::from(name).to_string();
                                let v = CoTaskMemPWSTR::from(value).to_string();
                                if !n.is_empty() {
                                    pairs.push((n, v));
                                }
                            }
                        }
                    }
                    Ok(pairs)
                })();

                if let Ok(mut lock) = callback_sender.lock() {
                    if let Some(tx) = lock.take() {
                        let _ = tx.send(parse_result.map_err(|e| e.to_string()));
                    }
                }
                Ok(())
            }));

            unsafe {
                cm.GetCookies(PCWSTR(uri.as_ptr()), &handler)?;
            }
            Ok(())
        })();

        if let Err(e) = op {
            if let Ok(mut lock) = sender_for_webview.lock() {
                if let Some(tx) = lock.take() {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        }
    })
    .map_err(|e| e.to_string())?;

    match tokio::time::timeout(Duration::from_secs(5), receiver).await {
        Ok(Ok(Ok(pairs))) => Ok(pairs),
        Ok(Ok(Err(e))) => Err(format!("backup.sync_network_failed: {e}")),
        _ => Err("backup.sync_network_failed: Cookie manager timeout".into()),
    }
}

#[cfg(windows)]
async fn collect_google_cookies(
    app: &AppHandle,
    preferred_tab_id: Option<&str>,
) -> Result<(String, HashMap<String, String>), String> {
    let mut map = HashMap::new();
    let mut ordered = Vec::new();
    for domain_url in [
        "https://drive.google.com",
        "https://clients6.google.com",
        "https://mail.google.com",
    ] {
        if let Ok(pairs) = get_webview_cookies_for_url(app, preferred_tab_id, domain_url).await {
            for (k, v) in pairs {
                if !map.contains_key(&k) {
                    ordered.push(format!("{k}={v}"));
                }
                map.insert(k, v);
            }
        }
    }
    let has_sapisid = map.contains_key("SAPISID")
        || map.contains_key("__Secure-3PAPISID")
        || map.contains_key("__Secure-1PAPISID");
    let has_sid = map.contains_key("SID")
        || map.contains_key("__Secure-1PSID")
        || map.contains_key("__Secure-3PSID");
    if !has_sapisid || !has_sid {
        return Err("backup.sync_gdrive_login_required".into());
    }
    Ok((ordered.join("; "), map))
}

pub fn build_sapisidhash_authorization(cookies: &HashMap<String, String>, timestamp: u64) -> Result<String, String> {
    let origin = "https://drive.google.com";
    let mut tokens = Vec::new();
    if let Some(s) = cookies
        .get("SAPISID")
        .or_else(|| cookies.get("__Secure-3PAPISID"))
    {
        let hash = sha1_hex(format!("{timestamp} {s} {origin}").as_bytes());
        tokens.push(format!("SAPISIDHASH {timestamp}_{hash}"));
    }
    if let Some(s1) = cookies.get("__Secure-1PAPISID") {
        let hash = sha1_hex(format!("{timestamp} {s1} {origin}").as_bytes());
        tokens.push(format!("SAPISID1PHASH {timestamp}_{hash}"));
    }
    if let Some(s3) = cookies.get("__Secure-3PAPISID") {
        let hash = sha1_hex(format!("{timestamp} {s3} {origin}").as_bytes());
        tokens.push(format!("SAPISID3PHASH {timestamp}_{hash}"));
    }
    if tokens.is_empty() {
        return Err("backup.sync_gdrive_login_required".into());
    }
    Ok(tokens.join(" "))
}

pub fn extract_api_keys_from_html(html: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let bytes = html.as_bytes();
    if bytes.len() < 40 {
        return keys;
    }
    for i in 0..=(bytes.len() - 39) {
        if &bytes[i..i + 4] == b"AIza" {
            let prev_ok = i == 0 || matches!(bytes[i - 1], b'"' | b'\'' | b'=' | b',' | b' ' | b'[');
            let next_ok = i + 39 == bytes.len()
                || matches!(
                    bytes[i + 39],
                    b'"' | b'\'' | b'&' | b',' | b' ' | b']' | b'}' | b'\\'
                );
            if prev_ok
                && next_ok
                && bytes[i + 4..i + 39]
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-')
            {
                let candidate = String::from_utf8_lossy(&bytes[i..i + 39]).to_string();
                let ctx_start = i.saturating_sub(140);
                let ctx_end = (i + 180).min(bytes.len());
                let ctx = String::from_utf8_lossy(&bytes[ctx_start..ctx_end]);
                let is_drive = ctx.contains("clients6.google.com") || ctx.contains("/drive/");
                if let Some(pos) = keys.iter().position(|k| k == &candidate) {
                    if is_drive && pos > 0 {
                        keys.remove(pos);
                        keys.insert(0, candidate);
                    }
                } else if is_drive {
                    keys.insert(0, candidate);
                } else {
                    keys.push(candidate);
                }
            }
        }
    }
    keys
}

fn extract_email_from_text(text: &str) -> Option<String> {
    for suffix in ["@gmail.com", "@googlemail.com"] {
        if let Some(at_pos) = text.find(suffix) {
            let prefix = &text[..at_pos];
            let user_part: String = prefix
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-'))
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            if !user_part.is_empty() && user_part.len() <= 64 {
                return Some(format!("{user_part}{suffix}"));
            }
        }
    }
    None
}

#[cfg(windows)]
fn build_gdrive_http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))
}

#[cfg(windows)]
fn discover_gdrive_session_blocking(
    client: &reqwest::blocking::Client,
    cookie_header: &str,
    authuser: &str,
    cached_key: Option<String>,
) -> Result<(Vec<String>, Option<String>), String> {
    let mut detected_email: Option<String> = None;

    // 1. Fast check on Gmail Atom feed for exact connected email (same as New Tab Gmail widget)
    let feed_url = format!("https://mail.google.com/mail/u/{authuser}/feed/atom");
    if let Ok(resp) = client
        .get(&feed_url)
        .header(reqwest::header::COOKIE, cookie_header)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
    {
        if resp.status().is_success() {
            if let Ok(xml) = resp.text() {
                detected_email = extract_email_from_text(&xml);
            }
        }
    }

    let mut keys = Vec::new();
    if let Some(k) = cached_key {
        keys.push(k);
    }

    // 2. Fetch Google Drive page HTML to extract Drive's first-party API key (and email fallback)
    let drive_url = format!("https://drive.google.com/drive/u/{authuser}/my-drive");
    let resp = client
        .get(&drive_url)
        .header(reqwest::header::COOKIE, cookie_header)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;

    let final_url = resp.url().as_str().to_string();
    if final_url.contains("accounts.google.com") || final_url.contains("/ServiceLogin") {
        return Err("backup.sync_gdrive_login_required".into());
    }
    if resp.status().as_u16() == 401 || resp.status().as_u16() == 403 {
        return Err("backup.sync_gdrive_login_required".into());
    }

    let html = resp
        .text()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    if detected_email.is_none() {
        detected_email = extract_email_from_text(&html);
    }
    for k in extract_api_keys_from_html(&html) {
        if !keys.contains(&k) {
            keys.push(k);
        }
    }

    Ok((keys, detected_email))
}

#[cfg(windows)]
struct FoundGDriveFile {
    id: Option<String>,
    download_url: Option<String>,
    api_version: String,
    api_key: String,
    folder_id: Option<String>,
}

pub fn build_folder_metadata_json(is_v2: bool) -> serde_json::Value {
    if is_v2 {
        serde_json::json!({
            "title": SYNC_FOLDER_NAME,
            "mimeType": FOLDER_MIME_TYPE,
            "parents": [{ "id": "root" }]
        })
    } else {
        serde_json::json!({
            "name": SYNC_FOLDER_NAME,
            "mimeType": FOLDER_MIME_TYPE,
            "parents": ["root"]
        })
    }
}

#[cfg(windows)]
fn ensure_backup_folder_blocking(
    client: &reqwest::blocking::Client,
    cookie_header: &str,
    auth_header: &str,
    authuser: &str,
    api_keys: &[String],
    create_if_missing: bool,
) -> Result<(Option<String>, String, String), String> {
    if api_keys.is_empty() {
        return Err("backup.sync_gdrive_login_required".into());
    }

    let mut last_status = 0u16;
    let mut last_detail = String::new();
    let mut working_ver_and_key: Option<(String, String)> = None;

    for key in api_keys {
        for v2_ver in ["v2beta", "v2internal", "v2"] {
            let mut list_url = Url::parse(&format!(
                "https://clients6.google.com/drive/{v2_ver}/files"
            ))
            .map_err(|e| e.to_string())?;
            list_url.query_pairs_mut()
                .append_pair(
                    "q",
                    &format!(
                        "title = '{SYNC_FOLDER_NAME}' and mimeType = '{FOLDER_MIME_TYPE}' and 'root' in parents and trashed = false"
                    ),
                )
                .append_pair("maxResults", "1")
                .append_pair("key", key);

            if let Ok(r) = client
                .get(list_url.as_str())
                .header(reqwest::header::COOKIE, cookie_header)
                .header(reqwest::header::AUTHORIZATION, auth_header)
                .header(reqwest::header::ORIGIN, "https://drive.google.com")
                .header(reqwest::header::REFERER, "https://drive.google.com/")
                .header(reqwest::header::USER_AGENT, USER_AGENT)
                .header("X-Goog-AuthUser", authuser)
                .send()
            {
                last_status = r.status().as_u16();
                if r.status().is_success() {
                    if let Ok(txt) = r.text() {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                            let id = json["items"]
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|item| item["id"].as_str())
                                .map(str::to_string);
                            if id.is_some() {
                                return Ok((id, v2_ver.to_string(), key.clone()));
                            }
                            if working_ver_and_key.is_none() {
                                working_ver_and_key = Some((v2_ver.to_string(), key.clone()));
                            }
                            break;
                        }
                    }
                } else if last_status == 401 {
                    return Err("backup.sync_gdrive_login_required".into());
                } else if let Ok(txt) = r.text() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                        if let Some(m) = v["error"]["message"].as_str() {
                            last_detail = m.to_string();
                        }
                    }
                }
            }
        }

        if working_ver_and_key.is_none() {
            let mut list_v3 = Url::parse("https://clients6.google.com/drive/v3/files")
                .map_err(|e| e.to_string())?;
            list_v3
                .query_pairs_mut()
                .append_pair(
                    "q",
                    &format!(
                        "name = '{SYNC_FOLDER_NAME}' and mimeType = '{FOLDER_MIME_TYPE}' and 'root' in parents and trashed = false"
                    ),
                )
                .append_pair("pageSize", "1")
                .append_pair("key", key);

            if let Ok(r) = client
                .get(list_v3.as_str())
                .header(reqwest::header::COOKIE, cookie_header)
                .header(reqwest::header::AUTHORIZATION, auth_header)
                .header(reqwest::header::ORIGIN, "https://drive.google.com")
                .header(reqwest::header::REFERER, "https://drive.google.com/")
                .header(reqwest::header::USER_AGENT, USER_AGENT)
                .header("X-Goog-AuthUser", authuser)
                .send()
            {
                last_status = r.status().as_u16();
                if r.status().is_success() {
                    if let Ok(txt) = r.text() {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                            let id = json["files"]
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|item| item["id"].as_str())
                                .map(str::to_string);
                            if id.is_some() {
                                return Ok((id, "v3".to_string(), key.clone()));
                            }
                            working_ver_and_key = Some(("v3".to_string(), key.clone()));
                        }
                    }
                } else if last_status == 401 {
                    return Err("backup.sync_gdrive_login_required".into());
                } else if let Ok(txt) = r.text() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                        if let Some(m) = v["error"]["message"].as_str() {
                            last_detail = m.to_string();
                        }
                    }
                }
            }
        }

        if working_ver_and_key.is_some() {
            break;
        }
    }

    let Some((ver, key)) = working_ver_and_key else {
        return Err(format!(
            "backup.sync_network_failed: Drive query HTTP {last_status} {last_detail}"
        ));
    };

    if !create_if_missing {
        return Ok((None, ver, key));
    }

    for candidate_ver in [ver.as_str(), "v2beta", "v2internal", "v2", "v3"] {
        let is_v2 = candidate_ver != "v3";
        let create_url = format!(
            "https://clients6.google.com/drive/{candidate_ver}/files?key={key}"
        );
        let body = build_folder_metadata_json(is_v2).to_string();

        if let Ok(r) = client
            .post(&create_url)
            .header(reqwest::header::COOKIE, cookie_header)
            .header(reqwest::header::AUTHORIZATION, auth_header)
            .header(reqwest::header::ORIGIN, "https://drive.google.com")
            .header(reqwest::header::REFERER, "https://drive.google.com/")
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .header("X-Goog-AuthUser", authuser)
            .header(reqwest::header::CONTENT_TYPE, "application/json; charset=UTF-8")
            .body(body)
            .send()
        {
            if r.status().is_success() {
                if let Ok(txt) = r.text() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                        if let Some(folder_id) = json["id"].as_str() {
                            return Ok((
                                Some(folder_id.to_string()),
                                candidate_ver.to_string(),
                                key,
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok((None, ver, key))
}

#[cfg(windows)]
fn query_backup_file_in_parent_blocking(
    client: &reqwest::blocking::Client,
    cookie_header: &str,
    auth_header: &str,
    authuser: &str,
    api_version: &str,
    api_key: &str,
    parent_id: Option<&str>,
) -> Result<Option<(String, Option<String>)>, String> {
    let is_v2 = api_version != "v3";
    let mut url = Url::parse(&format!(
        "https://clients6.google.com/drive/{api_version}/files"
    ))
    .map_err(|e| e.to_string())?;

    let query = match (is_v2, parent_id) {
        (true, Some(pid)) => {
            format!("title = '{SYNC_FILE_NAME}' and '{pid}' in parents and trashed = false")
        }
        (true, None) => format!("title = '{SYNC_FILE_NAME}' and trashed = false"),
        (false, Some(pid)) => {
            format!("name = '{SYNC_FILE_NAME}' and '{pid}' in parents and trashed = false")
        }
        (false, None) => format!("name = '{SYNC_FILE_NAME}' and trashed = false"),
    };

    {
        let mut qp = url.query_pairs_mut();
        qp.append_pair("q", &query);
        if is_v2 {
            qp.append_pair("maxResults", "1");
        } else {
            qp.append_pair("pageSize", "1");
        }
        qp.append_pair("key", api_key);
    }

    let resp = client
        .get(url.as_str())
        .header(reqwest::header::COOKIE, cookie_header)
        .header(reqwest::header::AUTHORIZATION, auth_header)
        .header(reqwest::header::ORIGIN, "https://drive.google.com")
        .header(reqwest::header::REFERER, "https://drive.google.com/")
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header("X-Goog-AuthUser", authuser)
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;

    if resp.status().as_u16() == 401 {
        return Err("backup.sync_gdrive_login_required".into());
    }
    if !resp.status().is_success() {
        return Ok(None);
    }

    let txt = resp
        .text()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    let json = serde_json::from_str::<serde_json::Value>(&txt)
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    let arr_key = if is_v2 { "items" } else { "files" };
    let first = json[arr_key].as_array().and_then(|arr| arr.first());
    let Some(id) = first.and_then(|item| item["id"].as_str()) else {
        return Ok(None);
    };
    let dl = first
        .and_then(|item| item["downloadUrl"].as_str())
        .map(str::to_string);
    Ok(Some((id.to_string(), dl)))
}

#[cfg(windows)]
fn find_backup_file_blocking(
    client: &reqwest::blocking::Client,
    cookie_header: &str,
    auth_header: &str,
    authuser: &str,
    api_keys: &[String],
    create_folder_if_missing: bool,
) -> Result<FoundGDriveFile, String> {
    let (folder_id, api_version, api_key) = ensure_backup_folder_blocking(
        client,
        cookie_header,
        auth_header,
        authuser,
        api_keys,
        create_folder_if_missing,
    )?;

    if let Some(ref fid) = folder_id {
        if let Some((file_id, dl_url)) = query_backup_file_in_parent_blocking(
            client,
            cookie_header,
            auth_header,
            authuser,
            &api_version,
            &api_key,
            Some(fid),
        )? {
            return Ok(FoundGDriveFile {
                id: Some(file_id),
                download_url: dl_url,
                api_version,
                api_key,
                folder_id,
            });
        }
    }

    // Legacy fallback during pull: if not yet inside the folder, check anywhere in My Drive
    if !create_folder_if_missing {
        if let Some((file_id, dl_url)) = query_backup_file_in_parent_blocking(
            client,
            cookie_header,
            auth_header,
            authuser,
            &api_version,
            &api_key,
            None,
        )? {
            return Ok(FoundGDriveFile {
                id: Some(file_id),
                download_url: dl_url,
                api_version,
                api_key,
                folder_id,
            });
        }
    }

    Ok(FoundGDriveFile {
        id: None,
        download_url: None,
        api_version,
        api_key,
        folder_id,
    })
}

#[cfg(windows)]
fn upload_multipart_blocking(
    client: &reqwest::blocking::Client,
    cookie_header: &str,
    auth_header: &str,
    authuser: &str,
    found: &FoundGDriveFile,
    json_text: &str,
) -> Result<(), String> {
    let mut versions = vec![found.api_version.as_str()];
    for v in ["v2beta", "v2internal", "v2", "v3"] {
        if !versions.contains(&v) {
            versions.push(v);
        }
    }

    let boundary = format!(
        "veloce_gdrive_boundary_{}",
        crate::state::tab_model::TabInfo::current_epoch_secs()
    );
    let mut last_status = 0u16;
    let mut last_detail = String::new();
    let parent_id = found.folder_id.as_deref().unwrap_or("root");

    for ver in versions {
        let is_v2 = ver == "v2beta" || ver == "v2internal" || ver == "v2";
        let metadata = if is_v2 {
            if found.id.is_some() {
                serde_json::json!({ "title": SYNC_FILE_NAME, "mimeType": "application/json" })
            } else {
                serde_json::json!({
                    "title": SYNC_FILE_NAME,
                    "mimeType": "application/json",
                    "parents": [{ "id": parent_id }]
                })
            }
        } else if found.id.is_some() {
            serde_json::json!({ "mimeType": "application/json" })
        } else {
            serde_json::json!({
                "name": SYNC_FILE_NAME,
                "mimeType": "application/json",
                "parents": [parent_id]
            })
        };

        let body = format!(
            "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{}\r\n--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{json_text}\r\n--{boundary}--",
            metadata
        );

        let url = match &found.id {
            Some(existing_id) => format!(
                "https://clients6.google.com/upload/drive/{ver}/files/{existing_id}?uploadType=multipart&key={}",
                found.api_key
            ),
            None => format!(
                "https://clients6.google.com/upload/drive/{ver}/files?uploadType=multipart&key={}",
                found.api_key
            ),
        };

        let builder = match (&found.id, is_v2) {
            (Some(_), true) => client.put(&url),
            (Some(_), false) => client.patch(&url),
            (None, _) => client.post(&url),
        };

        if let Ok(resp) = builder
            .header(reqwest::header::COOKIE, cookie_header)
            .header(reqwest::header::AUTHORIZATION, auth_header)
            .header(reqwest::header::ORIGIN, "https://drive.google.com")
            .header(reqwest::header::REFERER, "https://drive.google.com/")
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .header("X-Goog-AuthUser", authuser)
            .header(
                reqwest::header::CONTENT_TYPE,
                format!("multipart/related; boundary={boundary}"),
            )
            .body(body)
            .send()
        {
            last_status = resp.status().as_u16();
            if resp.status().is_success() {
                return Ok(());
            }
            if last_status == 401 {
                return Err("backup.sync_gdrive_login_required".into());
            }
            if let Ok(txt) = resp.text() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                    if let Some(m) = v["error"]["message"].as_str() {
                        last_detail = m.to_string();
                    }
                }
            }
        }
    }

    Err(format!(
        "backup.sync_network_failed: Drive upload HTTP {last_status} {last_detail}"
    ))
}

#[cfg(windows)]
pub async fn push_document_via_webview(
    app: &AppHandle,
    document: &BackupDocument,
) -> Result<(), String> {
    close_gdrive_sync_webview(app);
    super::validation::document(document)?;
    let json_text = serde_json::to_string_pretty(document).map_err(|e| e.to_string())?;
    if json_text.len() as u64 > MAX_BYTES {
        return Err("backup.file_too_large".into());
    }

    let (_data_dir, authuser, preferred_tab_id) = resolve_gdrive_session_context(app);
    let (cookie_header, cookie_map) =
        collect_google_cookies(app, preferred_tab_id.as_deref()).await?;
    let now_ts = crate::state::tab_model::TabInfo::current_epoch_secs();
    let auth_header = build_sapisidhash_authorization(&cookie_map, now_ts)?;

    let cached_key = app
        .state::<BackupService>()
        .sync
        .gdrive
        .cached_api_key
        .lock()
        .ok()
        .and_then(|g| g.clone());

    let authuser_clone = authuser.clone();
    let (verified_key, detected_email, folder_id) = tauri::async_runtime::spawn_blocking(move || {
        let client = build_gdrive_http_client()?;
        let (keys, email) =
            discover_gdrive_session_blocking(&client, &cookie_header, &authuser_clone, cached_key)?;
        let found = find_backup_file_blocking(
            &client,
            &cookie_header,
            &auth_header,
            &authuser_clone,
            &keys,
            true,
        )?;
        upload_multipart_blocking(
            &client,
            &cookie_header,
            &auth_header,
            &authuser_clone,
            &found,
            &json_text,
        )?;
        Ok::<(String, Option<String>, Option<String>), String>((
            found.api_key,
            email,
            found.folder_id,
        ))
    })
    .await
    .map_err(|e| e.to_string())??;

    let service = app.state::<BackupService>();
    if let Ok(mut guard) = service.sync.gdrive.cached_api_key.lock() {
        *guard = Some(verified_key);
    }
    if let Some(fid) = folder_id {
        if let Ok(mut guard) = service.sync.gdrive.cached_folder_id.lock() {
            *guard = Some(fid);
        }
    }
    if let Some(email) = detected_email {
        if let Ok(mut guard) = service.sync.gdrive.connected_email.lock() {
            *guard = Some(email);
        }
        persist_detected_gdrive_email(app);
    }
    Ok(())
}

#[cfg(windows)]
pub async fn pull_document_via_webview(app: &AppHandle) -> Result<BackupDocument, String> {
    close_gdrive_sync_webview(app);
    let (_data_dir, authuser, preferred_tab_id) = resolve_gdrive_session_context(app);
    let (cookie_header, cookie_map) =
        collect_google_cookies(app, preferred_tab_id.as_deref()).await?;
    let now_ts = crate::state::tab_model::TabInfo::current_epoch_secs();
    let auth_header = build_sapisidhash_authorization(&cookie_map, now_ts)?;

    let cached_key = app
        .state::<BackupService>()
        .sync
        .gdrive
        .cached_api_key
        .lock()
        .ok()
        .and_then(|g| g.clone());

    let authuser_clone = authuser.clone();
    let (bytes, verified_key, detected_email) = tauri::async_runtime::spawn_blocking(move || {
        let client = build_gdrive_http_client()?;
        let (keys, email) =
            discover_gdrive_session_blocking(&client, &cookie_header, &authuser_clone, cached_key)?;
        let found = find_backup_file_blocking(
            &client,
            &cookie_header,
            &auth_header,
            &authuser_clone,
            &keys,
            false,
        )?;
        let Some(file_id) = found.id.as_deref() else {
            return Err("backup.sync_file_not_found".to_string());
        };

        let media_url = format!(
            "https://clients6.google.com/drive/{}/files/{file_id}?alt=media&key={}",
            found.api_version, found.api_key
        );
        let resp = client
            .get(&media_url)
            .header(reqwest::header::COOKIE, &cookie_header)
            .header(reqwest::header::AUTHORIZATION, &auth_header)
            .header(reqwest::header::ORIGIN, "https://drive.google.com")
            .header(reqwest::header::REFERER, "https://drive.google.com/")
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .header("X-Goog-AuthUser", &authuser_clone)
            .send()
            .map_err(|e| format!("backup.sync_network_failed: {e}"))?;

        if resp.status().as_u16() == 404 {
            return Err("backup.sync_file_not_found".into());
        }
        if resp.status().is_success() {
            let mut buf = Vec::new();
            resp.take(MAX_BYTES + 1)
                .read_to_end(&mut buf)
                .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
            return Ok((buf, found.api_key, email));
        }

        if let Some(dl_url) = found.download_url.as_deref() {
            let dl_resp = client
                .get(dl_url)
                .header(reqwest::header::COOKIE, &cookie_header)
                .header(reqwest::header::USER_AGENT, USER_AGENT)
                .send()
                .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
            if dl_resp.status().is_success() {
                let mut buf = Vec::new();
                dl_resp
                    .take(MAX_BYTES + 1)
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
                return Ok((buf, found.api_key, email));
            }
        }

        Err("backup.sync_network_failed".into())
    })
    .await
    .map_err(|e| e.to_string())??;

    let service = app.state::<BackupService>();
    if let Ok(mut guard) = service.sync.gdrive.cached_api_key.lock() {
        *guard = Some(verified_key);
    }
    if let Some(email) = detected_email {
        if let Ok(mut guard) = service.sync.gdrive.connected_email.lock() {
            *guard = Some(email);
        }
        persist_detected_gdrive_email(app);
    }
    files::decode(&bytes)
}

#[cfg(not(windows))]
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

#[cfg(not(windows))]
fn ensure_hidden_gdrive_webview(app: &AppHandle) -> Result<(tauri::Webview, String), String> {
    let service = app.state::<BackupService>();
    let (data_dir, authuser, _) = resolve_gdrive_session_context(app);

    if let Some(existing) = app.get_webview(GDRIVE_SYNC_WEBVIEW_LABEL) {
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
        if nav_url.scheme() == "veloce-internal" {
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
    let _ = crate::security::harden(app, &webview, false);
    Ok((webview, token))
}

#[cfg(not(windows))]
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
                persist_detected_gdrive_email(app);
                return Ok((webview, token));
            }
            Some("login_required") => {
                reset_gdrive_webview(app, &webview);
                return Err("backup.sync_gdrive_login_required".into());
            }
            _ => {
                if attempt > 0 && attempt % 10 == 0 {
                    if let Ok(encoded_token) = serde_json::to_string(&token) {
                        let _ =
                            webview.eval(&format!("window.__veloceGDrivePing?.({encoded_token});"));
                    }
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
    reset_gdrive_webview(app, &webview);
    Err("backup.sync_network_failed: Google Drive page did not finish loading".into())
}

#[cfg(not(windows))]
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
    persist_detected_gdrive_email(app);
    Ok(())
}

#[cfg(not(windows))]
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

    persist_detected_gdrive_email(app);
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(b64_payload.as_bytes())
        .map_err(|e| format!("backup.invalid_document: {e}"))?;
    files::decode(&decoded_bytes)
}
