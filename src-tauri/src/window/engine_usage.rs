//! How much disk the web engine's cache and site data take, so the panel can say how much a cleaning frees.
//!
//! The sizes are measured on the engine's own folders (the same stores [`super::engine_data`] clears), so they
//! are real bytes, not a guess. With an age, only the files last changed before that are counted, which is how
//! the engine dates its entries (a cache entry is one file; a database is one file that changes as it is used,
//! so a recently used one counts as new). Cookies are a single small database and are only counted when
//! nothing is filtered by age.
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime};
use tauri::AppHandle;

/// What was measured, in bytes.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub cache_bytes: u64,
    pub site_bytes: u64,
    /// The measuring stopped at its time limit, so the real numbers are at least these.
    pub partial: bool,
}

/// The folders that make up one store, split the way the panel asks for them.
#[derive(Debug, Default)]
struct Folders {
    cache: Vec<PathBuf>,
    site: Vec<PathBuf>,
}

const REACH_TIMEOUT: Duration = Duration::from_secs(5);
const WALK_BUDGET: Duration = Duration::from_secs(6);

/// The folders of a Windows engine profile (`.../EBWebView/Default`).
#[cfg_attr(not(windows), allow(dead_code))]
fn profile_folders(profile: &Path) -> Folders {
    let at = |name: &str| profile.join(name);
    Folders {
        cache: vec![
            at("Cache"),
            at("Code Cache"),
            at("GPUCache"),
            at("DawnGraphiteCache"),
            at("DawnWebGPUCache"),
            at("Service Worker").join("CacheStorage"),
        ],
        site: vec![
            at("Local Storage"),
            at("Session Storage"),
            at("IndexedDB"),
            at("File System"),
            at("databases"),
            at("WebStorage"),
            at("Service Worker").join("Database"),
            at("Service Worker").join("ScriptCache"),
            at("Network").join("Cookies"),
            at("Network").join("Cookies-journal"),
        ],
    }
}

