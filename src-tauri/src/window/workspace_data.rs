//! Opt-in per-workspace data directories for content WebViews.
//!
//! When workspace isolation is enabled, every content WebView receives a
//! dedicated WebView2 user data folder derived from its workspace id. Cookies,
//! storage and cache become workspace-scoped, and the WebView2 runtime keeps a
//! separate browser process for each folder. Workspace ids are never trusted as
//! path components: only an ASCII-safe subset is used verbatim, every other id
//! hashes onto a fixed-length digest so distinct ids never share a folder.

use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub const WORKSPACE_DATA_DIR: &str = "webview-workspaces";
const MAX_VERBATIM_ID_BYTES: usize = 48;

/// A workspace id may only act as a folder name when it already is one.
pub fn is_verbatim_id(workspace_id: &str) -> bool {
    let bytes = workspace_id.as_bytes();
    !workspace_id.is_empty()
        && bytes.len() <= MAX_VERBATIM_ID_BYTES
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_')
        && !workspace_id.ends_with('-')
}

/// Deterministic folder name for a workspace id. Case differences collapse on
/// case-insensitive file systems, and host-dependent ids always hash so they
/// can never be combined into traversal sequences.
pub fn folder_name(workspace_id: &str) -> String {
    if is_verbatim_id(workspace_id) {
        workspace_id.to_ascii_lowercase()
    } else {
        format!("h-{:x}", Sha256::digest(workspace_id.as_bytes()))
    }
}

fn enabled(app: &AppHandle) -> bool {
    app.state::<crate::state::browser_state::BrowserState>()
        .workspace_isolation
        .load(std::sync::atomic::Ordering::Acquire)
}

/// Returns the per-workspace data directory while isolation stays enabled.
/// The Tauri webview manager creates missing directories before use.
pub fn directory(app: &AppHandle, workspace_id: &str) -> Result<Option<PathBuf>, String> {
    if !enabled(app) {
        return Ok(None);
    }
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve the workspace data root: {e}"))?
        .join(WORKSPACE_DATA_DIR)
        .join(folder_name(workspace_id));
    Ok(Some(root))
}

/// Data directory for a tab's content WebView. Unknown tabs fall back to the
/// active workspace so creation can still proceed inside a scoped folder.
pub fn content_directory(app: &AppHandle, tab_id: &str) -> Result<Option<PathBuf>, String> {
    if !enabled(app) {
        return Ok(None);
    }
    let state = app.state::<crate::state::browser_state::BrowserState>();
    let workspace_id = state
        .tabs
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|tab| tab.id == tab_id)
        .map(|tab| tab.workspace_id.clone())
        .or_else(|| {
            state
                .active_workspace_id
                .lock()
                .ok()
                .and_then(|id| Some(id.clone()))
        })
        .ok_or_else(|| "Workspace unavailable".to_string())?;
    directory(app, &workspace_id)
}

/// Removes a deleted workspace's data folder when nothing uses it anymore.
/// Live WebViews keep their files; the attempt is best effort by design.
pub fn remove(app: &AppHandle, workspace_id: &str) {
    if !enabled(app) {
        return;
    }
    if let Ok(root) = app.path().app_local_data_dir() {
        let folder = root
            .join(WORKSPACE_DATA_DIR)
            .join(folder_name(workspace_id));
        if folder.is_dir() {
            if let Err(error) = std::fs::remove_dir_all(&folder) {
                eprintln!("Failed to remove workspace data folder: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_ids_stay_verbatim_and_case_insensitive() {
        assert_eq!(folder_name("ws-personal"), "ws-personal");
        assert_eq!(folder_name("ws-custom-123"), "ws-custom-123");
        // Case-insensitive file systems unify both spellings into one folder.
        assert_eq!(
            folder_name("WS-Custom-123"),
            folder_name("ws-custom-123")
        );
    }

    #[test]
    fn traversal_and_separators_never_reach_a_verbatim_folder() {
        for id in ["", "..", "../cookies", "a/b", "a\\b", "a b", "é", "ws-a/../x"] {
            assert!(!is_verbatim_id(id), "{id} must not be verbatim");
            let name = folder_name(id);
            assert!(
                name.starts_with("h-")
                    && name.len() == 66
                    && name[2..].bytes().all(|byte| byte.is_ascii_hexdigit()),
                "{name} must hash: {name}"
            );
        }
    }

    #[test]
    fn different_ids_never_share_a_folder() {
        let names: Vec<String> = ["a/b", "a b", "a:b"]
            .iter()
            .map(|id| folder_name(id))
            .collect();
        assert_eq!(names.len(), names.iter().collect::<std::collections::HashSet<_>>().len());
    }

    #[test]
    fn oversized_ids_hash_instead_of_truncating() {
        let long = "ws-".to_string() + &"a".repeat(64);
        assert!(!is_verbatim_id(&long));
        let name = folder_name(&long);
        assert_eq!(name.len(), 66);
        // The digest is deterministic and never approximates another id.
        assert_eq!(folder_name(&long.clone()), name);
        assert_ne!(folder_name(&(long.clone() + "suffix")), name);
    }
}
