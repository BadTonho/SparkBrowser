//! What Settings shows next to each profile besides its name: whether it asks for an app lock when it
//! opens, and which profile Spark opens when it starts.
//!
//! The lock is read from each profile's own database, opened read-only so nothing is created or
//! changed in a profile that is not running here, even when another process has it open.
use super::launcher::{self, Startup};
use super::{store::ProfileInfo, DEFAULT_ID};
use crate::error_log::{self, Level};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use std::path::Path;
use std::time::Duration;

/// Whether the profile whose data folder this is has an app lock. A profile that never ran has no
/// database and therefore no lock. `None` when its database could not be read (it is reported).
pub fn has_app_lock(id: &str, data_dir: &Path) -> Option<bool> {
    let path = crate::database::connection::database_path(data_dir);
    if !path.is_file() {
        return Some(false);
    }
    read_lock(&path).map_or_else(
        |details| {
            error_log::report(
                Level::Warning,
                "profiles",
                "whether a profile has an app lock could not be read",
                &format!("{id}: {details}"),
            );
            None
        },
        Some,
    )
}

fn read_lock(path: &Path) -> Result<bool, String> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| e.to_string())?;
    conn.busy_timeout(Duration::from_millis(500))
        .map_err(|e| e.to_string())?;
    let table: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='app_lock'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if table.is_none() {
        return Ok(false);
    }
    conn.query_row("SELECT 1 FROM app_lock WHERE id=1", [], |row| {
        row.get::<_, i64>(0)
    })
    .optional()
    .map(|row| row.is_some())
    .map_err(|e| e.to_string())
}

/// The profile Spark opens when it starts without `--profile`, or `None` when it asks first.
pub fn startup_profile(root: &Path, profiles: &[ProfileInfo]) -> Option<String> {
    match launcher::decide(profiles, &launcher::load(root)) {
        Startup::Profile(None) => Some(DEFAULT_ID.to_string()),
        Startup::Profile(Some(id)) => Some(id),
        Startup::Picker => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_profile_without_a_database_has_no_lock() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(has_app_lock("0123456789ab", dir.path()), Some(false));
        assert!(!crate::database::connection::database_path(dir.path()).exists());
    }

    #[test]
    fn the_lock_is_read_from_the_profile_database_without_changing_it() {
        let dir = tempfile::tempdir().unwrap();
        let conn = crate::database::connection::initialize_profile_database(dir.path()).unwrap();
        assert_eq!(has_app_lock("0123456789ab", dir.path()), Some(false));
        crate::app_lock::ops::enable(
            &conn,
            crate::app_lock::ops::First::Password("a long secret"),
        )
        .unwrap();
        assert_eq!(has_app_lock("0123456789ab", dir.path()), Some(true));
    }

    #[test]
    fn a_database_without_the_lock_table_has_no_lock() {
        let dir = tempfile::tempdir().unwrap();
        let path = crate::database::connection::database_path(dir.path());
        Connection::open(&path)
            .unwrap()
            .execute_batch("CREATE TABLE other(id INTEGER);")
            .unwrap();
        assert_eq!(has_app_lock("0123456789ab", dir.path()), Some(false));
    }
}
