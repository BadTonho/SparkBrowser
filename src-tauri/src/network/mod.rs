//! DNS over HTTPS.
//!
//! WebView2 and WebKitGTK have no setting for secure DNS, so the browser runs a small proxy on the
//! loopback address that resolves every name through DNS over HTTPS (`proxy`, `resolver`) and points
//! the web engines at it. Like the graphics choice, the proxy address is part of how the engine
//! starts, so the choice lives in a small file read before any WebView exists and a change applies
//! after a restart.
//!
//! Covered: pages, frames, scripts, images, WebSockets and downloads that go through the engine's
//! network stack. Not covered: WebRTC media (UDP), the browser's own requests made from Rust
//! (updates, favicons, search suggestions) and anything outside the browser.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    path::Path,
    sync::{Arc, Mutex},
};

mod proxy;
pub mod resolver;

const FILE: &str = "network.json";
/// `host:port` of the running proxy. Private windows and restarted children inherit it.
const PROXY_VARIABLE: &str = "SPARK_DOH_PROXY";
const PROVIDER_VARIABLE: &str = "SPARK_DOH_PROVIDER";
#[cfg(windows)]
const ARGUMENTS_VARIABLE: &str = "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Cloudflare,
    Google,
}

const CLOUDFLARE: [IpAddr; 2] = [
    IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
    IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)),
];
const GOOGLE: [IpAddr; 2] = [
    IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
    IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4)),
];

impl Provider {
    pub fn id(self) -> &'static str {
        match self {
            Provider::Cloudflare => "cloudflare",
            Provider::Google => "google",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "cloudflare" => Some(Provider::Cloudflare),
            "google" => Some(Provider::Google),
            _ => None,
        }
    }

    /// The name on the server's certificate.
    pub fn host(self) -> &'static str {
        match self {
            Provider::Cloudflare => "cloudflare-dns.com",
            Provider::Google => "dns.google",
        }
    }

    pub fn endpoint(self) -> &'static str {
        match self {
            Provider::Cloudflare => "https://cloudflare-dns.com/dns-query",
            Provider::Google => "https://dns.google/resolve",
        }
    }

    /// Fixed addresses of the server: asking for the provider never uses the system DNS.
    pub fn bootstrap(self) -> &'static [IpAddr] {
        match self {
            Provider::Cloudflare => &CLOUDFLARE,
            Provider::Google => &GOOGLE,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Stored {
    #[serde(default)]
    doh_provider: Option<Provider>,
}

/// The saved provider, or `None` (off) when there is no file or it is damaged. A leading byte order
/// mark, which some editors add, is ignored.
pub fn load(directory: &Path) -> Option<Provider> {
    fs::read_to_string(directory.join(FILE))
        .ok()
        .and_then(|raw| serde_json::from_str::<Stored>(raw.trim_start_matches('\u{feff}')).ok())
        .and_then(|stored| stored.doh_provider)
}

pub fn save(directory: &Path, provider: Option<Provider>) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|_| "network.write_failed")?;
    let raw = serde_json::to_vec_pretty(&Stored {
        doh_provider: provider,
    })
    .map_err(|e| e.to_string())?;
    crate::backup::files::write_bytes(&directory.join(FILE), &raw)
        .map_err(|_| "network.write_failed".into())
}

fn proxy_flag(address: &str) -> String {
    format!("--proxy-server=http://{address}")
}

/// The engine arguments with the proxy of an earlier run taken away and the new one added.
#[cfg(any(windows, test))]
fn merge_arguments(existing: &str, inherited: Option<&str>, current: Option<&str>) -> String {
    let old = inherited.map(proxy_flag);
    let mut tokens: Vec<String> = existing
        .split_whitespace()
        .filter(|token| old.as_deref() != Some(*token))
        .map(str::to_string)
        .collect();
    if let Some(address) = current {
        tokens.push(proxy_flag(address));
    }
    tokens.join(" ")
}

static LISTENER: Mutex<Option<TcpListener>> = Mutex::new(None);

/// Puts the proxy into the process environment. Must run before the first WebView is created, and
/// only in the main process: private windows inherit what their parent set.
pub fn apply_environment(provider: Option<Provider>) {
    let inherited = std::env::var(PROXY_VARIABLE).ok();
    let mut running: Option<(SocketAddr, Provider)> = None;
    if let Some(provider) = provider {
        match TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).and_then(|listener| {
            listener.set_nonblocking(true)?;
            Ok((listener.local_addr()?, listener))
        }) {
            Ok((address, listener)) => {
                if let Ok(mut slot) = LISTENER.lock() {
                    *slot = Some(listener);
                }
                running = Some((address, provider));
            }
            Err(error) => crate::error_log::report(
                crate::error_log::Level::Error,
                "native",
                "DNS over HTTPS is on but its local proxy could not start",
                &error.to_string(),
            ),
        }
    }
    let address = running.map(|(address, _)| address.to_string());
    #[cfg(windows)]
    {
        let current = std::env::var(ARGUMENTS_VARIABLE).unwrap_or_default();
        let next = merge_arguments(&current, inherited.as_deref(), address.as_deref());
        if next.is_empty() {
            std::env::remove_var(ARGUMENTS_VARIABLE);
        } else {
            std::env::set_var(ARGUMENTS_VARIABLE, next);
        }
    }
    #[cfg(not(windows))]
    let _ = &inherited;
    match (address, running) {
        (Some(address), Some((_, provider))) => {
            std::env::set_var(PROXY_VARIABLE, address);
            std::env::set_var(PROVIDER_VARIABLE, provider.id());
        }
        _ => {
            std::env::remove_var(PROXY_VARIABLE);
            std::env::remove_var(PROVIDER_VARIABLE);
        }
    }
}

