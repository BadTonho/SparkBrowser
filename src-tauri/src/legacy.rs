//! One-time moves of data created before the app was renamed from Veloce to Spark, so an existing
//! profile (bookmarks, settings, WebView storage) keeps working after the identifier changed.
//! Everything here is best effort: a failed move leaves the old data untouched.
use std::{
    fs,
    path::{Path, PathBuf},
};

const OLD_IDENTIFIER: &str = "org.veloce.browser";
pub const NEW_IDENTIFIER: &str = "org.spark.browser";
const OLD_DATABASE: &str = "veloce_browser.db";
const NEW_DATABASE: &str = "spark_browser.db";

fn env_dir(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// The per-identifier folders the OS keeps for this app: data, local data (where WebView2 stores
/// its profile) on Windows; data, config and cache on Linux.
pub(crate) fn identifier_roots() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        ["APPDATA", "LOCALAPPDATA"]
            .iter()
            .filter_map(|name| env_dir(name))
            .collect()
    }
    #[cfg(not(windows))]
    {
        let home = env_dir("HOME");
        [
            ("XDG_DATA_HOME", ".local/share"),
            ("XDG_CONFIG_HOME", ".config"),
            ("XDG_CACHE_HOME", ".cache"),
        ]
        .iter()
        .filter_map(|(name, fallback)| {
            env_dir(name).or_else(|| home.as_ref().map(|home| home.join(fallback)))
        })
        .collect()
    }
}

fn move_directory(old: &Path, new: &Path) {
    if old.is_dir() && !new.exists() {
        if let Err(error) = fs::rename(old, new) {
            eprintln!(
                "Could not move {} to {}: {error}",
                old.display(),
                new.display()
            );
        }
    }
}

/// `veloce_browser.db` (and its WAL/shared-memory files) become `spark_browser.db`.
pub fn rename_database_files(directory: &Path) {
    let (old, new) = (directory.join(OLD_DATABASE), directory.join(NEW_DATABASE));
    if new.exists() || !old.is_file() {
        return;
    }
    if fs::rename(&old, &new).is_err() {
        return;
    }
    for suffix in ["-wal", "-shm"] {
        let from = directory.join(format!("{OLD_DATABASE}{suffix}"));
        if from.is_file() {
            let _ = fs::rename(from, directory.join(format!("{NEW_DATABASE}{suffix}")));
        }
    }
}

/// Moves the old per-identifier folders to the new names. Must run before the webview is created,
/// because WebView2 and WebKitGTK open their profile from these folders. Only the real app
/// identifier migrates; native test builds use their own folders.
pub fn migrate_profile(identifier: &str) {
    if identifier != NEW_IDENTIFIER {
        return;
    }
    for root in identifier_roots() {
        let new = root.join(NEW_IDENTIFIER);
        move_directory(&root.join(OLD_IDENTIFIER), &new);
        rename_database_files(&new);
    }
}

/// The database path as it was before the move, used to find secrets stored under the old names.
pub fn legacy_database_path(path: &str) -> String {
    path.replace(NEW_IDENTIFIER, OLD_IDENTIFIER)
        .replace(NEW_DATABASE, OLD_DATABASE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_a_profile_once_and_never_overwrites_a_new_one() {
        let root = tempfile::tempdir().unwrap();
        let (old, new) = (
            root.path().join(OLD_IDENTIFIER),
            root.path().join(NEW_IDENTIFIER),
        );
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("marker"), "profile").unwrap();
        move_directory(&old, &new);
        assert!(!old.exists());
        assert_eq!(fs::read_to_string(new.join("marker")).unwrap(), "profile");
        // A second run, or a profile that already exists, is left alone.
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("marker"), "other").unwrap();
        move_directory(&old, &new);
        assert_eq!(fs::read_to_string(new.join("marker")).unwrap(), "profile");
        assert!(old.join("marker").is_file());
    }

    #[test]
    fn renames_the_database_with_its_wal_files_and_keeps_an_existing_new_one() {
        let dir = tempfile::tempdir().unwrap();
        for suffix in ["", "-wal", "-shm"] {
            fs::write(dir.path().join(format!("{OLD_DATABASE}{suffix}")), suffix).unwrap();
        }
        rename_database_files(dir.path());
        for suffix in ["", "-wal", "-shm"] {
            let moved = dir.path().join(format!("{NEW_DATABASE}{suffix}"));
            assert_eq!(fs::read_to_string(moved).unwrap(), suffix);
            assert!(!dir.path().join(format!("{OLD_DATABASE}{suffix}")).exists());
        }
        fs::write(dir.path().join(OLD_DATABASE), "older").unwrap();
        rename_database_files(dir.path());
        assert!(
            dir.path().join(OLD_DATABASE).is_file(),
            "an existing new database wins"
        );
    }

    #[test]
    fn only_the_real_identifier_migrates_and_legacy_paths_are_derived() {
        migrate_profile("org.spark.browser.native-tests");
        assert_eq!(
            legacy_database_path(r"C:\Users\me\AppData\Roaming\org.spark.browser\spark_browser.db"),
            r"C:\Users\me\AppData\Roaming\org.veloce.browser\veloce_browser.db"
        );
    }
}
