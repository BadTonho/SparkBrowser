//! Desktop OAuth: only the temporary authorization code crosses loopback.
use super::{http, platform};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
pub const SCOPE: &str = "https://www.googleapis.com/auth/gmail.metadata";
#[derive(Clone, Deserialize)]
pub struct Config {
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Credential {
    pub refresh: String,
    pub email: String,
    pub client_id: String,
}
#[derive(Clone)]
pub struct Access {
    pub token: String,
    pub expires: u64,
}
pub struct Grant {
    pub access: Access,
    pub refresh: Option<String>,
}

pub fn config(app: &AppHandle) -> Result<Config, String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|_| "oauth_not_configured")?
        .join("oauth/google-desktop.credentials.json");
    let file = std::fs::File::open(path).map_err(|_| "oauth_not_configured")?;
    let mut bytes = Vec::new();
    file.take(16_385)
        .read_to_end(&mut bytes)
        .map_err(|_| "oauth_not_configured")?;
    if bytes.len() > 16_384 {
        return Err("oauth_not_configured".into());
    }
    #[derive(Deserialize)]
    struct Download {
        installed: Config,
    }
    let config: Download = serde_json::from_slice(&bytes).map_err(|_| "oauth_not_configured")?;
    if config.installed.client_id.len() > 256
        || !config
            .installed
            .client_id
            .ends_with(".apps.googleusercontent.com")
        || config.installed.client_secret.len() > 512
    {
        return Err("oauth_not_configured".into());
    }
    Ok(config.installed)
}
fn random() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| "oauth_error")?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}
pub fn connect(
    config: &Config,
    endpoints: &super::Endpoints,
    cancel: Arc<AtomicBool>,
) -> Result<Grant, String> {
    connect_scoped(
        config,
        endpoints,
        cancel,
        SCOPE,
        platform::open_authorization,
    )
}
pub(super) fn connect_scoped(
    config: &Config,
    endpoints: &super::Endpoints,
    cancel: Arc<AtomicBool>,
    scope: &str,
    open: fn(&url::Url) -> Result<(), String>,
) -> Result<Grant, String> {
    let listener =
        TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).map_err(|_| "oauth_error")?;
    listener.set_nonblocking(true).map_err(|_| "oauth_error")?;
    let state = random()?;
    let verifier = random()?;
    let redirect = format!(
        "http://127.0.0.1:{}/callback",
        listener.local_addr().map_err(|_| "oauth_error")?.port()
    );
    let mut authorize = url::Url::parse(&endpoints.authorize).map_err(|_| "oauth_error")?;
    authorize.query_pairs_mut().extend_pairs([
        ("client_id", config.client_id.as_str()),
        ("redirect_uri", &redirect),
        ("response_type", "code"),
        ("scope", scope),
        (
            "code_challenge",
            &URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
        ),
        ("code_challenge_method", "S256"),
        ("state", &state),
    ]);
    if scope == SCOPE {
        authorize
            .query_pairs_mut()
            .extend_pairs([("access_type", "offline"), ("prompt", "consent")]);
    } else {
        authorize
            .query_pairs_mut()
            .append_pair("show_dialog", "true");
    }
    if let Some(base) = &endpoints.fixture {
        http::send(
            http::client()?
                .post(format!("{base}/authorize"))
                .body(authorize.to_string()),
        )?
        .error_for_status()
        .map_err(|_| "oauth_error")?;
    } else {
        open(&authorize)?;
    }
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err("cancelled".into());
        }
        if Instant::now() >= deadline {
            return Err("oauth_timeout".into());
        }
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if !peer.ip().is_loopback() {
                    continue;
                }
                crate::error_log::ignored(stream.set_read_timeout(Some(Duration::from_secs(1))));
                crate::error_log::ignored(stream.set_write_timeout(Some(Duration::from_secs(1))));
                let mut data = Vec::new();
                let mut chunk = [0u8; 1024];
                while data.len() <= 8192 && !data.windows(4).any(|v| v == b"\r\n\r\n") {
                    match stream.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(n) => data.extend_from_slice(&chunk[..n]),
                        Err(_) => break,
                    }
                }
                let request = std::str::from_utf8(&data).unwrap_or("");
                let target = request
                    .lines()
                    .next()
                    .and_then(|l| l.strip_prefix("GET "))
                    .and_then(|l| l.strip_suffix(" HTTP/1.1"));
                let result = target
                    .filter(|_| data.len() <= 8192)
                    .ok_or("oauth_callback")
                    .and_then(|s| callback(s, &state));
                if let Err("oauth_callback") = result {
                    let _=stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                    continue;
                }
                let body =
                    b"Authorization complete. You can close this window and return to Spark.";
                let _=write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nContent-Security-Policy: default-src 'none'\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len());
                crate::error_log::ignored(stream.write_all(body));
                let code = result.map_err(str::to_owned)?;
                if cancel.load(Ordering::SeqCst) {
                    return Err("cancelled".into());
                }
                return grant(
                    &endpoints.token,
                    scope,
                    &[
                        ("client_id", &config.client_id),
                        ("client_secret", &config.client_secret),
                        ("code", &code),
                        ("code_verifier", &verifier),
                        ("redirect_uri", &redirect),
                        ("grant_type", "authorization_code"),
                    ],
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100))
            }
            Err(_) => return Err("oauth_error".into()),
        }
    }
}
fn callback(target: &str, state: &str) -> Result<String, &'static str> {
    if !target.starts_with("/callback?") {
        return Err("oauth_callback");
    }
    let url =
        url::Url::parse(&format!("http://127.0.0.1{target}")).map_err(|_| "oauth_callback")?;
    let values: Vec<_> = url.query_pairs().collect();
    if values.iter().filter(|(k, _)| k == "state").count() != 1
        || !values.iter().any(|(k, v)| k == "state" && v == state)
    {
        return Err("oauth_callback");
    }
    if values.iter().any(|(k, _)| k == "error") {
        return Err("access_denied");
    }
    if values.iter().filter(|(k, _)| k == "code").count() != 1 {
        return Err("oauth_callback");
    }
    values
        .iter()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .filter(|v| !v.is_empty() && v.len() <= 2048)
        .ok_or("oauth_callback")
}
pub fn refresh(
    config: &Config,
    endpoints: &super::Endpoints,
    refresh: &str,
) -> Result<Grant, String> {
    grant(
        &endpoints.token,
        SCOPE,
        &[
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("refresh_token", refresh),
            ("grant_type", "refresh_token"),
        ],
    )
}
pub(super) fn grant(endpoint: &str, scope: &str, params: &[(&str, &str)]) -> Result<Grant, String> {
    let params: Vec<_> = params
        .iter()
        .filter(|(key, value)| *key != "client_secret" || !value.is_empty())
        .collect();
    let response = http::send(http::client()?.post(endpoint).form(&params))?;
    if scope != SCOPE && response.status().as_u16() == 429 {
        let retry = response
            .headers()
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.parse::<u64>().ok())
            .unwrap_or(30)
            .max(1);
        return Err(format!("spotify_rate_limited:{retry}"));
    }
    if response.status().as_u16() == 400 {
        let value: serde_json::Value =
            serde_json::from_reader(response.take(8192)).map_err(|_| "oauth_error")?;
        return Err(if value["error"] == "invalid_grant" {
            "reconnect"
        } else {
            "oauth_error"
        }
        .into());
    }
    #[derive(Deserialize)]
    struct Token {
        access_token: String,
        refresh_token: Option<String>,
        expires_in: u64,
        scope: Option<String>,
    }
    let token: Token = http::json(response)?;
    if token.access_token.is_empty()
        || token.access_token.len() > 4096
        || token
            .refresh_token
            .as_ref()
            .is_some_and(|s| s.is_empty() || s.len() > 2048)
        || token.scope.as_ref().is_some_and(|s| {
            scope
                .split_whitespace()
                .any(|required| !s.split_whitespace().any(|v| v == required))
        })
    {
        return Err("access_denied".into());
    }
    Ok(Grant {
        access: Access {
            token: token.access_token,
            expires: super::now() + token.expires_in.min(86400).saturating_sub(30),
        },
        refresh: token.refresh_token,
    })
}
pub fn revoke(endpoint: &str, token: &str) {
    if let Ok(client) = http::client() {
        crate::error_log::ignored(client.post(endpoint).form(&[("token", token)]).send());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callbacks_require_exact_state_single_code_and_expected_path() {
        assert_eq!(
            callback("/callback?code=ok&state=current", "current").unwrap(),
            "ok"
        );
        for target in [
            "/callback?code=ok&state=old",
            "/callback?code=a&code=b&state=current",
            "/callback?code=a&state=current&state=current",
            "/evil?code=a&state=current",
        ] {
            assert!(callback(target, "current").is_err());
        }
        assert_eq!(
            callback("/callback?error=access_denied&state=current", "current"),
            Err("access_denied")
        );
    }
}
