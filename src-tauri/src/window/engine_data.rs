//! Clears what the web engine keeps on disk for pages: its cache and the data that sites store
//! (cookies, local storage, databases, service workers).
//!
//! The engine keeps one store per data folder, and workspaces with isolated data have a folder of their
//! own, so every distinct store reachable from an open page is cleared once. Stores that only belong to
//! hibernated tabs cannot be reached until those tabs wake up.
//!
//! Local storage of the browser's own interface lives in the same store as the sites' when the data is
//! shared, so the interface saves and restores its own settings around this call (see the panel).
use tauri::{AppHandle, Manager};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wanted {
    pub cache: bool,
    pub site_data: bool,
    /// Only what is older than this many days; `None` clears everything. Windows only: the engine on Linux can
    /// only clear what changed in the last N days, which is the opposite, so there it is refused.
    pub older_than_days: Option<u32>,
}

impl Wanted {
    pub fn is_empty(self) -> bool {
        !self.cache && !self.site_data
    }
}

/// What each store reported.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Stores that were cleared.
    pub cleared: usize,
    /// Stores the engine refused or that did not answer in time.
    pub failed: usize,
}

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(25);

enum Event {
    /// This page's store was already handled through another page.
    Skipped,
    Started,
    Done(bool),
}

/// Waits for every page to report, then for every started store to finish.
fn collect(receiver: &std::sync::mpsc::Receiver<Event>, pages: usize) -> Outcome {
    let deadline = std::time::Instant::now() + TIMEOUT;
    let (mut reported, mut started, mut finished) = (0usize, 0usize, 0usize);
    let mut outcome = Outcome::default();
    while reported < pages || finished < started {
        let wait = deadline.saturating_duration_since(std::time::Instant::now());
        match receiver.recv_timeout(wait) {
            Ok(Event::Skipped) => reported += 1,
            Ok(Event::Started) => {
                reported += 1;
                started += 1;
            }
            Ok(Event::Done(true)) => {
                finished += 1;
                outcome.cleared += 1;
            }
            Ok(Event::Done(false)) => {
                finished += 1;
                outcome.failed += 1;
            }
            Err(_) => {
                outcome.failed += started
                    .saturating_sub(finished)
                    .max(pages - reported.min(pages));
                break;
            }
        }
    }
    outcome
}

/// The moment `days` days ago, in seconds since 1970.
#[cfg(windows)]
fn cutoff_seconds(days: u32) -> f64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs_f64())
        .unwrap_or(0.0);
    (now - f64::from(days) * 86_400.0).max(0.0)
}

pub(super) fn pages(app: &AppHandle) -> Vec<tauri::Webview> {
    app.webviews()
        .into_iter()
        .filter(|(label, _)| {
            label.starts_with(super::webview_manager::CONTENT_WEBVIEW_PREFIX)
                || crate::state::tab_model::is_window_label(label)
        })
        .map(|(_, view)| view)
        .collect()
}

