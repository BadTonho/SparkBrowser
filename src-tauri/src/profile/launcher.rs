//! What Spark does at start-up when there is more than one profile: open a profile straight away or
//! show a window to choose one. The choice is remembered in `launcher.json` next to the profiles, so
//! every profile process reads and writes the same small file (written through a temporary file and a
//! rename, so a crash never leaves half of it).
use super::store::{self, ProfileInfo};
use super::DEFAULT_ID;
use crate::error_log::{self, Level};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const FILE: &str = "launcher.json";
/// The picker has no profile: it runs with folders of its own, away from every profile.
pub const LAUNCHER_DIR: &str = "launcher";

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Prefs {
    /// Show the picker at every start, even after a profile was chosen.
    pub ask_every_time: bool,
    /// The profile chosen last. `None` until the person chose one.
    pub last_profile: Option<String>,
}

/// What this start does about the profile.
#[derive(Debug, PartialEq)]
pub enum Startup {
    /// Open this profile (`None` is the Default profile).
    Profile(Option<String>),
    /// Show the window that asks which profile to open.
    Picker,
}

fn file(root: &Path) -> PathBuf {
    root.join(FILE)
}

/// The saved choice. A missing or unreadable file means the defaults, and an unreadable one is reported.
pub fn load(root: &Path) -> Prefs {
    let Ok(raw) = std::fs::read_to_string(file(root)) else {
        return Prefs::default();
    };
    match serde_json::from_str::<Prefs>(raw.trim_start_matches('\u{feff}')) {
        Ok(prefs) => prefs,
        Err(error) => {
            error_log::report(
                Level::Warning,
                "profiles",
                "launcher.json could not be read, using the defaults",
                &error.to_string(),
            );
            Prefs::default()
        }
    }
}

pub fn save(root: &Path, prefs: &Prefs) -> Result<(), String> {
    let write = || -> Result<(), String> {
        std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec_pretty(prefs).map_err(|e| e.to_string())?;
        let temporary = root.join(format!("{FILE}.tmp"));
        std::fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
        std::fs::rename(&temporary, file(root)).map_err(|e| e.to_string())
    };
    write().map_err(|details| {
        error_log::report(
            Level::Error,
            "profiles",
            "the profile choice at start-up could not be saved",
            &details,
        );
        "profile.prefs_failed".to_string()
    })
}

/// Decides what a start without `--profile` does. With a single profile there is nothing to choose.
/// The picker shows up the first time there is a choice, and every time when the person asked for it
/// (or when the profile chosen last no longer exists).
pub fn decide(profiles: &[ProfileInfo], prefs: &Prefs) -> Startup {
    if profiles.len() < 2 {
        return Startup::Profile(None);
    }
    if prefs.ask_every_time {
        return Startup::Picker;
    }
    match prefs.last_profile.as_deref() {
        Some(DEFAULT_ID) => Startup::Profile(None),
        Some(id) if profiles.iter().any(|profile| profile.id == id) => {
            Startup::Profile(Some(id.to_string()))
        }
        _ => Startup::Picker,
    }
}

/// Reads what is on disk and decides.
pub fn startup(root: &Path) -> Startup {
    decide(&store::list(root), &load(root))
}

/// Remembers the profile that was chosen, and whether to ask next time.
pub fn remember(root: &Path, id: &str, ask_every_time: bool) -> Result<(), String> {
    save(
        root,
        &Prefs {
            ask_every_time,
            last_profile: Some(id.to_string()),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(id: &str, is_default: bool) -> ProfileInfo {
        ProfileInfo {
            id: id.into(),
            name: if is_default { String::new() } else { id.into() },
            is_default,
            created_at: 0,
            avatar: None,
        }
    }

    fn two() -> Vec<ProfileInfo> {
        vec![profile(DEFAULT_ID, true), profile("0123456789ab", false)]
    }

    #[test]
    fn with_one_profile_nothing_is_asked() {
        let only = vec![profile(DEFAULT_ID, true)];
        for prefs in [
            Prefs::default(),
            Prefs {
                ask_every_time: true,
                last_profile: Some("0123456789ab".into()),
            },
        ] {
            assert_eq!(decide(&only, &prefs), Startup::Profile(None));
        }
    }

    #[test]
    fn the_first_time_there_is_a_choice_the_picker_shows() {
        assert_eq!(decide(&two(), &Prefs::default()), Startup::Picker);
    }

    #[test]
    fn after_a_choice_it_opens_that_profile_unless_asked_every_time() {
        let chosen = Prefs {
            ask_every_time: false,
            last_profile: Some("0123456789ab".into()),
        };
        assert_eq!(
            decide(&two(), &chosen),
            Startup::Profile(Some("0123456789ab".into()))
        );
        let default_chosen = Prefs {
            ask_every_time: false,
            last_profile: Some(DEFAULT_ID.into()),
        };
        assert_eq!(decide(&two(), &default_chosen), Startup::Profile(None));
        let always = Prefs {
            ask_every_time: true,
            ..chosen
        };
        assert_eq!(decide(&two(), &always), Startup::Picker);
    }

    #[test]
    fn a_profile_chosen_and_deleted_since_asks_again() {
        let gone = Prefs {
            ask_every_time: false,
            last_profile: Some("ffffffffffff".into()),
        };
        assert_eq!(decide(&two(), &gone), Startup::Picker);
    }

    #[test]
    fn the_choice_survives_a_restart_and_a_broken_file_means_the_defaults() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(load(root.path()), Prefs::default());
        remember(root.path(), "0123456789ab", true).unwrap();
        assert_eq!(
            load(root.path()),
            Prefs {
                ask_every_time: true,
                last_profile: Some("0123456789ab".into())
            }
        );
        std::fs::write(file(root.path()), "not json").unwrap();
        assert_eq!(load(root.path()), Prefs::default());
        assert!(!root.path().join("launcher.json.tmp").exists());
    }

    #[test]
    fn startup_reads_profiles_and_choice_from_disk() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(startup(root.path()), Startup::Profile(None));
        let work = store::create(root.path(), "Work", 1).unwrap();
        assert_eq!(startup(root.path()), Startup::Picker);
        remember(root.path(), &work.id, false).unwrap();
        assert_eq!(
            startup(root.path()),
            Startup::Profile(Some(work.id.clone()))
        );
        store::delete(root.path(), DEFAULT_ID, &work.id).unwrap();
        assert_eq!(startup(root.path()), Startup::Profile(None));
    }
}
