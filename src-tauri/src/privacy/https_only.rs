//! HTTPS-only mode: top-level `http://` navigations are upgraded to `https://`.
//!
//! When the upgraded navigation fails, the interface asks the user whether to continue with the
//! original `http://` address. The answer is a one-use exception bound to one tab and one exact
//! URL: any other navigation, even to the same address later, is upgraded again.
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};
use tauri::Url;

/// A failure only counts for an upgrade that started recently, so an old, unrelated failure
/// never opens the prompt.
const UPGRADE_WINDOW: Duration = Duration::from_secs(60);

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Upgrade(Url),
    /// There is no safe HTTPS form of the address (it carries credentials): it never loads.
    Block,
}

struct Attempt {
    original: Url,
    upgraded: Url,
    started: Instant,
}

#[derive(Default)]
pub struct HttpsOnly {
    enabled: AtomicBool,
    attempts: Mutex<HashMap<String, Attempt>>,
    exceptions: Mutex<HashMap<String, Url>>,
}

/// Local development servers and devices on the machine have no certificate to check.
fn is_exempt(url: &Url) -> bool {
    match url.host() {
        Some(url::Host::Domain(name)) => {
            let name = name.trim_end_matches('.').to_ascii_lowercase();
            name == "localhost" || name.ends_with(".localhost")
        }
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        None => true,
    }
}

/// The upgrade keeps host, path, query and fragment. An explicit port is kept as the user wrote it;
/// the default HTTP port (80) was never written, so the HTTPS default (443) applies.
fn upgraded(url: &Url) -> Option<Url> {
    if url.scheme() != "http"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    let mut upgraded = url.clone();
    upgraded.set_scheme("https").ok()?;
    Some(upgraded)
}

impl HttpsOnly {
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    pub fn set_enabled(&self, value: bool) {
        self.enabled.store(value, Ordering::Release);
        if !value {
            if let Ok(mut attempts) = self.attempts.lock() {
                attempts.clear();
            }
            if let Ok(mut exceptions) = self.exceptions.lock() {
                exceptions.clear();
            }
        }
    }

    /// Called for every top-level navigation of a tab before it starts.
    pub fn decide(&self, tab: &str, url: &Url) -> Decision {
        if !self.enabled() || url.scheme() != "http" || is_exempt(url) {
            return Decision::Allow;
        }
        if let Ok(mut exceptions) = self.exceptions.lock() {
            // A permission is consumed by the first navigation that follows it, whatever it targets.
            if exceptions
                .remove(tab)
                .is_some_and(|allowed| allowed == *url)
            {
                return Decision::Allow;
            }
        }
        let Some(target) = upgraded(url) else {
            return Decision::Block;
        };
        if let Ok(mut attempts) = self.attempts.lock() {
            attempts.insert(
                tab.to_string(),
                Attempt {
                    original: url.clone(),
                    upgraded: target.clone(),
                    started: Instant::now(),
                },
            );
        }
        Decision::Upgrade(target)
    }

    /// The user chose to continue with this exact HTTP address, once.
    pub fn allow_once(&self, tab: &str, url: &Url) -> bool {
        if !self.enabled() || url.scheme() != "http" {
            return false;
        }
        let known = self
            .attempts
            .lock()
            .map(|attempts| attempts.get(tab).is_some_and(|a| a.original == *url))
            .unwrap_or(false);
        if !known {
            return false;
        }
        if let Ok(mut exceptions) = self.exceptions.lock() {
            exceptions.insert(tab.to_string(), url.clone());
        }
        true
    }

    /// A navigation of the tab failed. Returns the HTTP address to offer when it was an upgrade.
    pub fn failed(&self, tab: &str) -> Option<Url> {
        let mut attempts = self.attempts.lock().ok()?;
        let fresh = attempts
            .get(tab)
            .is_some_and(|attempt| attempt.started.elapsed() <= UPGRADE_WINDOW);
        if fresh {
            attempts.get(tab).map(|attempt| attempt.original.clone())
        } else {
            attempts.remove(tab);
            None
        }
    }

    /// A navigation finished correctly: the pending upgrade (if any) worked.
    pub fn succeeded(&self, tab: &str, finished: &Url) {
        if let Ok(mut attempts) = self.attempts.lock() {
            if attempts
                .get(tab)
                .is_some_and(|attempt| attempt.upgraded.host_str() == finished.host_str())
            {
                attempts.remove(tab);
            }
        }
    }

    pub fn forget(&self, tab: &str) {
        if let Ok(mut attempts) = self.attempts.lock() {
            attempts.remove(tab);
        }
        if let Ok(mut exceptions) = self.exceptions.lock() {
            exceptions.remove(tab);
        }
    }
}

/// The interface learns that an upgrade could not be reached and offers the HTTP address once.
#[derive(serde::Serialize, Clone)]
struct Blocked {
    tab_id: String,
    url: String,
}

/// Navigation of a tab failed (not cancelled): offer the HTTP address if it was an upgrade.
pub fn navigation_failed(app: &tauri::AppHandle, tab_id: &str) {
    use tauri::Manager;
    let Some(state) = app.try_state::<crate::state::browser_state::BrowserState>() else {
        return;
    };
    if let Some(url) = state.policies().https_only().failed(tab_id) {
        crate::error_log::ignored(crate::security::emit_shell_for_tab(
            app,
            tab_id,
            "https-only-blocked",
            Blocked {
                tab_id: tab_id.to_string(),
                url: url.to_string(),
            },
        ));
    }
}

