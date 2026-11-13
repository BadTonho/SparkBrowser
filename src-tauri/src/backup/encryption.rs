//! Backup-encryption setting. The flag lives in SQLite; the password only in the
//! operating-system credential store, never in the database, backups or logs.
use super::{credentials::Credentials, crypto};
use crate::database::preferences_repo as prefs;
use rusqlite::Connection;
use sha2::{Digest, Sha256};

pub fn password_key(conn: &Connection) -> String {
    format!(
        "org.veloce.browser/backup-password/{:x}",
        Sha256::digest(conn.path().unwrap_or("memory").as_bytes())
    )
}

pub fn enabled(conn: &Connection) -> Result<bool, String> {
    prefs::load_bool(conn, prefs::BACKUP_ENCRYPTION_KEY)
}

/// Password used when this app writes a backup. `None` means "write plaintext".
/// When encryption is on but the secret cannot be read, writing fails instead of
/// silently producing an unencrypted file.
pub fn write_password(
    conn: &Connection,
    credentials: &Credentials,
) -> Result<Option<String>, String> {
    if !enabled(conn)? {
        return Ok(None);
    }
    credentials
        .read(&password_key(conn))?
        .filter(|password| !password.is_empty())
        .map(Some)
        .ok_or_else(|| "backup.password_missing".to_string())
}

/// Best-effort password for reading. A missing or locked store yields `None`, so
/// callers surface `backup.password_required` and the user can type it.
pub fn read_password(conn: &Connection, credentials: &Credentials) -> Option<String> {
    credentials
        .read(&password_key(conn))
        .ok()
        .flatten()
        .filter(|password| !password.is_empty())
}

/// Enable (with a password), change the password, or disable encryption.
/// Existing backups keep the password they were written with.
pub fn configure(
    conn: &Connection,
    credentials: &Credentials,
    enable: bool,
    password: Option<String>,
) -> Result<(), String> {
    if !enable {
        credentials.delete(&password_key(conn))?;
        return prefs::remove(conn, prefs::BACKUP_ENCRYPTION_KEY);
    }
    match password.filter(|value| !value.is_empty()) {
        Some(password) => {
            crypto::validate_password(&password)?;
            credentials.write(&password_key(conn), &password)?;
        }
        // Re-saving the toggle without a new password is only valid once one is stored.
        None if enabled(conn)? && read_password(conn, credentials).is_some() => {}
        None => return Err("backup.password_invalid".into()),
    }
    prefs::store(conn, prefs::BACKUP_ENCRYPTION_KEY, "true")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        prefs::initialize_preferences_table(&conn).unwrap();
        conn
    }

    #[test]
    fn password_lives_in_credential_store_not_database() {
        let (conn, credentials) = (conn(), Credentials::default());
        assert_eq!(write_password(&conn, &credentials).unwrap(), None);
        configure(&conn, &credentials, true, Some("correct horse".into())).unwrap();
        assert_eq!(
            write_password(&conn, &credentials).unwrap().as_deref(),
            Some("correct horse")
        );
        let stored: String = conn
            .query_row(
                "SELECT group_concat(key || value) FROM app_preferences",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!stored.contains("correct horse"));
        // Re-saving without a password keeps the current one.
        configure(&conn, &credentials, true, None).unwrap();
        assert!(read_password(&conn, &credentials).is_some());
        configure(&conn, &credentials, false, None).unwrap();
        assert_eq!(write_password(&conn, &credentials).unwrap(), None);
        assert!(read_password(&conn, &credentials).is_none());
    }

    #[test]
    fn enabling_requires_a_valid_password_and_never_falls_back_to_plaintext() {
        let (conn, credentials) = (conn(), Credentials::default());
        assert_eq!(
            configure(&conn, &credentials, true, None).unwrap_err(),
            "backup.password_invalid"
        );
        assert_eq!(
            configure(&conn, &credentials, true, Some("short".into())).unwrap_err(),
            "backup.password_invalid"
        );
        assert!(!enabled(&conn).unwrap());
        configure(&conn, &credentials, true, Some("correct horse".into())).unwrap();
        credentials.delete(&password_key(&conn)).unwrap();
        assert_eq!(
            write_password(&conn, &credentials).unwrap_err(),
            "backup.password_missing"
        );
    }
}
