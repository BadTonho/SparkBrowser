use crate::state::split_layout::SplitLayout;
use rusqlite::{params, Connection};
use std::collections::HashMap;
pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "workspace-splits-v1", |conn| {
        conn.execute_batch("CREATE TABLE workspace_splits(workspace_id TEXT PRIMARY KEY,left_tab_id TEXT NOT NULL,right_tab_id TEXT NOT NULL,ratio REAL NOT NULL,focused_tab_id TEXT NOT NULL)").map_err(|e|e.to_string())
    })?;
    super::migrations::apply(conn, "workspace-split-groups-v1", |conn| {
        conn.execute_batch("ALTER TABLE workspace_splits ADD COLUMN left_tab_ids TEXT NOT NULL DEFAULT '[]'; ALTER TABLE workspace_splits ADD COLUMN right_tab_ids TEXT NOT NULL DEFAULT '[]';").map_err(|e| e.to_string())
    })
}
pub fn load(conn: &Connection) -> Result<HashMap<String, SplitLayout>, String> {
    let mut stmt=conn.prepare("SELECT workspace_id,left_tab_id,right_tab_id,ratio,focused_tab_id,left_tab_ids,right_tab_ids FROM workspace_splits").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            let layout = SplitLayout {
                left_tab_ids: serde_json::from_str(&r.get::<_, String>(5)?).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?,
                right_tab_ids: serde_json::from_str(&r.get::<_, String>(6)?).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?,
                workspace_id: r.get(0)?,
                left_tab_id: r.get(1)?,
                right_tab_id: r.get(2)?,
                ratio: r.get(3)?,
                focused_tab_id: r.get(4)?,
            };
            Ok((layout.workspace_id.clone(), layout))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<HashMap<_, _>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}
pub fn save(conn: &Connection, layouts: &HashMap<String, SplitLayout>) -> Result<(), String> {
    for id in load(conn)?.keys().filter(|id| !layouts.contains_key(*id)) {
        conn.execute("DELETE FROM workspace_splits WHERE workspace_id=?1", [id])
            .map_err(|e| e.to_string())?;
    }
    let mut upsert=conn.prepare_cached("INSERT INTO workspace_splits(workspace_id,left_tab_id,right_tab_id,ratio,focused_tab_id,left_tab_ids,right_tab_ids) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(workspace_id) DO UPDATE SET left_tab_id=excluded.left_tab_id,right_tab_id=excluded.right_tab_id,ratio=excluded.ratio,focused_tab_id=excluded.focused_tab_id,left_tab_ids=excluded.left_tab_ids,right_tab_ids=excluded.right_tab_ids WHERE left_tab_id!=excluded.left_tab_id OR right_tab_id!=excluded.right_tab_id OR ratio!=excluded.ratio OR focused_tab_id!=excluded.focused_tab_id OR left_tab_ids!=excluded.left_tab_ids OR right_tab_ids!=excluded.right_tab_ids").map_err(|e|e.to_string())?;
    for layout in layouts.values() {
        upsert
            .execute(params![
                layout.workspace_id,
                layout.left_tab_id,
                layout.right_tab_id,
                layout.ratio,
                layout.focused_tab_id,
                serde_json::to_string(&layout.left_tab_ids).map_err(|e| e.to_string())?,
                serde_json::to_string(&layout.right_tab_ids).map_err(|e| e.to_string())?
            ])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn group_migration_retains_legacy_selections_and_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE app_migrations(name TEXT PRIMARY KEY); CREATE TABLE workspace_splits(workspace_id TEXT PRIMARY KEY,left_tab_id TEXT NOT NULL,right_tab_id TEXT NOT NULL,ratio REAL NOT NULL,focused_tab_id TEXT NOT NULL); INSERT INTO app_migrations VALUES('workspace-splits-v1'); INSERT INTO workspace_splits VALUES('ws','a','b',0.65,'b');").unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        let mut layout = load(&conn).unwrap().remove("ws").unwrap();
        assert_eq!(layout.left_tab_id, "a");
        assert_eq!(layout.right_tab_id, "b");
        assert_eq!(layout.ratio, 0.65);
        let tabs = ["a", "b", "c"].map(|id| {
            crate::state::tab_model::TabInfo::new(
                id.into(),
                "https://example.test/".into(),
                "ws".into(),
                id == "b",
            )
        });
        layout.normalize(&tabs);
        assert_eq!(layout.left_tab_ids, ["a", "c"]);
        assert_eq!(layout.right_tab_ids, ["b"]);
        assert_eq!(layout.focused_tab_id, "b");
        assert!(layout.valid_groups(&tabs));
    }
    #[test]
    fn layouts_roundtrip_without_rewriting_unchanged_rows() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let layout = SplitLayout {
            left_tab_ids: Vec::new(),
            right_tab_ids: Vec::new(),
            workspace_id: "ws".into(),
            left_tab_id: "a".into(),
            right_tab_id: "b".into(),
            focused_tab_id: "a".into(),
            ratio: 0.65,
        };
        let map = HashMap::from([("ws".into(), layout.clone())]);
        save(&conn, &map).unwrap();
        assert_eq!(load(&conn).unwrap()["ws"], layout);
        let before = conn.total_changes();
        save(&conn, &map).unwrap();
        assert_eq!(conn.total_changes(), before);
        save(&conn, &HashMap::new()).unwrap();
        assert!(load(&conn).unwrap().is_empty());
    }
}
