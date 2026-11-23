//! Tab groups of the vertical tab list: named, colored sets of tabs. Stored as one validated JSON
//! document in the durable preferences, so groups survive restarts.
use super::preferences_repo;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const KEY: &str = "tab_groups_v1";
const MAX_GROUPS: usize = 50;
const MAX_MEMBERS: usize = 1000;
const MAX_ID: usize = 64;
const MAX_NAME: usize = 40;
pub const COLORS: [&str; 8] = ["grey", "blue", "red", "yellow", "green", "pink", "purple", "cyan"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Collapse,
    Popup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub color: String,
    pub mode: Mode,
    pub collapsed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct TabGroups {
    pub groups: Vec<Group>,
    /// Tab id to the id of its group.
    pub members: HashMap<String, String>,
}

fn validate(value: &TabGroups) -> Result<(), String> {
    if value.groups.len() > MAX_GROUPS || value.members.len() > MAX_MEMBERS {
        return Err("tab_groups.too_many".into());
    }
    let mut ids = HashSet::new();
    for group in &value.groups {
        if group.id.is_empty()
            || group.id.chars().count() > MAX_ID
            || group.name.chars().count() > MAX_NAME
            || !COLORS.contains(&group.color.as_str())
            || !ids.insert(group.id.as_str())
        {
            return Err("tab_groups.invalid".into());
        }
    }
    for (tab, group) in &value.members {
        if tab.is_empty() || tab.chars().count() > MAX_ID || !ids.contains(group.as_str()) {
            return Err("tab_groups.invalid".into());
        }
    }
    Ok(())
}

pub fn load(conn: &Connection) -> Result<TabGroups, String> {
    let Some(raw) = preferences_repo::load(conn, KEY)? else {
        return Ok(TabGroups::default());
    };
    let value: TabGroups = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    validate(&value)?;
    Ok(value)
}

pub fn save(conn: &Connection, value: &TabGroups) -> Result<(), String> {
    validate(value)?;
    let raw = serde_json::to_string(value).map_err(|e| e.to_string())?;
    preferences_repo::store(conn, KEY, &raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn group(id: &str) -> Group {
        Group { id: id.into(), name: "Work".into(), color: "blue".into(), mode: Mode::Collapse, collapsed: false }
    }
    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        preferences_repo::initialize_preferences_table(&conn).unwrap();
        conn
    }
    #[test]
    fn groups_roundtrip() {
        let conn = database();
        assert_eq!(load(&conn).unwrap(), TabGroups::default());
        let value = TabGroups {
            groups: vec![group("g1")],
            members: HashMap::from([("tab-1".to_string(), "g1".to_string())]),
        };
        save(&conn, &value).unwrap();
        assert_eq!(load(&conn).unwrap(), value);
    }
    #[test]
    fn invalid_documents_are_rejected() {
        let conn = database();
        let member = |group: &str| HashMap::from([("tab-1".to_string(), group.to_string())]);
        assert!(save(&conn, &TabGroups { groups: vec![group("g1")], members: member("missing") }).is_err());
        assert!(save(&conn, &TabGroups { groups: vec![group("g1"), group("g1")], members: HashMap::new() }).is_err());
        let mut bad = group("g1");
        bad.color = "orange".into();
        assert!(save(&conn, &TabGroups { groups: vec![bad], members: HashMap::new() }).is_err());
        let mut long = group("g1");
        long.name = "n".repeat(41);
        assert!(save(&conn, &TabGroups { groups: vec![long], members: HashMap::new() }).is_err());
    }
    #[test]
    fn unknown_fields_and_modes_are_rejected() {
        assert!(serde_json::from_str::<TabGroups>(r#"{"groups":[{"id":"a","name":"","color":"blue","mode":"fly","collapsed":false}]}"#).is_err());
        assert!(serde_json::from_str::<TabGroups>(r#"{"extra":1}"#).is_err());
    }
}
