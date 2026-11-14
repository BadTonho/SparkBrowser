//! Hardware (GPU) acceleration preference.
//!
//! WebView2 and WebKitGTK read their graphics flags once, when the browser process starts, so the
//! choice cannot live in the database or the interface: it is a small file in the app data folder,
//! read before any webview exists, and a change applies after a restart. Acceleration is on unless
//! the file says otherwise.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const FILE: &str = "graphics.json";
/// Set (to `1`) in the process environment when this module turned acceleration off, so a restarted
/// child, which inherits the environment, knows which flags are ours to take away again.
const MARKER: &str = "SPARK_GRAPHICS_FLAGS";
#[cfg(windows)]
const ARGUMENTS_VARIABLE: &str = "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS";
const GPU_FLAG: &str = "--disable-gpu";
/// WebKitGTK: software compositing, and no DMA-BUF renderer (the usual cause of blank windows).
#[cfg(not(windows))]
const WEBKIT_VARIABLES: [&str; 2] = [
    "WEBKIT_DISABLE_COMPOSITING_MODE",
    "WEBKIT_DISABLE_DMABUF_RENDERER",
];

#[derive(Serialize, Deserialize)]
struct Stored {
    hardware_acceleration: bool,
}

/// Where the preference lives, matching Tauri's app data folder. Read early, before the app handle
/// (and so `app_data_dir()`) exists, so it follows the config's directory override the same way
/// Tauri will (native tests redirect it) and otherwise the per-identifier folder.
pub fn data_directory(config: &tauri::utils::config::Config) -> Option<PathBuf> {
    use tauri::utils::config::AppDirectoriesOverride;
    if let Some(AppDirectoriesOverride::Directories(directories)) =
        &config.app.app_directories_override
    {
        if let Some(data) = &directories.data {
            return Some(data.clone());
        }
    }
    crate::legacy::identifier_roots()
        .into_iter()
        .next()
        .map(|root| root.join(&config.identifier))
}

/// `true` (accelerated) unless the file exists, parses and says `false`. A leading byte order mark,
/// which Notepad and Windows PowerShell add when saving as UTF-8, is ignored.
pub fn load(directory: &Path) -> bool {
    fs::read_to_string(directory.join(FILE))
        .ok()
        .and_then(|raw| serde_json::from_str::<Stored>(raw.trim_start_matches('\u{feff}')).ok())
        .is_none_or(|stored| stored.hardware_acceleration)
}

pub fn save(directory: &Path, enabled: bool) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|_| "graphics.write_failed")?;
    let raw = serde_json::to_vec_pretty(&Stored {
        hardware_acceleration: enabled,
    })
    .map_err(|e| e.to_string())?;
    crate::backup::files::write_bytes(&directory.join(FILE), &raw)
        .map_err(|_| "graphics.write_failed".into())
}

/// Returns the argument string with our flag removed (when we put it there) or added (when
/// disabling), plus whether this call added it.
fn merge_arguments(existing: &str, ours_inherited: bool, disable: bool) -> (String, bool) {
    let mut tokens: Vec<&str> = existing.split_whitespace().collect();
    if ours_inherited {
        tokens.retain(|token| *token != GPU_FLAG);
    }
    let added = disable && !tokens.contains(&GPU_FLAG);
    if added {
        tokens.push(GPU_FLAG);
    }
    (tokens.join(" "), added)
}

/// Puts the graphics flags into the process environment. Must run before the first webview is
/// created, and only in the main process: private windows inherit what their parent set.
pub fn apply_environment(enabled: bool) {
    let ours_inherited = std::env::var_os(MARKER).is_some();
    #[cfg(windows)]
    let added = {
        let current = std::env::var(ARGUMENTS_VARIABLE).unwrap_or_default();
        let (next, added) = merge_arguments(&current, ours_inherited, !enabled);
        if next.is_empty() {
            std::env::remove_var(ARGUMENTS_VARIABLE);
        } else {
            std::env::set_var(ARGUMENTS_VARIABLE, next);
        }
        added
    };
    #[cfg(not(windows))]
    let added = {
        let mut added = false;
        for name in WEBKIT_VARIABLES {
            if !enabled && std::env::var_os(name).is_none() {
                std::env::set_var(name, "1");
                added = true;
            } else if enabled && ours_inherited {
                std::env::remove_var(name);
            }
        }
        added
    };
    if added {
        std::env::set_var(MARKER, "1");
    } else {
        std::env::remove_var(MARKER);
    }
}