#[cfg(windows)]
pub async fn clear(app: &AppHandle, wanted: Wanted) -> Result<Outcome, String> {
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use webview2_com::{
        ClearBrowsingDataCompletedHandler, CoTaskMemPWSTR,
        Microsoft::Web::WebView2::Win32::{
            ICoreWebView2Profile2, ICoreWebView2_13, COREWEBVIEW2_BROWSING_DATA_KINDS,
            COREWEBVIEW2_BROWSING_DATA_KINDS_CACHE_STORAGE,
            COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES, COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE,
            COREWEBVIEW2_BROWSING_DATA_KINDS_FILE_SYSTEMS,
            COREWEBVIEW2_BROWSING_DATA_KINDS_INDEXED_DB,
            COREWEBVIEW2_BROWSING_DATA_KINDS_LOCAL_STORAGE,
            COREWEBVIEW2_BROWSING_DATA_KINDS_SERVICE_WORKERS,
            COREWEBVIEW2_BROWSING_DATA_KINDS_WEB_SQL,
        },
    };
    use windows_core::{Interface, PWSTR};

    let older_than_days = wanted.older_than_days;
    let mut kinds = 0i32;
    if wanted.cache {
        kinds |= COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE.0
            | COREWEBVIEW2_BROWSING_DATA_KINDS_CACHE_STORAGE.0;
    }
    if wanted.site_data {
        kinds |= COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES.0
            | COREWEBVIEW2_BROWSING_DATA_KINDS_LOCAL_STORAGE.0
            | COREWEBVIEW2_BROWSING_DATA_KINDS_INDEXED_DB.0
            | COREWEBVIEW2_BROWSING_DATA_KINDS_WEB_SQL.0
            | COREWEBVIEW2_BROWSING_DATA_KINDS_FILE_SYSTEMS.0
            | COREWEBVIEW2_BROWSING_DATA_KINDS_SERVICE_WORKERS.0;
    }
    if kinds == 0 {
        return Ok(Outcome::default());
    }
    let views = pages(app);
    if views.is_empty() {
        return Err("browser_data.no_page".into());
    }
    let (sender, receiver) = std::sync::mpsc::channel();
    let seen = Arc::new(Mutex::new(HashSet::<String>::new()));
    let count = views.len();
    for view in views {
        let sender = sender.clone();
        let seen = Arc::clone(&seen);
        let failed = sender.clone();
        let queued = view.with_webview(move |platform| unsafe {
            let started = (|| -> windows_core::Result<bool> {
                let profile = platform
                    .controller()
                    .CoreWebView2()?
                    .cast::<ICoreWebView2_13>()?
                    .Profile()?;
                let mut path = PWSTR::null();
                profile.ProfilePath(&mut path)?;
                let path = CoTaskMemPWSTR::from(path).to_string().to_lowercase();
                if !seen
                    .lock()
                    .map(|mut seen| seen.insert(path))
                    .unwrap_or(true)
                {
                    return Ok(false);
                }
                let done = sender.clone();
                let handler = ClearBrowsingDataCompletedHandler::create(Box::new(move |result| {
                    crate::error_log::ignored(done.send(Event::Done(result.is_ok())));
                    Ok(())
                }));
                let profile = profile.cast::<ICoreWebView2Profile2>()?;
                match older_than_days {
                    None => profile
                        .ClearBrowsingData(COREWEBVIEW2_BROWSING_DATA_KINDS(kinds), &handler)?,
                    // From the beginning of time up to `days` days ago, in seconds since 1970.
                    Some(days) => profile.ClearBrowsingDataInTimeRange(
                        COREWEBVIEW2_BROWSING_DATA_KINDS(kinds),
                        0.0,
                        cutoff_seconds(days),
                        &handler,
                    )?,
                }
                Ok(true)
            })();
            match started {
                Ok(true) => crate::error_log::ignored(sender.send(Event::Started)),
                Ok(false) => crate::error_log::ignored(sender.send(Event::Skipped)),
                Err(error) => {
                    crate::error_log::warn(&format!("Could not clear a page's data: {error}"));
                    crate::error_log::ignored(sender.send(Event::Started));
                    crate::error_log::ignored(sender.send(Event::Done(false)));
                }
            }
        });
        if let Err(error) = queued {
            crate::error_log::warn(&format!(
                "Could not reach a page to clear its data: {error}"
            ));
            crate::error_log::ignored(failed.send(Event::Started));
            crate::error_log::ignored(failed.send(Event::Done(false)));
        }
    }
    drop(sender);
    tauri::async_runtime::spawn_blocking(move || collect(&receiver, count))
        .await
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "linux")]
pub async fn clear(app: &AppHandle, wanted: Wanted) -> Result<Outcome, String> {
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use webkit2gtk::{
        WebViewExt, WebsiteDataManagerExt, WebsiteDataManagerExtManual, WebsiteDataTypes,
    };

    if wanted.older_than_days.is_some() {
        return Err("browser_data.age_unsupported".into());
    }
    let mut types = WebsiteDataTypes::empty();
    if wanted.cache {
        types |= WebsiteDataTypes::MEMORY_CACHE
            | WebsiteDataTypes::DISK_CACHE
            | WebsiteDataTypes::DOM_CACHE
            | WebsiteDataTypes::OFFLINE_APPLICATION_CACHE;
    }
    if wanted.site_data {
        types |= WebsiteDataTypes::COOKIES
            | WebsiteDataTypes::LOCAL_STORAGE
            | WebsiteDataTypes::SESSION_STORAGE
            | WebsiteDataTypes::INDEXEDDB_DATABASES
            | WebsiteDataTypes::WEBSQL_DATABASES
            | WebsiteDataTypes::SERVICE_WORKER_REGISTRATIONS;
    }
    if types.is_empty() {
        return Ok(Outcome::default());
    }
    let views = pages(app);
    if views.is_empty() {
        return Err("browser_data.no_page".into());
    }
    let (sender, receiver) = std::sync::mpsc::channel();
    let seen = Arc::new(Mutex::new(HashSet::<String>::new()));
    let count = views.len();
    for view in views {
        let sender = sender.clone();
        let seen = Arc::clone(&seen);
        let failed = sender.clone();
        let queued = view.with_webview(move |platform| {
            let Some(manager) = platform.inner().website_data_manager() else {
                crate::error_log::ignored(sender.send(Event::Started));
                crate::error_log::ignored(sender.send(Event::Done(false)));
                return;
            };
            let folder = manager
                .base_data_directory()
                .map(|path| path.to_string())
                .unwrap_or_default();
            if !folder.is_empty()
                && !seen
                    .lock()
                    .map(|mut seen| seen.insert(folder))
                    .unwrap_or(true)
            {
                crate::error_log::ignored(sender.send(Event::Skipped));
                return;
            }
            crate::error_log::ignored(sender.send(Event::Started));
            let done = sender.clone();
            manager.clear(
                types,
                webkit2gtk::glib::TimeSpan(0),
                None::<&webkit2gtk::gio::Cancellable>,
                move |result| {
                    crate::error_log::ignored(done.send(Event::Done(result.is_ok())));
                },
            );
        });
        if let Err(error) = queued {
            crate::error_log::warn(&format!(
                "Could not reach a page to clear its data: {error}"
            ));
            crate::error_log::ignored(failed.send(Event::Started));
            crate::error_log::ignored(failed.send(Event::Done(false)));
        }
    }
    drop(sender);
    tauri::async_runtime::spawn_blocking(move || collect(&receiver, count))
        .await
        .map_err(|error| error.to_string())
}

#[cfg(not(any(windows, target_os = "linux")))]
pub async fn clear(_app: &AppHandle, _wanted: Wanted) -> Result<Outcome, String> {
    Err("browser_data.unsupported_platform".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn nothing_wanted_is_empty() {
        assert!(Wanted {
            cache: false,
            site_data: false,
            older_than_days: None
        }
        .is_empty());
        assert!(!Wanted {
            cache: true,
            site_data: false,
            older_than_days: Some(5)
        }
        .is_empty());
    }

    #[test]
    fn every_started_store_is_waited_for_and_shared_stores_count_once() {
        let (send, receive) = channel();
        // Three pages: two share a store, one has its own that fails.
        send.send(Event::Started).unwrap();
        send.send(Event::Skipped).unwrap();
        send.send(Event::Started).unwrap();
        send.send(Event::Done(true)).unwrap();
        send.send(Event::Done(false)).unwrap();
        assert_eq!(
            collect(&receive, 3),
            Outcome {
                cleared: 1,
                failed: 1
            }
        );
    }
}
