use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionRecord {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub path: String,
    pub enabled: bool,
    pub manifest_json: String,
    pub installed_at: i64,
}

pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "installed-extensions-v1", |conn| {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS installed_extensions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                path TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                manifest_json TEXT NOT NULL,
                installed_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_extensions_installed_at ON installed_extensions(installed_at DESC);",
        )
        .map_err(|e| e.to_string())
    })
}

pub fn save(conn: &Connection, record: &ExtensionRecord) -> Result<(), String> {
    conn.execute(
        "INSERT INTO installed_extensions (id, name, version, description, path, enabled, manifest_json, installed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            version = excluded.version,
            description = excluded.description,
            path = excluded.path,
            enabled = excluded.enabled,
            manifest_json = excluded.manifest_json",
        params![
            record.id,
            record.name,
            record.version,
            record.description,
            record.path,
            record.enabled as i32,
            record.manifest_json,
            record.installed_at
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_all(conn: &Connection) -> Result<Vec<ExtensionRecord>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, version, description, path, enabled, manifest_json, installed_at FROM installed_extensions ORDER BY installed_at ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ExtensionRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                description: row.get(3)?,
                path: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                manifest_json: row.get(6)?,
                installed_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<ExtensionRecord>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, version, description, path, enabled, manifest_json, installed_at FROM installed_extensions WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query_map([id], |row| {
            Ok(ExtensionRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                description: row.get(3)?,
                path: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                manifest_json: row.get(6)?,
                installed_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    if let Some(row) = rows.next() {
        Ok(Some(row.map_err(|e| e.to_string())?))
    } else {
        Ok(None)
    }
}

pub fn set_enabled(conn: &Connection, id: &str, enabled: bool) -> Result<(), String> {
    conn.execute(
        "UPDATE installed_extensions SET enabled = ?1 WHERE id = ?2",
        params![enabled as i32, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM installed_extensions WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extensions_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        let record = ExtensionRecord {
            id: "ext-1".to_string(),
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            description: "A test extension".to_string(),
            path: "/path/to/ext".to_string(),
            enabled: true,
            manifest_json: "{}".to_string(),
            installed_at: 1000,
        };

        save(&conn, &record).unwrap();

        let loaded = get(&conn, "ext-1").unwrap().unwrap();
        assert_eq!(loaded, record);

        set_enabled(&conn, "ext-1", false).unwrap();
        let loaded = get(&conn, "ext-1").unwrap().unwrap();
        assert!(!loaded.enabled);

        let all = get_all(&conn).unwrap();
        assert_eq!(all.len(), 1);

        delete(&conn, "ext-1").unwrap();
        assert!(get(&conn, "ext-1").unwrap().is_none());
    }
}