/// Adds up the size of a file or folder, counting only the files last changed before `cutoff` when there is one.
/// Stops when `deadline` passes and says so.
fn tree_size(path: &Path, cutoff: Option<SystemTime>, deadline: Instant, partial: &mut bool) -> u64 {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if meta.is_file() {
        let old_enough = cutoff.is_none_or(|limit| meta.modified().is_ok_and(|changed| changed < limit));
        return if old_enough { meta.len() } else { 0 };
    }
    if !meta.is_dir() {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    let mut total = 0;
    for entry in entries.flatten() {
        if Instant::now() >= deadline {
            *partial = true;
            break;
        }
        total += tree_size(&entry.path(), cutoff, deadline, partial);
    }
    total
}

fn measure_folders(stores: Vec<Folders>, older_than_days: Option<u32>) -> Usage {
    let deadline = Instant::now() + WALK_BUDGET;
    let cutoff = older_than_days
        .map(|days| SystemTime::now() - Duration::from_secs(u64::from(days) * 86_400));
    let mut usage = Usage::default();
    for store in &stores {
        for folder in &store.cache {
            usage.cache_bytes += tree_size(folder, cutoff, deadline, &mut usage.partial);
        }
        for folder in &store.site {
            // The cookie database is one file that changes all the time: its age says nothing, so it only counts unfiltered.
            let is_cookies = folder.file_name().is_some_and(|name| name.to_string_lossy().starts_with("Cookies"));
            if is_cookies && cutoff.is_some() {
                continue;
            }
            usage.site_bytes += tree_size(folder, cutoff, deadline, &mut usage.partial);
        }
    }
    usage
}

/// Asks every open page for the folders of its store, once per distinct store.
#[cfg(windows)]
fn reach_stores(app: &AppHandle) -> Result<Vec<Folders>, String> {
    use webview2_com::{
        CoTaskMemPWSTR, Microsoft::Web::WebView2::Win32::ICoreWebView2_13,
    };
    use windows_core::{Interface, PWSTR};

    let views = super::engine_data::pages(app);
    if views.is_empty() {
        return Err("browser_data.no_page".into());
    }
    let (sender, receiver) = mpsc::channel::<Option<PathBuf>>();
    let mut asked = 0;
    for view in views {
        let sender = sender.clone();
        let failed = sender.clone();
        let queued = view.with_webview(move |platform| unsafe {
            let path = (|| -> windows_core::Result<PathBuf> {
                let profile = platform
                    .controller()
                    .CoreWebView2()?
                    .cast::<ICoreWebView2_13>()?
                    .Profile()?;
                let mut path = PWSTR::null();
                profile.ProfilePath(&mut path)?;
                Ok(PathBuf::from(CoTaskMemPWSTR::from(path).to_string()))
            })();
            if let Err(error) = &path {
                crate::error_log::warn(&format!("Could not find a page's data folder to measure it: {error}"));
            }
            crate::error_log::ignored(sender.send(path.ok()));
        });
        match queued {
            Ok(()) => asked += 1,
            Err(error) => {
                crate::error_log::warn(&format!("Could not reach a page to measure its data: {error}"));
                crate::error_log::ignored(failed.send(None));
                asked += 1;
            }
        }
    }
    drop(sender);
    let deadline = Instant::now() + REACH_TIMEOUT;
    let mut seen = HashSet::new();
    let mut stores = Vec::new();
    for _ in 0..asked {
        let wait = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(wait) {
            Ok(Some(path)) => {
                if seen.insert(path.to_string_lossy().to_lowercase()) {
                    stores.push(profile_folders(&path));
                }
            }
            Ok(None) => {}
            Err(_) => break,
        }
    }
    Ok(stores)
}

#[cfg(target_os = "linux")]
fn reach_stores(app: &AppHandle) -> Result<Vec<Folders>, String> {
    use webkit2gtk::{WebViewExt, WebsiteDataManagerExt};

    let views = super::engine_data::pages(app);
    if views.is_empty() {
        return Err("browser_data.no_page".into());
    }
    let (sender, receiver) = mpsc::channel::<Option<(Option<PathBuf>, Option<PathBuf>)>>();
    let mut asked = 0;
    for view in views {
        let sender = sender.clone();
        let failed = sender.clone();
        let queued = view.with_webview(move |platform| {
            let folders = platform.inner().website_data_manager().map(|manager| {
                (
                    manager.base_cache_directory().map(|path| PathBuf::from(path.to_string())),
                    manager.base_data_directory().map(|path| PathBuf::from(path.to_string())),
                )
            });
            crate::error_log::ignored(sender.send(folders));
        });
        match queued {
            Ok(()) => asked += 1,
            Err(error) => {
                crate::error_log::warn(&format!("Could not reach a page to measure its data: {error}"));
                crate::error_log::ignored(failed.send(None));
                asked += 1;
            }
        }
    }
    drop(sender);
    let deadline = Instant::now() + REACH_TIMEOUT;
    let mut seen = HashSet::new();
    let mut stores = Vec::new();
    for _ in 0..asked {
        let wait = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(wait) {
            Ok(Some((cache, data))) => {
                let key = format!("{cache:?}|{data:?}");
                if seen.insert(key) {
                    stores.push(Folders {
                        cache: cache.into_iter().collect(),
                        site: data.into_iter().collect(),
                    });
                }
            }
            Ok(None) => {}
            Err(_) => break,
        }
    }
    Ok(stores)
}

#[cfg(not(any(windows, target_os = "linux")))]
fn reach_stores(_app: &AppHandle) -> Result<Vec<Folders>, String> {
    Err("browser_data.unsupported_platform".into())
}

/// Measures the engine's cache and site data. `None` when it cannot be measured on this system: on Linux
/// the engine cannot clear by age (see [`super::engine_data`]), so nothing is promised for an age there.
pub async fn measure(app: &AppHandle, older_than_days: Option<u32>) -> Result<Option<Usage>, String> {
    if cfg!(target_os = "linux") && older_than_days.is_some() {
        return Ok(None);
    }
    let stores = reach_stores(app)?;
    tauri::async_runtime::spawn_blocking(move || Some(measure_folders(stores, older_than_days)))
        .await
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("spark-usage-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn counts_every_file_and_skips_the_newer_ones_when_there_is_an_age() {
        let root = scratch("age");
        fs::create_dir_all(root.join("Cache")).unwrap();
        fs::write(root.join("Cache").join("a"), vec![0u8; 100]).unwrap();
        fs::write(root.join("Cache").join("b"), vec![0u8; 50]).unwrap();
        let stores = || vec![profile_folders(&root)];
        assert_eq!(measure_folders(stores(), None).cache_bytes, 150);
        // Everything was just written, so nothing is older than five days.
        assert_eq!(measure_folders(stores(), Some(5)).cache_bytes, 0);
        let old = SystemTime::now() - Duration::from_secs(10 * 86_400);
        fs::File::options().write(true).open(root.join("Cache").join("a")).unwrap().set_modified(old).unwrap();
        assert_eq!(measure_folders(stores(), Some(5)).cache_bytes, 100);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn cache_and_site_data_are_told_apart_and_cookies_wait_for_no_age() {
        let root = scratch("kinds");
        fs::create_dir_all(root.join("Local Storage")).unwrap();
        fs::create_dir_all(root.join("Network")).unwrap();
        fs::create_dir_all(root.join("Service Worker").join("CacheStorage")).unwrap();
        fs::write(root.join("Local Storage").join("x"), vec![0u8; 10]).unwrap();
        fs::write(root.join("Network").join("Cookies"), vec![0u8; 20]).unwrap();
        fs::write(root.join("Service Worker").join("CacheStorage").join("c"), vec![0u8; 40]).unwrap();
        let usage = measure_folders(vec![profile_folders(&root)], None);
        assert_eq!((usage.cache_bytes, usage.site_bytes, usage.partial), (40, 30, false));
        let old = SystemTime::now() - Duration::from_secs(30 * 86_400);
        for file in [root.join("Local Storage").join("x"), root.join("Network").join("Cookies")] {
            fs::File::options().write(true).open(file).unwrap().set_modified(old).unwrap();
        }
        assert_eq!(measure_folders(vec![profile_folders(&root)], Some(5)).site_bytes, 10, "the cookie file is left out when filtering by age");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_missing_folder_is_zero() {
        assert_eq!(measure_folders(vec![profile_folders(Path::new("/does/not/exist"))], None), Usage::default());
    }
}
