//! Storage of the password vault: one header row and one encrypted blob per entry.
//!
//! Nothing here can read a secret: entries arrive and leave already encrypted. These tables are not
//! part of `BackupData` (so backups, WebDAV sync and Drive never carry them) and are not copied into
//! private-window profiles.
use crate::vault::crypto::Header;
use rusqlite::{params, Connection, OptionalExtension};

pub struct Row {
    pub id: String,
    pub blob: Vec<u8>,
    pub updated_at: i64,
}

pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "vault-v1", |conn| {
        conn.execute_batch(
            "CREATE TABLE vault_meta(
                id INTEGER PRIMARY KEY CHECK(id=1),
                header TEXT NOT NULL,
                auto_lock_minutes INTEGER NOT NULL
            );
            CREATE TABLE vault_entries(
                id TEXT PRIMARY KEY,
                blob BLOB NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        )
        .map_err(|e| e.to_string())
    })
}

pub fn load_header(conn: &Connection) -> Result<Option<Header>, String> {
    let text: Option<String> = conn
        .query_row("SELECT header FROM vault_meta WHERE id=1", [], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())?;
    text.map(|text| serde_json::from_str(&text).map_err(|_| "vault.corrupt".to_string()))
        .transpose()
}

/// Writes the first header. Fails if a vault already exists, so one can never be overwritten by accident.
pub fn create(conn: &Connection, header: &Header, auto_lock_minutes: u32) -> Result<(), String> {
    let text = serde_json::to_string(header).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO vault_meta(id,header,auto_lock_minutes) VALUES(1,?1,?2)",
        params![text, auto_lock_minutes],
    )
    .map(|_| ())
    .map_err(|_| "vault.already_initialized".to_string())
}

pub fn update_header(conn: &Connection, header: &Header) -> Result<(), String> {
    let text = serde_json::to_string(header).map_err(|e| e.to_string())?;
    let changed = conn
        .execute("UPDATE vault_meta SET header=?1 WHERE id=1", [text])
        .map_err(|e| e.to_string())?;
    if changed == 1 {
        Ok(())
    } else {
        Err("vault.not_initialized".into())
    }
}

pub fn auto_lock_minutes(conn: &Connection) -> Result<Option<u32>, String> {
    conn.query_row("SELECT auto_lock_minutes FROM vault_meta WHERE id=1", [], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())
}

pub fn set_auto_lock_minutes(conn: &Connection, minutes: u32) -> Result<(), String> {
    conn.execute("UPDATE vault_meta SET auto_lock_minutes=?1 WHERE id=1", [minutes])
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn count(conn: &Connection) -> Result<usize, String> {
    conn.query_row("SELECT COUNT(*) FROM vault_entries", [], |row| row.get::<_, i64>(0))
        .map(|n| n.max(0) as usize)
        .map_err(|e| e.to_string())
}

pub fn rows(conn: &Connection) -> Result<Vec<Row>, String> {
    let mut statement = conn
        .prepare("SELECT id,blob,updated_at FROM vault_entries ORDER BY updated_at DESC,id")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(Row {
                id: row.get(0)?,
                blob: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn row(conn: &Connection, id: &str) -> Result<Option<Row>, String> {
    conn.query_row(
        "SELECT id,blob,updated_at FROM vault_entries WHERE id=?1",
        [id],
        |row| {
            Ok(Row {
                id: row.get(0)?,
                blob: row.get(1)?,
                updated_at: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(|e| e.to_string())
}

pub fn put(conn: &Connection, id: &str, blob: &[u8], updated_at: i64) -> Result<(), String> {
    conn.execute(
        "INSERT INTO vault_entries(id,blob,updated_at) VALUES(?1,?2,?3)
         ON CONFLICT(id) DO UPDATE SET blob=excluded.blob,updated_at=excluded.updated_at",
        params![id, blob, updated_at],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

pub fn delete(conn: &Connection, id: &str) -> Result<bool, String> {
    conn.execute("DELETE FROM vault_entries WHERE id=?1", [id])
        .map(|changed| changed > 0)
        .map_err(|e| e.to_string())
}

/// Removes the header and every entry. Used only when the user has lost the password and the
/// recovery key and chooses to start over: the old entries cannot be read anyway.
pub fn wipe(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("DELETE FROM vault_entries; DELETE FROM vault_meta;")
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::crypto;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    #[test]
    fn the_migration_is_idempotent_and_an_empty_database_has_no_vault() {
        let conn = db();
        migrate(&conn).unwrap();
        assert!(load_header(&conn).unwrap().is_none());
        assert_eq!(count(&conn).unwrap(), 0);
        assert_eq!(auto_lock_minutes(&conn).unwrap(), None);
    }

    #[test]
    fn the_header_is_stored_once_and_can_be_replaced() {
        let conn = db();
        let (header, _, _) = crypto::create(Some("correct horse battery"), None).unwrap();
        create(&conn, &header, 5).unwrap();
        assert_eq!(load_header(&conn).unwrap(), Some(header.clone()));
        assert_eq!(create(&conn, &header, 5).unwrap_err(), "vault.already_initialized");
        let (other, _, _) = crypto::create(Some("another long password"), None).unwrap();
        update_header(&conn, &other).unwrap();
        assert_eq!(load_header(&conn).unwrap(), Some(other));
        set_auto_lock_minutes(&conn, 15).unwrap();
        assert_eq!(auto_lock_minutes(&conn).unwrap(), Some(15));
    }

    #[test]
    fn entries_are_stored_as_given_replaced_and_deleted() {
        let conn = db();
        put(&conn, "a", b"one", 1).unwrap();
        put(&conn, "b", b"two", 2).unwrap();
        put(&conn, "a", b"three", 3).unwrap();
        assert_eq!(count(&conn).unwrap(), 2);
        let all = rows(&conn).unwrap();
        assert_eq!(all.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["a", "b"]);
        assert_eq!(row(&conn, "a").unwrap().unwrap().blob, b"three");
        assert!(delete(&conn, "a").unwrap());
        assert!(!delete(&conn, "a").unwrap());
        assert!(row(&conn, "a").unwrap().is_none());
    }

    #[test]
    fn wiping_removes_everything_and_allows_a_new_vault() {
        let conn = db();
        let (header, _, _) = crypto::create(Some("correct horse battery"), None).unwrap();
        create(&conn, &header, 5).unwrap();
        put(&conn, "a", b"one", 1).unwrap();
        wipe(&conn).unwrap();
        assert!(load_header(&conn).unwrap().is_none());
        assert_eq!(count(&conn).unwrap(), 0);
        create(&conn, &header, 5).unwrap();
    }

    #[test]
    fn a_damaged_header_is_reported_as_corrupt_not_as_missing() {
        let conn = db();
        conn.execute("INSERT INTO vault_meta(id,header,auto_lock_minutes) VALUES(1,'{not json',5)", [])
            .unwrap();
        assert_eq!(load_header(&conn).unwrap_err(), "vault.corrupt");
    }
}
