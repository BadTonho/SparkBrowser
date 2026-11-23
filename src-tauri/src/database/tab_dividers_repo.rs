//! Named divider lines of the vertical tab list. The list is stored as one validated JSON document
//! in the durable preferences, so it survives restarts and is not tied to the web view's storage.
use super::preferences_repo;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

const KEY: &str = "tab_dividers_v1";
const MAX_DIVIDERS: usize = 200;
const MAX_ID: usize = 64;
const MAX_NAME: usize = 40;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TabDivider {
    pub id: String,
    /// The tab the divider sits below; empty for the top of the list.
    pub after: String,
    pub name: String,
}

fn validate(items: &[TabDivider]) -> Result<(), String> {
    if items.len() > MAX_DIVIDERS {
        return Err("tab_dividers.too_many".into());
    }
    let mut seen = std::collections::HashSet::new();
    for item in items {
        if item.id.is_empty()
            || item.id.chars().count() > MAX_ID
            || item.after.chars().count() > MAX_ID
            || item.name.chars().count() > MAX_NAME
            || !seen.insert(item.id.as_str())
        {
            return Err("tab_dividers.invalid".into());
        }
    }
    Ok(())
}

pub fn load(conn: &Connection) -> Result<Vec<TabDivider>, String> {
    let Some(raw) = preferences_repo::load(conn, KEY)? else {
        return Ok(Vec::new());
    };
    let items: Vec<TabDivider> = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    validate(&items)?;
    Ok(items)
}

pub fn save(conn: &Connection, items: &[TabDivider]) -> Result<(), String> {
    validate(items)?;
    let raw = serde_json::to_string(items).map_err(|e| e.to_string())?;
    preferences_repo::store(conn, KEY, &raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn divider(id: &str, name: &str) -> TabDivider {
        TabDivider { id: id.into(), after: "tab-1".into(), name: name.into() }
    }
    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        preferences_repo::initialize_preferences_table(&conn).unwrap();
        conn
    }
    #[test]
    fn dividers_roundtrip_and_survive_a_new_connection_read() {
        let conn = database();
        assert!(load(&conn).unwrap().is_empty());
        let items = vec![divider("a", "Work"), divider("b", "")];
        save(&conn, &items).unwrap();
        assert_eq!(load(&conn).unwrap(), items);
    }
    #[test]
    fn invalid_lists_are_rejected() {
        let conn = database();
        assert!(save(&conn, &[divider("a", "x"), divider("a", "y")]).is_err());
        assert!(save(&conn, &[divider("", "x")]).is_err());
        assert!(save(&conn, &[divider("a", &"n".repeat(41))]).is_err());
        assert!(save(&conn, &vec![divider("a", "x"); 1]).is_ok());
    }
}
