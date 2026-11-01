use crate::state::split_layout::SplitLayout;
use rusqlite::{params, Connection};
use std::collections::HashMap;
pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "workspace-splits-v1", |conn| {
        conn.execute_batch("CREATE TABLE workspace_splits(workspace_id TEXT PRIMARY KEY,left_tab_id TEXT NOT NULL,right_tab_id TEXT NOT NULL,ratio REAL NOT NULL,focused_tab_id TEXT NOT NULL)").map_err(|e|e.to_string())
    })
}
pub fn load(conn: &Connection) -> Result<HashMap<String, SplitLayout>, String> {
    let mut stmt=conn.prepare("SELECT workspace_id,left_tab_id,right_tab_id,ratio,focused_tab_id FROM workspace_splits").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            let layout = SplitLayout {
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
    let mut upsert=conn.prepare_cached("INSERT INTO workspace_splits VALUES(?1,?2,?3,?4,?5) ON CONFLICT(workspace_id) DO UPDATE SET left_tab_id=excluded.left_tab_id,right_tab_id=excluded.right_tab_id,ratio=excluded.ratio,focused_tab_id=excluded.focused_tab_id WHERE left_tab_id!=excluded.left_tab_id OR right_tab_id!=excluded.right_tab_id OR ratio!=excluded.ratio OR focused_tab_id!=excluded.focused_tab_id").map_err(|e|e.to_string())?;
    for layout in layouts.values() {
        upsert
            .execute(params![
                layout.workspace_id,
                layout.left_tab_id,
                layout.right_tab_id,
                layout.ratio,
                layout.focused_tab_id
            ])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layouts_roundtrip_without_rewriting_unchanged_rows() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let layout = SplitLayout {
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
