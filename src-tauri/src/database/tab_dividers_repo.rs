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

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DividerEdit {
    Add { divider: TabDivider },
    Rename { id: String, name: String },
    Move { id: String, after: String },
    Remove { id: String },
    Import { dividers: Vec<TabDivider> },
}

impl DividerEdit {
    /// An edit queued just before its target tab closed can still use the top of the list.
    pub fn repair_anchor(&mut self, exists: impl FnOnce(&str) -> bool) {
        let after = match self {
            Self::Add { divider } => &mut divider.after,
            Self::Move { after, .. } => after,
            _ => return,
        };
        if !after.is_empty() && !exists(after) {
            after.clear();
        }
    }
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

pub fn edit(conn: &Connection, edit: DividerEdit) -> Result<Vec<TabDivider>, String> {
    let mut items = load(conn)?;
    match edit {
        DividerEdit::Add { divider } => {
            if !items.iter().any(|item| item.id == divider.id) {
                items.push(divider);
            }
        }
        DividerEdit::Rename { id, name } => {
            if let Some(item) = items.iter_mut().find(|item| item.id == id) {
                item.name = name;
            }
        }
        DividerEdit::Move { id, after } => {
            if let Some(item) = items.iter_mut().find(|item| item.id == id) {
                item.after = after;
            }
        }
        DividerEdit::Remove { id } => items.retain(|item| item.id != id),
        DividerEdit::Import { dividers } => {
            validate(&dividers)?;
            for divider in dividers {
                if !items.iter().any(|item| item.id == divider.id) {
                    items.push(divider);
                }
            }
        }
    }
    save(conn, &items)?;
    Ok(items)
}

pub fn reanchor(
    conn: &Connection,
    anchors: &std::collections::HashMap<&str, &str>,
) -> Result<(), String> {
    let mut items = load(conn)?;
    let mut changed = false;
    for item in &mut items {
        if let Some(&after) = anchors.get(item.after.as_str()) {
            item.after = after.to_owned();
            changed = true;
        }
    }
    if changed {
        save(conn, &items)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn divider(id: &str, name: &str) -> TabDivider {
        TabDivider {
            id: id.into(),
            after: "tab-1".into(),
            name: name.into(),
        }
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
