//! Nextcloud sign-in for sync, through Nextcloud's own login flow for apps (Login Flow v2), so it
//! needs no app registration: Spark asks the server for a login page, the person approves Spark
//! there, and the server hands back an app password that only Spark uses. The sync itself is the
//! WebDAV target pointed at a "Spark" folder of the account.
//!
//! The poll token and the app password stay in Rust; the interface only gets the login page.
use crate::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Nextcloud lists Spark under this name in the person's devices and sessions.
const USER_AGENT: &str = "Spark Browser (sync)";
/// Nextcloud drops an unfinished login after 20 minutes.
const FLOW_LIFETIME: Duration = Duration::from_secs(20 * 60);
/// Folder of the account that holds the sync file.
const FOLDER: &str = "Spark";

#[derive(Default)]
pub struct NextcloudLogin(Mutex<Option<Flow>>);

struct Flow {
    id: u64,
    server: url::Url,
    token: String,
    endpoint: url::Url,
    started: Instant,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginStart {
    pub flow_id: u64,
    pub login_url: String,
}

/// What a finished login gives Spark.
pub struct Account {
    pub server: url::Url,
    pub login_name: String,
    pub app_password: String,
}

#[derive(Deserialize)]
struct StartReply {
    poll: PollInfo,
    login: String,
}

#[derive(Deserialize)]
struct PollInfo {
    token: String,
    endpoint: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PollReply {
    login_name: String,
    app_password: String,
}

/// The server address as typed: `https://` is added when missing, and anything after the
/// Nextcloud folder (`/index.php/...`, `/apps/...`, `/login`) is removed.
pub fn normalize_server(raw: &str) -> Result<url::Url, String> {
    let trimmed = raw.trim();
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let mut url = super::sync::validate_webdav_url(&with_scheme)
        .map_err(|_| "backup.nextcloud_invalid_server".to_string())?;
    url.set_query(None);
    url.set_fragment(None);
    let path = url.path().to_string();
    let base = ["/index.php", "/apps/", "/login", "/remote.php"]
        .iter()
        .filter_map(|marker| path.find(marker))
        .min()
        .map_or(path.as_str(), |at| &path[..at]);
    let base = base.trim_end_matches('/').to_string();
    url.set_path(&format!("{base}/"));
    Ok(url)
}

fn same_origin(a: &url::Url, b: &url::Url) -> bool {
    a.scheme() == b.scheme()
        && a.host_str() == b.host_str()
        && a.port_or_known_default() == b.port_or_known_default()
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))
}

/// Reads a small JSON reply (a login reply is a few hundred bytes).
fn read_json<T: serde::de::DeserializeOwned>(
    response: reqwest::blocking::Response,
) -> Result<T, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    response
        .take(64 * 1024)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

/// The WebDAV folder of `login_name` that holds the sync file.
pub fn webdav_folder(server: &url::Url, login_name: &str) -> Result<url::Url, String> {
    let mut url = server.clone();
    url.path_segments_mut()
        .map_err(|()| "backup.nextcloud_invalid_server".to_string())?
        .pop_if_empty()
        .extend(["remote.php", "dav", "files", login_name, FOLDER, ""]);
    Ok(url)
}

/// Asks the server for a login page. Network work: call it off the main thread.
fn request_login(server: &url::Url) -> Result<(String, url::Url, url::Url), String> {
    let start = server
        .join("index.php/login/v2")
        .map_err(|e| e.to_string())?;
    let response = client()?
        .post(start)
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    match response.status().as_u16() {
        200 => {}
        404 => return Err("backup.nextcloud_not_found".into()),
        status => return Err(format!("backup.sync_network_failed: HTTP {status}")),
    }
    let reply: StartReply =
        read_json(response).map_err(|_| "backup.nextcloud_not_found".to_string())?;
    let login = url::Url::parse(&reply.login).map_err(|_| "backup.nextcloud_not_found")?;
    let endpoint =
        url::Url::parse(&reply.poll.endpoint).map_err(|_| "backup.nextcloud_not_found")?;
    // The login page and the poll address must belong to the server the person typed.
    if !same_origin(&login, server)
        || !same_origin(&endpoint, server)
        || reply.poll.token.is_empty()
    {
        return Err("backup.nextcloud_unexpected_server".into());
    }
    Ok((reply.poll.token, endpoint, login))
}

impl NextcloudLogin {
    /// Starts a login on `server`, replacing any login still waiting.
    pub fn start(&self, raw_server: &str) -> Result<LoginStart, String> {
        let server = normalize_server(raw_server)?;
        let (token, endpoint, login) = request_login(&server)?;
        let mut current = self.0.lock().map_err(|e| e.to_string())?;
        let id = current.as_ref().map_or(1, |flow| flow.id.wrapping_add(1));
        *current = Some(Flow {
            id,
            server,
            token,
            endpoint,
            started: Instant::now(),
        });
        Ok(LoginStart {
            flow_id: id,
            login_url: login.to_string(),
        })
    }

