//! Listing, creating, renaming and deleting profiles. A profile is a folder with a `profile.json`;
//! nothing here is shared between the processes of different profiles.
use super::{DEFAULT_ID, PROFILES_DIR};
use crate::error_log::{self, Level};
use crate::recovery::profile_lock::ProfileLock;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MAX_PROFILES: usize = 10;
pub const MAX_NAME: usize = 40;
const FILE: &str = "profile.json";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
struct Stored {
    version: u32,
    id: String,
    name: String,
    created_at: i64,
}

/// What the interface sees of a profile. The Default profile has an empty name: the interface
/// translates "Default".
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ProfileInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub created_at: i64,
}

pub fn folder(root: &Path, id: &str) -> PathBuf {
    root.join(PROFILES_DIR).join(id)
}

/// Ids are 12 lowercase hex characters made here; anything else never reaches a path.
pub fn valid_id(id: &str) -> bool {
    id.len() == 12 && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn new_id() -> Result<String, String> {
    let mut bytes = [0u8; 6];
    getrandom::fill(&mut bytes).map_err(|error| {
        error_log::report(
            Level::Error,
            "profiles",
            "a profile id could not be generated",
            &error.to_string(),
        );
        "profile.create_failed".to_string()
    })?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// A profile name: 1 to 40 characters once trimmed, with no control characters.
pub fn clean_name(raw: &str) -> Result<String, String> {
    let name = raw.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME || name.chars().any(char::is_control) {
        return Err("profile.invalid_name".into());
    }
    Ok(name.to_string())
}

fn read(folder: &Path) -> Result<Stored, String> {
    let raw = std::fs::read_to_string(folder.join(FILE)).map_err(|e| e.to_string())?;
    let stored: Stored =
        serde_json::from_str(raw.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?;
    if stored.version != 1 {
        return Err(format!("unknown profile version {}", stored.version));
    }
    Ok(stored)
}

/// Writes through a temporary file and a rename, so a crash never leaves half a file.
fn write(folder: &Path, stored: &Stored) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(stored).map_err(|e| e.to_string())?;
    let temporary = folder.join(format!("{FILE}.tmp"));
    std::fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&temporary, folder.join(FILE)).map_err(|e| e.to_string())
}

fn default_profile() -> ProfileInfo {
    ProfileInfo {
        id: DEFAULT_ID.into(),
        name: String::new(),
        is_default: true,
        created_at: 0,
    }
}

/// The Default profile first, then the others by creation time. A folder that cannot be read is
/// skipped and reported, so one broken profile never hides the rest.
pub fn list(root: &Path) -> Vec<ProfileInfo> {
    let mut others = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root.join(PROFILES_DIR)) {
        for entry in entries.flatten() {
            let Some(id) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if !valid_id(&id) || !entry.path().is_dir() {
                continue;
            }
            match read(&entry.path()) {
                Ok(stored) if stored.id == id => others.push(ProfileInfo {
                    id,
                    name: stored.name,
                    is_default: false,
                    created_at: stored.created_at,
                }),
                Ok(_) => error_log::report(
                    Level::Warning,
                    "profiles",
                    "a profile folder was skipped: its profile.json names another id",
                    &format!("folder {id}"),
                ),
                Err(details) => error_log::report(
                    Level::Warning,
                    "profiles",
                    "a profile folder was skipped: its profile.json could not be read",
                    &format!("folder {id}: {details}"),
                ),
            }
        }
    }
    others.sort_by(|a, b| {
        a.created_at
            .cmp(&b.created_at)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    let mut all = vec![default_profile()];
    all.extend(others);
    all
}

/// One profile, or `None` when there is none with that id.
pub fn get(root: &Path, id: &str) -> Option<ProfileInfo> {
    list(root).into_iter().find(|profile| profile.id == id)
}

fn name_taken(all: &[ProfileInfo], name: &str, except: Option<&str>) -> bool {
    let wanted = name.to_lowercase();
    all.iter()
        .filter(|profile| !profile.is_default && Some(profile.id.as_str()) != except)
        .any(|profile| profile.name.to_lowercase() == wanted)
}

pub fn create(root: &Path, raw_name: &str, now: i64) -> Result<ProfileInfo, String> {
    let name = clean_name(raw_name).inspect_err(|code| {
        error_log::report(
            Level::Info,
            "profiles",
            "a profile was not created: the name is not valid",
            code,
        );
    })?;
    let all = list(root);
    if all.len() >= MAX_PROFILES {
        error_log::report(
            Level::Info,
            "profiles",
            "a profile was not created: the limit was reached",
            "",
        );
        return Err("profile.limit".into());
    }
    if name_taken(&all, &name, None) {
        error_log::report(
            Level::Info,
            "profiles",
            "a profile was not created: the name is already used",
            "",
        );
        return Err("profile.duplicate".into());
    }
    let id = new_id()?;
    let directory = folder(root, &id);
    let stored = Stored {
        version: 1,
        id: id.clone(),
        name: name.clone(),
        created_at: now,
    };
    let created = std::fs::create_dir_all(&directory)
        .map_err(|e| e.to_string())
        .and_then(|()| write(&directory, &stored));
    if let Err(details) = created {
        error_log::report(
            Level::Error,
            "profiles",
            "the profile folder could not be created",
            &details,
        );
        error_log::ignored(std::fs::remove_dir_all(&directory));
        return Err("profile.create_failed".into());
    }
    Ok(ProfileInfo {
        id,
        name,
        is_default: false,
        created_at: now,
    })
}

pub fn rename(root: &Path, id: &str, raw_name: &str) -> Result<ProfileInfo, String> {
    if id == DEFAULT_ID {
        return Err("profile.cannot_rename_default".into());
    }
    if !valid_id(id) {
        return Err("profile.not_found".into());
    }
    let name = clean_name(raw_name)?;
    let all = list(root);
    let Some(current) = all.iter().find(|profile| profile.id == id) else {
        return Err("profile.not_found".into());
    };
    if name_taken(&all, &name, Some(id)) {
        return Err("profile.duplicate".into());
    }
    let stored = Stored {
        version: 1,
        id: id.to_string(),
        name: name.clone(),
        created_at: current.created_at,
    };
    write(&folder(root, id), &stored).map_err(|details| {
        error_log::report(
            Level::Error,
            "profiles",
            "the profile could not be renamed",
            &format!("{id}: {details}"),
        );
        "profile.rename_failed".to_string()
    })?;
    Ok(ProfileInfo {
        id: id.to_string(),
        name,
        is_default: false,
        created_at: current.created_at,
    })
}

/// Deletes a profile and everything in it. The Default profile and the one running this process
/// cannot be deleted, and neither can a profile that is open in another process.
pub fn delete(root: &Path, current_id: &str, id: &str) -> Result<(), String> {
    if id == DEFAULT_ID {
        return Err("profile.cannot_delete_default".into());
    }
    if !valid_id(id) || get(root, id).is_none() {
        return Err("profile.not_found".into());
    }
    if id == current_id {
        return Err("profile.cannot_delete_current".into());
    }
    let directory = folder(root, id);
    match ProfileLock::acquire(&directory.join("data")) {
        Ok(lock) => drop(lock),
        Err(std::fs::TryLockError::WouldBlock) => {
            error_log::report(
                Level::Warning,
                "profiles",
                "a profile was not deleted: it is open in another window",
                id,
            );
            return Err("profile.in_use".into());
        }
        Err(error) => {
            error_log::report(
                Level::Error,
                "profiles",
                "a profile was not deleted: its folder could not be locked",
                &format!("{id}: {error}"),
            );
            return Err("profile.delete_failed".into());
        }
    }
    std::fs::remove_dir_all(&directory).map_err(|error| {
        error_log::report(
            Level::Error,
            "profiles",
            "the profile folder could not be removed",
            &format!("{id}: {error}"),
        );
        "profile.delete_failed".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_twelve_lowercase_hex_characters() {
        assert!(valid_id("0123456789ab"));
        for bad in [
            "",
            "default",
            "0123456789AB",
            "0123456789a",
            "../0123456789",
            "0123456789ag",
        ] {
            assert!(!valid_id(bad), "{bad}");
        }
        assert!(valid_id(&new_id().unwrap()));
    }

    #[test]
    fn names_are_trimmed_and_bounded() {
        assert_eq!(clean_name("  Work  ").unwrap(), "Work");
        for bad in ["", "   ", "a\nb", "a\u{0}b"] {
            assert_eq!(
                clean_name(bad).unwrap_err(),
                "profile.invalid_name",
                "{bad:?}"
            );
        }
        assert!(clean_name(&"a".repeat(MAX_NAME)).is_ok());
        assert!(clean_name(&"a".repeat(MAX_NAME + 1)).is_err());
    }

    #[test]
    fn a_new_profile_is_listed_after_the_default_and_survives_a_rename() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(list(root.path()).len(), 1);
        assert!(list(root.path())[0].is_default);
        let work = create(root.path(), "Work", 10).unwrap();
        let home = create(root.path(), "Home", 5).unwrap();
        let names: Vec<_> = list(root.path()).into_iter().map(|p| p.name).collect();
        assert_eq!(names, ["", "Home", "Work"]);
        let renamed = rename(root.path(), &work.id, "Office").unwrap();
        assert_eq!(renamed.created_at, 10);
        assert_eq!(get(root.path(), &work.id).unwrap().name, "Office");
        assert_eq!(get(root.path(), &home.id).unwrap().name, "Home");
        assert!(folder(root.path(), &work.id).join("profile.json").is_file());
    }

    #[test]
    fn names_cannot_repeat_and_the_number_of_profiles_is_limited() {
        let root = tempfile::tempdir().unwrap();
        let work = create(root.path(), "Work", 1).unwrap();
        assert_eq!(
            create(root.path(), "work", 2).unwrap_err(),
            "profile.duplicate"
        );
        let other = create(root.path(), "Other", 3).unwrap();
        assert_eq!(
            rename(root.path(), &other.id, "WORK").unwrap_err(),
            "profile.duplicate"
        );
        assert!(
            rename(root.path(), &work.id, "Work").is_ok(),
            "keeping its own name is fine"
        );
        for index in 0..(MAX_PROFILES - 3) {
            create(root.path(), &format!("P{index}"), 10).unwrap();
        }
        assert_eq!(list(root.path()).len(), MAX_PROFILES);
        assert_eq!(
            create(root.path(), "One more", 11).unwrap_err(),
            "profile.limit"
        );
    }

    #[test]
    fn the_default_profile_cannot_be_renamed_or_deleted() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            rename(root.path(), DEFAULT_ID, "x").unwrap_err(),
            "profile.cannot_rename_default"
        );
        let work = create(root.path(), "Work", 1).unwrap();
        assert_eq!(
            delete(root.path(), &work.id, DEFAULT_ID).unwrap_err(),
            "profile.cannot_delete_default"
        );
    }

    #[test]
    fn a_profile_is_deleted_only_when_it_is_not_the_current_one_and_not_open_elsewhere() {
        let root = tempfile::tempdir().unwrap();
        let work = create(root.path(), "Work", 1).unwrap();
        let home = create(root.path(), "Home", 2).unwrap();
        assert_eq!(
            delete(root.path(), &work.id, &work.id).unwrap_err(),
            "profile.cannot_delete_current"
        );
        assert_eq!(
            delete(root.path(), &work.id, "ffffffffffff").unwrap_err(),
            "profile.not_found"
        );
        assert_eq!(
            delete(root.path(), &work.id, "../x").unwrap_err(),
            "profile.not_found"
        );
        // Open in another process: its lock is held.
        let running = ProfileLock::acquire(&folder(root.path(), &home.id).join("data")).unwrap();
        assert_eq!(
            delete(root.path(), &work.id, &home.id).unwrap_err(),
            "profile.in_use"
        );
        assert!(folder(root.path(), &home.id).is_dir());
        drop(running);
        delete(root.path(), &work.id, &home.id).unwrap();
        assert!(!folder(root.path(), &home.id).exists());
        assert!(get(root.path(), &home.id).is_none());
        assert!(get(root.path(), &work.id).is_some());
    }

    #[test]
    fn a_broken_profile_folder_is_skipped_without_hiding_the_others() {
        let root = tempfile::tempdir().unwrap();
        let work = create(root.path(), "Work", 1).unwrap();
        let broken = folder(root.path(), "aaaaaaaaaaaa");
        std::fs::create_dir_all(&broken).unwrap();
        std::fs::write(broken.join(FILE), "not json").unwrap();
        let wrong = folder(root.path(), "bbbbbbbbbbbb");
        std::fs::create_dir_all(&wrong).unwrap();
        std::fs::write(
            wrong.join(FILE),
            r#"{"version":1,"id":"cccccccccccc","name":"Liar","created_at":1}"#,
        )
        .unwrap();
        std::fs::create_dir_all(folder(root.path(), "not-an-id")).unwrap();
        let ids: Vec<_> = list(root.path()).into_iter().map(|p| p.id).collect();
        assert_eq!(ids, [DEFAULT_ID.to_string(), work.id]);
    }
}