/// Navigation of a tab finished: a pending upgrade to this address worked.
pub fn navigation_succeeded(app: &tauri::AppHandle, tab_id: &str, finished: &Url) {
    use tauri::Manager;
    if let Some(state) = app.try_state::<crate::state::browser_state::BrowserState>() {
        state.policies().https_only().succeeded(tab_id, finished);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on() -> HttpsOnly {
        let policy = HttpsOnly::default();
        policy.set_enabled(true);
        policy
    }

    fn url(value: &str) -> Url {
        Url::parse(value).unwrap()
    }

    #[test]
    fn nothing_changes_while_the_mode_is_off() {
        let policy = HttpsOnly::default();
        assert_eq!(
            policy.decide("t", &url("http://example.test/")),
            Decision::Allow
        );
    }

    #[test]
    fn http_is_upgraded_keeping_every_part_of_the_address() {
        let policy = on();
        assert_eq!(
            policy.decide("t", &url("http://example.test:8080/path?q=1#part")),
            Decision::Upgrade(url("https://example.test:8080/path?q=1#part"))
        );
        assert_eq!(
            policy.decide("t", &url("http://example.test/")),
            Decision::Upgrade(url("https://example.test/"))
        );
    }

    #[test]
    fn https_and_the_local_machine_are_never_touched() {
        let policy = on();
        for value in [
            "https://example.test/",
            "http://localhost:3000/",
            "http://app.localhost/",
            "http://127.0.0.1:8080/",
            "http://[::1]:8080/",
        ] {
            assert_eq!(policy.decide("t", &url(value)), Decision::Allow, "{value}");
        }
        assert_eq!(
            policy.decide("t", &url("http://192.0.2.1/")),
            Decision::Upgrade(url("https://192.0.2.1/"))
        );
    }

    #[test]
    fn an_address_with_credentials_is_not_loaded_over_http() {
        let policy = on();
        let secret = url("http://user:secret@example.test/");
        assert_eq!(policy.decide("t", &secret), Decision::Block);
        assert!(!policy.allow_once("t", &secret));
    }

    #[test]
    fn a_failed_upgrade_offers_the_original_address_only_while_fresh() {
        let policy = on();
        assert_eq!(policy.failed("t"), None);
        policy.decide("t", &url("http://example.test/a"));
        assert_eq!(policy.failed("t"), Some(url("http://example.test/a")));
        policy
            .attempts
            .lock()
            .unwrap()
            .get_mut("t")
            .unwrap()
            .started = Instant::now() - UPGRADE_WINDOW - Duration::from_secs(1);
        assert_eq!(policy.failed("t"), None);
        assert_eq!(policy.failed("t"), None);
    }

    #[test]
    fn a_working_upgrade_clears_the_attempt() {
        let policy = on();
        policy.decide("t", &url("http://example.test/a"));
        policy.succeeded("t", &url("https://other.test/"));
        assert!(policy.failed("t").is_some());
        policy.succeeded("t", &url("https://example.test/a"));
        assert_eq!(policy.failed("t"), None);
    }

    #[test]
    fn the_exception_is_exact_bound_to_the_tab_and_used_once() {
        let policy = on();
        let page = url("http://example.test/a");
        // Nothing to continue with before an upgrade happened.
        assert!(!policy.allow_once("t", &page));
        policy.decide("t", &page);
        // Another tab or another address gets nothing.
        assert!(!policy.allow_once("other", &page));
        assert!(!policy.allow_once("t", &url("http://example.test/b")));
        assert!(policy.allow_once("t", &page));
        // Another tab still gets upgraded.
        assert_eq!(
            policy.decide("other", &page),
            Decision::Upgrade(url("https://example.test/a"))
        );
        assert_eq!(policy.decide("t", &page), Decision::Allow);
        // Used: the same address is upgraded again.
        assert_eq!(
            policy.decide("t", &page),
            Decision::Upgrade(url("https://example.test/a"))
        );
    }

    #[test]
    fn a_different_navigation_consumes_the_exception() {
        let policy = on();
        let page = url("http://example.test/a");
        policy.decide("t", &page);
        assert!(policy.allow_once("t", &page));
        assert!(matches!(
            policy.decide("t", &url("http://example.test/b")),
            Decision::Upgrade(_)
        ));
        assert!(matches!(policy.decide("t", &page), Decision::Upgrade(_)));
    }

    #[test]
    fn turning_the_mode_off_drops_attempts_and_exceptions() {
        let policy = on();
        let page = url("http://example.test/a");
        policy.decide("t", &page);
        policy.allow_once("t", &page);
        policy.set_enabled(false);
        assert_eq!(policy.failed("t"), None);
        policy.set_enabled(true);
        assert!(matches!(policy.decide("t", &page), Decision::Upgrade(_)));
    }

    #[test]
    fn closing_a_tab_forgets_its_state() {
        let policy = on();
        policy.decide("t", &url("http://example.test/a"));
        policy.forget("t");
        assert_eq!(policy.failed("t"), None);
    }
}
