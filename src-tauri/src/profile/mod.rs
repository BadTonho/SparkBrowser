//! Separate profiles. Each profile has its own folders (database, password vault, WebView data), so
//! history, favorites, cookies, logins and settings never mix between profiles.
//!
//! The Default profile is the folder Spark has always used and is never moved. Every other profile
//! lives in `<local data of the Default profile>/profiles/<id>/` and runs as its own process, started
//! with `--profile <id>`: that process redirects every app directory into the profile folder (the
//! same mechanism private windows use) before any WebView exists. Profiles are described by a small
//! `profile.json` in each folder, so listing them is a folder scan and no file is shared between
//! processes that could be written at the same time.
mod avatar;
pub mod launch;
pub mod launcher;
pub mod store;

use std::path::{Path, PathBuf};
use tauri::utils::config::{AppDirectoriesOverride, AppDirectoryOverrides};

/// The id of the profile Spark has always had.
pub const DEFAULT_ID: &str = "default";
/// Folder (inside the local data folder of the Default profile) that holds the other profiles.
pub const PROFILES_DIR: &str = "profiles";
pub const FLAG: &str = "--profile";

/// Managed state: which profile this process is, and where the others are.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileRole {
    id: String,
    /// The local data folder of the Default profile, where `profiles/` lives.
    local_root: PathBuf,
    /// The data folder of the Default profile (its lock and instance link live there).
    default_data: PathBuf,
    /// This process only asks which profile to open (see `launcher`): it has no profile of its own.
    picker: bool,
}

impl ProfileRole {
    pub fn new(id: Option<&str>, local_root: PathBuf, default_data: PathBuf) -> Self {
        Self {
            id: id.unwrap_or(DEFAULT_ID).to_string(),
            local_root,
            default_data,
            picker: false,
        }
    }

    pub fn with_picker(mut self, picker: bool) -> Self {
        self.picker = picker;
        self
    }

    pub fn is_picker(&self) -> bool {
        self.picker
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_default(&self) -> bool {
        self.id == DEFAULT_ID
    }

    pub fn local_root(&self) -> &Path {
        &self.local_root
    }

    /// The data folder of a profile: where its database, lock and instance link are.
    pub fn data_dir_of(&self, id: &str) -> PathBuf {
        if id == DEFAULT_ID {
            self.default_data.clone()
        } else {
            store::folder(&self.local_root, id).join("data")
        }
    }
}

/// Whether the running process is the Default profile (also true where no profile state exists).
pub fn is_default_profile(app: &tauri::AppHandle) -> bool {
    use tauri::Manager;
    app.try_state::<ProfileRole>()
        .is_none_or(|role| role.is_default())
}

/// The profile asked for with `--profile <id>`. `None` means the Default profile.
pub fn requested_id() -> Result<Option<String>, String> {
    parse_id(&std::env::args().collect::<Vec<_>>())
}

/// Whether this launch named a profile at all (`--profile default` counts). A launch that did not
/// leaves the choice to `launcher`.
pub fn profile_requested() -> bool {
    std::env::args().any(|arg| arg == FLAG)
}

fn parse_id(args: &[String]) -> Result<Option<String>, String> {
    let Some(position) = args.iter().position(|arg| arg == FLAG) else {
        return Ok(None);
    };
    match args.get(position + 1).map(String::as_str) {
        Some(DEFAULT_ID) => Ok(None),
        Some(id) if store::valid_id(id) => Ok(Some(id.to_string())),
        _ => Err("profile.invalid_id".into()),
    }
}

/// The local data folder of the Default profile, read before anything is redirected. It follows the
/// config's directory override the same way Tauri will (native tests redirect it).
pub fn local_root(config: &tauri::utils::config::Config) -> Option<PathBuf> {
    if let Some(AppDirectoriesOverride::Directories(directories)) =
        &config.app.app_directories_override
    {
        if let Some(local) = &directories.local_data {
            return Some(local.clone());
        }
    }
    dirs::data_local_dir().map(|root| root.join(&config.identifier))
}

/// Points every app directory of a non-default profile into its folder.
pub fn configure(context: &mut tauri::Context<tauri::Wry>, id: Option<&str>, local_root: &Path) {
    let Some(id) = id else {
        return;
    };
    let folder = store::folder(local_root, id);
    context.config_mut().app.app_directories_override =
        Some(AppDirectoriesOverride::Directories(AppDirectoryOverrides {
            config: Some(folder.join("config")),
            data: Some(folder.join("data")),
            local_data: Some(folder.join("profile")),
            cache: Some(folder.join("cache")),
            log: Some(folder.join("logs")),
        }));
}

/// Points every app directory of the profile picker into its own folder, away from every profile.
pub fn configure_launcher(context: &mut tauri::Context<tauri::Wry>, local_root: &Path) {
    let folder = local_root.join(launcher::LAUNCHER_DIR);
    context.config_mut().app.app_directories_override =
        Some(AppDirectoriesOverride::Directories(AppDirectoryOverrides {
            config: Some(folder.join("config")),
            data: Some(folder.join("data")),
            local_data: Some(folder.join("profile")),
            cache: Some(folder.join("cache")),
            log: Some(folder.join("logs")),
        }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn the_profile_argument_is_optional_and_strictly_validated() {
        assert_eq!(parse_id(&args(&["spark"])), Ok(None));
        assert_eq!(
            parse_id(&args(&["spark", "--profile", "default"])),
            Ok(None)
        );
        assert_eq!(
            parse_id(&args(&["spark", "--profile", "0123456789ab"])),
            Ok(Some("0123456789ab".into()))
        );
        for bad in ["", "..", "../x", "0123456789AB", "short", "0123456789abc"] {
            assert_eq!(
                parse_id(&args(&["spark", "--profile", bad])),
                Err("profile.invalid_id".into()),
                "{bad}"
            );
        }
        assert!(parse_id(&args(&["spark", "--profile"])).is_err());
    }

    #[test]
    fn the_default_profile_keeps_its_own_folder_and_the_others_live_under_profiles() {
        let role = ProfileRole::new(None, PathBuf::from("/local"), PathBuf::from("/data"));
        assert!(role.is_default());
        assert_eq!(role.data_dir_of(DEFAULT_ID), PathBuf::from("/data"));
        assert_eq!(
            role.data_dir_of("0123456789ab"),
            PathBuf::from("/local/profiles/0123456789ab/data")
        );
        let other = ProfileRole::new(
            Some("0123456789ab"),
            PathBuf::from("/l"),
            PathBuf::from("/d"),
        );
        assert!(!other.is_default());
        assert_eq!(other.id(), "0123456789ab");
    }
}
