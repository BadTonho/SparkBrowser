//! Sync choices that apply to every target (Google Drive, folder, WebDAV): how often the automatic
//! sync runs and whether it is paused. They live apart from the target configuration so changing
//! them never touches the target, its account or its saved password.
use crate::database::preferences_repo;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// How often the automatic sync sends changes.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncInterval {
    /// A folder follows each change; Google Drive and WebDAV send after a quiet moment, at most every minute.
    #[default]
    Auto,
    FiveMinutes,
    FifteenMinutes,
    /// Only when the browser closes.
    OnExit,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct SyncOptions {
    pub interval: SyncInterval,
    /// Unix seconds until which the automatic sync does nothing; `None` when it is not paused.
    pub paused_until: Option<i64>,
}

pub fn load(conn: &Connection) -> SyncOptions {
    let Ok(Some(raw)) = preferences_repo::load(conn, preferences_repo::BACKUP_SYNC_OPTIONS_KEY)
    else {
        return SyncOptions::default();
    };
    serde_json::from_str(&raw).unwrap_or_else(|error| {
        crate::error_log::report(
            crate::error_log::Level::Warning,
            "backup",
            "the saved sync options could not be read, using the defaults",
            &error.to_string(),
        );
        SyncOptions::default()
    })
}

pub fn validate(options: &SyncOptions) -> Result<(), String> {
    match options.paused_until {
        Some(until) if until <= 0 => Err("backup.sync_invalid_config".into()),
        _ => Ok(()),
    }
}

pub fn save(conn: &Connection, options: &SyncOptions) -> Result<(), String> {
    validate(options)?;
    let serialized = serde_json::to_string(options).map_err(|e| e.to_string())?;
    preferences_repo::store(conn, preferences_repo::BACKUP_SYNC_OPTIONS_KEY, &serialized)
}

/// Remembers which computer wrote the file in the cloud, as last seen by a send or a download.
pub fn record_device(conn: &Connection, device: Option<&str>) -> Result<(), String> {
    match device {
        Some(name) => {
            preferences_repo::store(conn, preferences_repo::BACKUP_SYNC_LAST_DEVICE_KEY, name)
        }
        None => preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_LAST_DEVICE_KEY),
    }
}

pub fn last_device(conn: &Connection) -> Option<String> {
    preferences_repo::load(conn, preferences_repo::BACKUP_SYNC_LAST_DEVICE_KEY)
        .ok()
        .flatten()
        .filter(|name| !name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        preferences_repo::initialize_preferences_table(&conn).unwrap();
        conn
    }

    #[test]
    fn options_default_to_automatic_and_not_paused_and_round_trip() {
        let conn = db();
        assert_eq!(load(&conn), SyncOptions::default());
        let options = SyncOptions {
            interval: SyncInterval::FifteenMinutes,
            paused_until: Some(1_900_000_000),
        };
        save(&conn, &options).unwrap();
        assert_eq!(load(&conn), options);
        assert!(save(
            &conn,
            &SyncOptions {
                paused_until: Some(0),
                ..options
            }
        )
        .is_err());
    }

    #[test]
    fn unreadable_options_fall_back_to_the_defaults() {
        let conn = db();
        preferences_repo::store(
            &conn,
            preferences_repo::BACKUP_SYNC_OPTIONS_KEY,
            "{\"interval\":\"never\"}",
        )
        .unwrap();
        assert_eq!(load(&conn), SyncOptions::default());
    }

    #[test]
    fn the_last_device_is_remembered_and_forgotten() {
        let conn = db();
        assert_eq!(last_device(&conn), None);
        record_device(&conn, Some("NOTEBOOK")).unwrap();
        assert_eq!(last_device(&conn).as_deref(), Some("NOTEBOOK"));
        record_device(&conn, None).unwrap();
        assert_eq!(last_device(&conn), None);
    }
}
