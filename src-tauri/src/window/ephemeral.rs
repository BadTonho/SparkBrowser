//! Ephemeral / Disposable Containers for private, throwaway tab sessions.
//!
//! Ephemeral tabs have isolated cookies, cache, local storage, and session data
//! stored inside a dedicated temporary directory. Closed data is removed on a
//! best-effort basis and any leftovers are removed before the next session starts.

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub const EPHEMERAL_DATA_DIR: &str = "webview-ephemeral";

/// Returns the dedicated, disposable data directory for an ephemeral tab.
pub fn ephemeral_directory(app: &AppHandle, tab_id: &str) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve local data root: {e}"))?
        .join(EPHEMERAL_DATA_DIR)
        .join(tab_id);
    Ok(root)
}

/// Purges the ephemeral tab data folder from disk when closed.
pub fn purge_ephemeral_tab_data(app: &AppHandle, tab_id: &str) {
    if let Ok(dir) = ephemeral_directory(app, tab_id) {
        if dir.is_dir() {
            let dir_clone = dir.clone();
            std::thread::spawn(move || {
                // Wait briefly for WebView2/WebKit process lock release
                std::thread::sleep(std::time::Duration::from_millis(300));
                for attempt in 0..5 {
                    if !dir_clone.exists() {
                        break;
                    }
                    if std::fs::remove_dir_all(&dir_clone).is_ok() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(300 * (attempt + 1)));
                }
            });
        }
    }
}

/// Purges all leftover ephemeral data on browser startup.
///
/// Called synchronously only after acquiring the exclusive profile lock.
pub fn purge_all_ephemeral_data(app: &AppHandle) -> Result<(), String> {
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join(EPHEMERAL_DATA_DIR);
    purge_root(&root)
}

fn purge_root(root: &std::path::Path) -> Result<(), String> {
    match std::fs::remove_dir_all(root) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Could not remove temporary browsing data: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_cleanup_removes_only_the_temporary_root() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join(EPHEMERAL_DATA_DIR);
        std::fs::create_dir_all(root.join("tab-1")).unwrap();
        std::fs::write(root.join("tab-1/cookie"), b"fixture").unwrap();
        std::fs::write(directory.path().join("keep"), b"fixture").unwrap();
        purge_root(&root).unwrap();
        purge_root(&root).unwrap();
        assert!(!root.exists());
        assert!(directory.path().join("keep").exists());
    }
}