    pub fn cancel(&self) {
        if let Ok(mut current) = self.0.lock() {
            *current = None;
        }
    }

    /// `None` while the person has not approved Spark yet. The login is forgotten once it ends.
    pub fn poll(&self, flow_id: u64) -> Result<Option<Account>, String> {
        let (server, token, endpoint) = {
            let mut current = self.0.lock().map_err(|e| e.to_string())?;
            let Some(flow) = current.as_ref().filter(|flow| flow.id == flow_id) else {
                return Err("backup.nextcloud_login_cancelled".into());
            };
            if flow.started.elapsed() > FLOW_LIFETIME {
                *current = None;
                return Err("backup.nextcloud_login_expired".into());
            }
            (
                flow.server.clone(),
                flow.token.clone(),
                flow.endpoint.clone(),
            )
        };
        let response = client()?
            .post(endpoint)
            .form(&[("token", token.as_str())])
            .send()
            .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
        match response.status().as_u16() {
            // Not approved yet.
            404 => return Ok(None),
            200 => {}
            status => return Err(format!("backup.sync_network_failed: HTTP {status}")),
        }
        let reply: PollReply =
            read_json(response).map_err(|_| "backup.nextcloud_unexpected_server".to_string())?;
        if reply.login_name.is_empty()
            || reply.login_name.chars().count() > 256
            || reply.app_password.is_empty()
            || reply.app_password.len() > 4096
        {
            return Err("backup.nextcloud_unexpected_server".into());
        }
        let mut current = self.0.lock().map_err(|e| e.to_string())?;
        if current.as_ref().is_some_and(|flow| flow.id == flow_id) {
            *current = None;
        }
        Ok(Some(Account {
            server,
            login_name: reply.login_name,
            app_password: reply.app_password,
        }))
    }
}

/// Creates the "Spark" folder of the account when it does not exist yet.
pub fn ensure_folder(
    folder: &url::Url,
    login_name: &str,
    app_password: &str,
) -> Result<(), String> {
    let mkcol = reqwest::Method::from_bytes(b"MKCOL").map_err(|e| e.to_string())?;
    let response = client()?
        .request(mkcol, folder.clone())
        .basic_auth(login_name, Some(app_password))
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    match response.status().as_u16() {
        // Created, or already there.
        201 | 405 => Ok(()),
        401 | 403 => Err("backup.sync_auth_failed".into()),
        status => Err(format!("backup.sync_network_failed: HTTP {status}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_addresses_are_cleaned_up() {
        let cases = [
            ("nuvem.exemplo.com", "https://nuvem.exemplo.com/"),
            ("https://nuvem.exemplo.com/", "https://nuvem.exemplo.com/"),
            (
                "https://exemplo.com/nextcloud/index.php/apps/files/?dir=/",
                "https://exemplo.com/nextcloud/",
            ),
            ("https://exemplo.com/login", "https://exemplo.com/"),
            ("http://localhost:8080", "http://localhost:8080/"),
        ];
        for (typed, expected) in cases {
            assert_eq!(
                normalize_server(typed).unwrap().as_str(),
                expected,
                "{typed}"
            );
        }
        assert!(
            normalize_server("http://nuvem.exemplo.com").is_err(),
            "plain HTTP outside this computer"
        );
        assert!(normalize_server("").is_err());
    }

    #[test]
    fn the_sync_folder_is_inside_the_account_files() {
        let server = normalize_server("https://exemplo.com/nextcloud").unwrap();
        assert_eq!(
            webdav_folder(&server, "ana maria@exemplo.com")
                .unwrap()
                .as_str(),
            "https://exemplo.com/nextcloud/remote.php/dav/files/ana%20maria@exemplo.com/Spark/"
        );
    }

    #[test]
    fn another_origin_is_not_the_same_server() {
        let server = url::Url::parse("https://nuvem.exemplo.com/").unwrap();
        assert!(same_origin(
            &server,
            &url::Url::parse("https://nuvem.exemplo.com/login/v2/flow/x").unwrap()
        ));
        assert!(!same_origin(
            &server,
            &url::Url::parse("https://evil.test/login").unwrap()
        ));
        assert!(!same_origin(
            &server,
            &url::Url::parse("http://nuvem.exemplo.com/").unwrap()
        ));
    }

    #[test]
    fn a_poll_for_another_or_no_login_is_refused() {
        let login = NextcloudLogin::default();
        assert_eq!(
            login.poll(1).err().as_deref(),
            Some("backup.nextcloud_login_cancelled")
        );
    }
}
