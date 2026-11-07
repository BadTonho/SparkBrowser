use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AccountContainer {
    pub id: String,
    pub name: String,
    pub deleting: bool,
}

pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "account-containers-v1", |conn| {
        conn.execute_batch("CREATE TABLE account_containers(id TEXT PRIMARY KEY, name TEXT NOT NULL, deleting INTEGER NOT NULL DEFAULT 0); ALTER TABLE session_tabs ADD COLUMN container_id TEXT;").map_err(|e| e.to_string())
    })
}

pub fn list(conn: &Connection) -> Result<Vec<AccountContainer>, String> {
    conn.prepare("SELECT id, name, deleting FROM account_containers ORDER BY name COLLATE NOCASE, id")
        .map_err(|e| e.to_string())?
        .query_map([], |row| Ok(AccountContainer { id: row.get(0)?, name: row.get(1)?, deleting: row.get(2)? }))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn valid_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn validate_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) { return Err("Invalid container name".into()); }
    Ok(name)
}

pub fn insert(conn: &Connection, id: &str, name: &str) -> Result<(), String> {
    if !valid_id(id) { return Err("Invalid container ID".into()); }
    conn.execute("INSERT INTO account_containers(id,name) VALUES (?1,?2)", params![id, validate_name(name)?]).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn rename(conn: &Connection, id: &str, name: &str) -> Result<(), String> {
    if conn.execute("UPDATE account_containers SET name=?2 WHERE id=?1 AND deleting=0", params![id, validate_name(name)?]).map_err(|e| e.to_string())? != 1 { return Err("Container not found".into()); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_and_validation() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE session_tabs(tab_id TEXT PRIMARY KEY)").unwrap();
        migrate(&conn).unwrap(); migrate(&conn).unwrap();
        let id = "0123456789abcdef0123456789abcdef";
        insert(&conn,id,"Work").unwrap(); rename(&conn,id,"Personal").unwrap();
        assert_eq!(list(&conn).unwrap()[0].name,"Personal");
        assert!(insert(&conn,"../invalid","x").is_err());
        assert!(rename(&conn,id," ").is_err());
    }
}
