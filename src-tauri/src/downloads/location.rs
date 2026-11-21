//! Where downloads are saved when Spark does not ask each time (Settings → Downloads → Location).
//! Empty means the system Downloads folder. A folder that is gone or can no longer be written to
//! never breaks a download: Spark falls back to the system folder and says so in the error log.
use crate::database::preferences_repo;
use crate::error_log::{self, Level};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const KEY: &str = "download_directory";

#[derive(Default)]
pub struct DownloadLocation(Mutex<Option<PathBuf>>);

/// What Settings shows.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Location {
    pub path: String,
    /// True while the system Downloads folder is used.
    pub is_default: bool,
}

impl DownloadLocation {
    pub fn custom(&self) -> Option<PathBuf> {
        self.0.lock().ok()?.clone()
    }

    pub fn set(&self, path: Option<PathBuf>) {
        if let Ok(mut slot) = self.0.lock() {
            *slot = path;
        }
    }

    pub fn load(&self, conn: &rusqlite::Connection) -> Result<(), String> {
        let saved = preferences_repo::load(conn, KEY)?
            .map(PathBuf::from)
            .filter(|path| !path.as_os_str().is_empty());
        self.set(saved);
        Ok(())
    }
}

pub fn store(conn: &rusqlite::Connection, path: Option<&Path>) -> Result<(), String> {
    match path {
        Some(path) => preferences_repo::store(conn, KEY, &path.to_string_lossy()),
        None => preferences_repo::remove(conn, KEY),
    }
}

/// A folder Spark may use: it exists, is a directory and a file can be created in it.
pub fn usable(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("downloads.invalid_location".into());
    }
    let canonical = std::fs::canonicalize(path).map_err(|_| "downloads.invalid_location")?;
    if !canonical.is_dir() {
        return Err("downloads.invalid_location".into());
    }
    let probe = tempfile::Builder::new()
        .prefix(".spark-write-check")
        .tempfile_in(&canonical)
        .map_err(|_| "downloads.location_not_writable")?;
    drop(probe);
    Ok(canonical)
}

/// The chosen folder when it is still usable, otherwise `None` (the system folder is used).
pub fn chosen(custom: Option<PathBuf>) -> Option<PathBuf> {
    let path = custom?;
    match usable(&path) {
        Ok(path) => Some(path),
        Err(code) => {
            error_log::report(
                Level::Warning,
                "downloads",
                "The chosen download folder cannot be used, so the system Downloads folder is used",
                &format!("{code}: {}", path.display()),
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_must_exist_be_absolute_and_be_writable() {
        let dir = tempfile::tempdir().unwrap();
        assert!(usable(dir.path()).is_ok());
        assert!(usable(&dir.path().join("missing")).is_err());
        assert!(usable(Path::new("relative/folder")).is_err());
        let file = dir.path().join("a.txt");
        std::fs::write(&file, b"x").unwrap();
        assert_eq!(usable(&file).unwrap_err(), "downloads.invalid_location");
        assert_eq!(
            std::fs::read_dir(dir.path()).unwrap().count(),
            1,
            "the write check leaves nothing behind"
        );
    }

    #[test]
    fn a_vanished_folder_falls_back_to_the_system_one() {
        let dir = tempfile::tempdir().unwrap();
        let gone = dir.path().join("gone");
        assert!(chosen(Some(gone)).is_none());
        assert!(chosen(None).is_none());
        assert!(chosen(Some(dir.path().to_path_buf())).is_some());
    }

    #[test]
    fn the_choice_is_saved_and_cleared_in_preferences() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        preferences_repo::initialize_preferences_table(&conn).unwrap();
        let location = DownloadLocation::default();
        location.load(&conn).unwrap();
        assert!(location.custom().is_none());
        let dir = tempfile::tempdir().unwrap();
        store(&conn, Some(dir.path())).unwrap();
        location.load(&conn).unwrap();
        assert_eq!(location.custom().as_deref(), Some(dir.path()));
        store(&conn, None).unwrap();
        location.load(&conn).unwrap();
        assert!(location.custom().is_none());
    }
}