/// What this process (or the parent that started it) runs with.
pub fn running() -> Option<(SocketAddr, Provider)> {
    let address = std::env::var(PROXY_VARIABLE).ok()?.parse().ok()?;
    let provider = Provider::from_id(&std::env::var(PROVIDER_VARIABLE).ok()?)?;
    Some((address, provider))
}

/// The proxy for engines that take it per WebView (WebKitGTK). WebView2 reads it from its arguments.
#[cfg(not(windows))]
pub fn proxy_url() -> Option<tauri::Url> {
    let (address, _) = running()?;
    tauri::Url::parse(&format!("http://{address}")).ok()
}

/// Starts serving on the address reserved by `apply_environment`. Only the main process has one.
pub fn start() {
    let Some(listener) = LISTENER.lock().ok().and_then(|mut slot| slot.take()) else {
        return;
    };
    let Some((_, provider)) = running() else {
        return;
    };
    let resolver = match resolver::DohResolver::new(provider) {
        Ok(resolver) => Arc::new(resolver) as Arc<dyn resolver::Resolve>,
        Err(error) => {
            crate::error_log::report(
                crate::error_log::Level::Error,
                "native",
                "DNS over HTTPS could not prepare its secure connection",
                &error,
            );
            return;
        }
    };
    tauri::async_runtime::spawn(async move {
        match tokio::net::TcpListener::from_std(listener) {
            Ok(listener) => proxy::serve(listener, resolver).await,
            Err(error) => crate::error_log::report(
                crate::error_log::Level::Error,
                "native",
                "DNS over HTTPS proxy could not listen",
                &error.to_string(),
            ),
        }
    });
}

/// What the interface shows: the saved choice, what runs now and whether a restart would change it.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    pub doh_provider: Option<String>,
    pub active_provider: Option<String>,
    pub restart_required: bool,
}

pub fn status(wanted: Option<Provider>) -> NetworkStatus {
    let active = running().map(|(_, provider)| provider);
    NetworkStatus {
        doh_provider: wanted.map(|p| p.id().to_string()),
        active_provider: active.map(|p| p.id().to_string()),
        restart_required: wanted != active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_is_off_without_a_file_or_with_a_damaged_one() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()), None);
        fs::write(dir.path().join(FILE), "not json").unwrap();
        assert_eq!(load(dir.path()), None);
        fs::write(dir.path().join(FILE), r#"{"doh_provider":"unknown"}"#).unwrap();
        assert_eq!(load(dir.path()), None);
        fs::write(
            dir.path().join(FILE),
            "\u{feff}{\"doh_provider\":\"google\"}",
        )
        .unwrap();
        assert_eq!(load(dir.path()), Some(Provider::Google));
    }

    #[test]
    fn the_choice_roundtrips_and_creates_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("profile");
        save(&nested, Some(Provider::Cloudflare)).unwrap();
        assert_eq!(load(&nested), Some(Provider::Cloudflare));
        save(&nested, None).unwrap();
        assert_eq!(load(&nested), None);
    }

    #[test]
    fn every_provider_names_a_certificate_host_that_matches_its_endpoint() {
        for provider in [Provider::Cloudflare, Provider::Google] {
            assert_eq!(Provider::from_id(provider.id()), Some(provider));
            let endpoint = tauri::Url::parse(provider.endpoint()).unwrap();
            assert_eq!(endpoint.scheme(), "https");
            assert_eq!(endpoint.host_str(), Some(provider.host()));
            assert!(provider.bootstrap().iter().all(|ip| !ip.is_loopback()));
        }
        assert_eq!(Provider::from_id("quad9"), None);
    }

    #[test]
    fn our_flag_replaces_an_inherited_one_and_leaves_the_users_arguments_alone() {
        let own = "--remote-debugging-port=9222";
        assert_eq!(
            merge_arguments("", None, Some("127.0.0.1:5000")),
            "--proxy-server=http://127.0.0.1:5000"
        );
        assert_eq!(
            merge_arguments(
                &format!("{own} --proxy-server=http://127.0.0.1:4000"),
                Some("127.0.0.1:4000"),
                Some("127.0.0.1:5000")
            ),
            format!("{own} --proxy-server=http://127.0.0.1:5000")
        );
        // Turned off after a run with it on: only our flag goes away.
        assert_eq!(
            merge_arguments(
                &format!("{own} --proxy-server=http://127.0.0.1:4000"),
                Some("127.0.0.1:4000"),
                None
            ),
            own
        );
        // A proxy the user set themselves is never touched.
        assert_eq!(
            merge_arguments("--proxy-server=http://corp:3128", None, None),
            "--proxy-server=http://corp:3128"
        );
    }

    #[test]
    fn a_restart_is_needed_only_when_the_choice_differs_from_what_runs() {
        // No proxy runs in tests.
        assert!(!status(None).restart_required);
        assert!(status(Some(Provider::Google)).restart_required);
        assert_eq!(
            status(Some(Provider::Google)).doh_provider.as_deref(),
            Some("google")
        );
    }
}