/// What this process is running with, read from the environment so private windows and restarted
/// children agree with the parent that configured it.
pub fn running_accelerated() -> bool {
    std::env::var_os(MARKER).is_none()
}

pub struct GraphicsService {
    started_accelerated: bool,
}
impl GraphicsService {
    pub fn new(started_accelerated: bool) -> Self {
        Self {
            started_accelerated,
        }
    }
    pub fn status(&self, wanted: bool) -> GraphicsStatus {
        GraphicsStatus {
            enabled: wanted,
            active: self.started_accelerated,
            restart_required: wanted != self.started_accelerated,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GraphicsStatus {
    /// What the setting says; takes effect on the next start.
    pub enabled: bool,
    /// What the running process started with.
    pub active: bool,
    pub restart_required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acceleration_is_on_without_a_file_or_with_a_damaged_one() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load(dir.path()));
        fs::write(dir.path().join(FILE), "not json").unwrap();
        assert!(load(dir.path()));
        fs::write(dir.path().join(FILE), r#"{"other":1}"#).unwrap();
        assert!(load(dir.path()));
    }

    #[test]
    fn a_file_saved_with_a_byte_order_mark_is_still_read() {
        // Regression: a hand-edited file with a BOM was treated as damaged, so "off" was ignored.
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(FILE),
            "\u{feff}{\"hardware_acceleration\": false}",
        )
        .unwrap();
        assert!(!load(dir.path()));
    }

    #[test]
    fn the_choice_roundtrips_and_creates_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("profile");
        save(&nested, false).unwrap();
        assert!(!load(&nested));
        save(&nested, true).unwrap();
        assert!(load(&nested));
    }

    #[test]
    fn our_flag_is_added_once_and_only_removed_when_we_added_it() {
        let flag = GPU_FLAG;
        assert_eq!(merge_arguments("", false, true), (flag.to_string(), true));
        // Disabling twice never duplicates the flag.
        assert_eq!(merge_arguments(flag, true, true), (flag.to_string(), true));
        // Re-enabling removes it only when it was ours.
        assert_eq!(merge_arguments(flag, true, false), (String::new(), false));
        assert_eq!(
            merge_arguments(flag, false, false),
            (flag.to_string(), false)
        );
        // The user's own arguments survive both ways.
        let own = "--remote-debugging-port=9222";
        assert_eq!(
            merge_arguments(own, false, true),
            (format!("{own} {flag}"), true)
        );
        assert_eq!(
            merge_arguments(&format!("{own} {flag}"), true, false),
            (own.to_string(), false)
        );
        // A flag the user set themselves is left alone and not claimed.
        assert_eq!(
            merge_arguments(flag, false, true),
            (flag.to_string(), false)
        );
    }

    #[test]
    fn the_environment_gets_the_flags_when_off_and_gives_them_back_when_on() {
        // The only test that touches these process-wide variables.
        let own = "--remote-debugging-port=9222";
        #[cfg(windows)]
        std::env::set_var(ARGUMENTS_VARIABLE, own);
        std::env::remove_var(MARKER);
        apply_environment(false);
        assert!(!running_accelerated());
        #[cfg(windows)]
        assert_eq!(
            std::env::var(ARGUMENTS_VARIABLE).unwrap(),
            format!("{own} {GPU_FLAG}")
        );
        #[cfg(not(windows))]
        assert!(WEBKIT_VARIABLES
            .iter()
            .all(|name| std::env::var_os(name).is_some()));
        // A restarted child inherits that environment and turns acceleration back on.
        apply_environment(true);
        assert!(running_accelerated());
        #[cfg(windows)]
        {
            assert_eq!(std::env::var(ARGUMENTS_VARIABLE).unwrap(), own);
            std::env::remove_var(ARGUMENTS_VARIABLE);
        }
        #[cfg(not(windows))]
        assert!(WEBKIT_VARIABLES
            .iter()
            .all(|name| std::env::var_os(name).is_none()));
    }

    #[test]
    fn status_reports_when_the_wanted_value_differs_from_the_running_one() {
        let running_on = GraphicsService::new(true);
        assert!(!running_on.status(true).restart_required);
        let pending = running_on.status(false);
        assert!(pending.restart_required && !pending.enabled && pending.active);
        assert!(GraphicsService::new(false).status(true).restart_required);
    }
}
