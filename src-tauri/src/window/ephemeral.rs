//! Ephemeral / Disposable Containers for private, throwaway tab sessions.
//!
//! Ephemeral tabs have isolated cookies, cache, local storage, and session data
//! stored inside a dedicated temporary directory. Upon closing the tab, all its
//! persistent data is wiped from disk and never written to the SQLite session database.

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
pub fn purge_all_ephemeral_data(app: &AppHandle) {
    if let Ok(data_dir) = app.path().app_local_data_dir() {
        let root = data_dir.join(EPHEMERAL_DATA_DIR);
        if root.is_dir() {
            let _ = std::fs::remove_dir_all(&root);
        }
    }
}
