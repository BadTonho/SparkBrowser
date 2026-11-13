use rusqlite::{params, Connection, OptionalExtension};

/// Named durable preferences are stored as strings; callers own their validation.
pub const WORKSPACE_ISOLATION_KEY: &str = "workspace_isolation";
pub const HISTORY_RETENTION_KEY: &str = "history_retention";
pub const ALLOW_POPUPS_KEY: &str = "allow_popups";
pub const SMART_SCREEN_KEY: &str = "smart_screen_enabled";
pub const TRACKING_CLEANING_KEY: &str = "tracking_cleaning_v1";
pub const GPC_KEY: &str = "global_privacy_control_v1";
pub const SHIELD_EXCEPTIONS_KEY: &str = "shield_exceptions_v1";
pub const BACKUP_SYNC_CONFIG_KEY: &str = "backup_sync_config_v1";
pub const BACKUP_SYNC_SECRET_KEY: &str = "backup_sync_secret_v1";
pub const BACKUP_SYNC_CREDENTIAL_KEY: &str = "backup_sync_credential_v2";
pub const BACKUP_SYNC_LAST_AT_KEY: &str = "backup_sync_last_at_v1";
pub const BACKUP_SYNC_FINGERPRINT_KEY: &str = "backup_sync_fingerprint_v1";
pub const BACKUP_SYNC_GDRIVE_EMAIL_KEY: &str = "backup_sync_gdrive_email_v1";
pub const BACKUP_SYNC_GDRIVE_AUTHUSER_KEY: &str = "backup_sync_gdrive_authuser_v1";
// The `backup_sync` prefix keeps these out of private-profile copies.
pub const BACKUP_ENCRYPTION_KEY: &str = "backup_sync_encryption_v1";
pub const BACKUP_LOCAL_SCHEDULE_KEY: &str = "backup_sync_local_schedule_v1";
pub const BACKUP_LOCAL_LAST_RUN_KEY: &str = "backup_sync_local_last_run_v1";
pub const BACKUP_LOCAL_FINGERPRINT_KEY: &str = "backup_sync_local_fingerprint_v1";

pub fn initialize_preferences_table(conn: &Connection) -> Result<(), String> {
    crate::database::migrations::apply(conn, "app-preferences-v1", |tx| {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS app_preferences (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )
        .map_err(|e| format!("Failed to create app_preferences table: {e}"))?;
        Ok(())
    })
}

pub fn load(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT value FROM app_preferences WHERE key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}

pub fn store(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO app_preferences (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove(conn: &Connection, key: &str) -> Result<(), String> {
    conn.execute("DELETE FROM app_preferences WHERE key = ?1", [key])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Unknown or absent values leave the global default (false) in force.
pub fn load_bool(conn: &Connection, key: &str) -> Result<bool, String> {
    Ok(load(conn, key)?.as_deref() == Some("true"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        initialize_preferences_table(&conn).unwrap();
        conn
    }

    #[test]
    fn absent_values_fall_back_to_the_default() {
        let conn = database();
        assert!(!load_bool(&conn, WORKSPACE_ISOLATION_KEY).unwrap());
    }

    #[test]
    fn stores_and_reloads_the_last_written_value() {
        let conn = database();
        assert!(load_bool(&conn, WORKSPACE_ISOLATION_KEY).is_ok());
        store(&conn, WORKSPACE_ISOLATION_KEY, "true").unwrap();
        assert!(load_bool(&conn, WORKSPACE_ISOLATION_KEY).unwrap());
        store(&conn, WORKSPACE_ISOLATION_KEY, "false").unwrap();
        assert!(!load_bool(&conn, WORKSPACE_ISOLATION_KEY).unwrap());
    }

    #[test]
    fn initialization_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_preferences_table(&conn).unwrap();
        store(&conn, WORKSPACE_ISOLATION_KEY, "true").unwrap();
        initialize_preferences_table(&conn).unwrap();
        assert!(load_bool(&conn, WORKSPACE_ISOLATION_KEY).unwrap());
    }
}
