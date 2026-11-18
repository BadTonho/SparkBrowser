use crate::state::{tab_model::TabInfo, workspace_model::WorkspaceItem};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

pub const RETENTION_SECONDS: i64 = 30 * 24 * 60 * 60;
pub const MAX_RECORDS: usize = 100;

#[derive(Clone, Debug, Serialize)]
pub struct ClosedTab {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub workspace_id: String,
    pub workspace_name: String,
    pub position: usize,
    pub is_pinned: bool,
    pub closed_at: i64,
}
pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "recently-closed-tabs-v1", |conn| {
        conn.execute_batch(
            "CREATE TABLE closed_tabs(
            id INTEGER PRIMARY KEY AUTOINCREMENT, url TEXT NOT NULL, title TEXT NOT NULL,
            workspace_id TEXT NOT NULL, workspace_name TEXT NOT NULL, position INTEGER NOT NULL,
            is_pinned INTEGER NOT NULL, closed_at INTEGER NOT NULL);
            CREATE INDEX idx_closed_tabs_recent ON closed_tabs(closed_at DESC,id DESC);",
        )
        .map_err(|e| e.to_string())
    })
}
pub fn records(
    tabs: &[TabInfo],
    ids: &[String],
    workspaces: &[WorkspaceItem],
    now: i64,
) -> Vec<ClosedTab> {
    let ids: HashSet<_> = ids.iter().collect();
    let mut positions: HashMap<&str, usize> = HashMap::new();
    tabs.iter()
        .filter(|tab| !tab.is_ephemeral)
        .filter_map(|tab| {
            let position = positions.entry(&tab.workspace_id).or_default();
            let row = ids.contains(&tab.id).then(|| ClosedTab {
                id: 0,
                url: tab.url.clone(),
                title: tab.title.clone(),
                workspace_id: tab.workspace_id.clone(),
                workspace_name: workspaces
                    .iter()
                    .find(|ws| ws.id == tab.workspace_id)
                    .map(|ws| ws.name.clone())
                    .unwrap_or_else(|| tab.workspace_id.clone()),
                position: *position,
                is_pinned: tab.is_pinned,
                closed_at: now,
            });
            *position += 1;
            row
        })
        .collect()
}
pub fn prune(conn: &Connection, now: i64) -> Result<(), String> {
    conn.execute(
        "DELETE FROM closed_tabs WHERE closed_at <= ?1",
        [now.saturating_sub(RETENTION_SECONDS)],
    )
    .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM closed_tabs WHERE id NOT IN (SELECT id FROM closed_tabs ORDER BY closed_at DESC,id DESC LIMIT ?1)", [MAX_RECORDS as i64]).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn append(conn: &Connection, rows: &[ClosedTab], now: i64) -> Result<(), String> {
    let mut insert = conn.prepare_cached("INSERT INTO closed_tabs(url,title,workspace_id,workspace_name,position,is_pinned,closed_at) VALUES(?1,?2,?3,?4,?5,?6,?7)").map_err(|e|e.to_string())?;
    for row in rows {
        insert
            .execute(params![
                row.url,
                row.title,
                row.workspace_id,
                row.workspace_name,
                row.position as i64,
                row.is_pinned,
                row.closed_at
            ])
            .map_err(|e| e.to_string())?;
    }
    prune(conn, now)
}
pub fn list(conn: &Connection, now: i64) -> Result<Vec<ClosedTab>, String> {
    prune(conn, now)?;
    conn.prepare_cached("SELECT id,url,title,workspace_id,workspace_name,position,is_pinned,closed_at FROM closed_tabs ORDER BY closed_at DESC,id DESC").map_err(|e|e.to_string())?
        .query_map([], |row| Ok(ClosedTab{id:row.get(0)?,url:row.get(1)?,title:row.get(2)?,workspace_id:row.get(3)?,workspace_name:row.get(4)?,position:row.get::<_,i64>(5)?.max(0) as usize,is_pinned:row.get(6)?,closed_at:row.get(7)?}))
        .map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}
pub fn consume(conn: &Connection, id: i64) -> Result<(), String> {
    if conn
        .execute("DELETE FROM closed_tabs WHERE id=?1", [id])
        .map_err(|e| e.to_string())?
        != 1
    {
        return Err("closed_tabs.not_found".into());
    }
    Ok(())
}
/// Forgets the closed tabs from before `cutoff` (a Unix time).
pub fn clear_before(conn: &Connection, cutoff: i64) -> Result<(), String> {
    conn.execute("DELETE FROM closed_tabs WHERE closed_at < ?1", [cutoff])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM closed_tabs", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expiry_limit_and_workspace_positions_are_preserved() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let now = RETENTION_SECONDS + 1000;
        let tabs: Vec<_> = (0..120)
            .map(|i| {
                TabInfo::new(
                    format!("t{i}"),
                    format!("https://example.test/{i}"),
                    if i % 2 == 0 { "a" } else { "b" }.into(),
                    false,
                )
            })
            .collect();
        let ids = tabs.iter().map(|tab| tab.id.clone()).collect::<Vec<_>>();
        let records = records(&tabs, &ids, &[], now);
        assert_eq!(records[7].position, 3);
        append(&conn, &records, now).unwrap();
        let rows = list(&conn, now).unwrap();
        assert_eq!(rows.len(), 100);
        assert_eq!(rows[0].url, "https://example.test/119");
        assert_eq!(list(&conn, now + RETENTION_SECONDS - 1).unwrap().len(), 100);
        assert!(list(&conn, now + RETENTION_SECONDS).unwrap().is_empty());
    }
}
