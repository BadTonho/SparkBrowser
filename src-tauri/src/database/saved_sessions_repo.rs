use crate::state::tab_model::TabInfo;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionPage {
    pub title: String,
    pub url: String,
    pub is_pinned: bool,
    pub is_selected: bool,
}

#[derive(Serialize)]
pub struct SavedSession {
    pub id: i64,
    pub name: String,
    pub pages: Vec<SessionPage>,
}

pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "named-sessions-v1", |conn| {
        conn.execute_batch("CREATE TABLE saved_sessions(id INTEGER PRIMARY KEY, name TEXT NOT NULL);
            CREATE TABLE saved_session_pages(session_id INTEGER NOT NULL, position INTEGER NOT NULL,
            title TEXT NOT NULL, url TEXT NOT NULL, is_pinned INTEGER NOT NULL, is_selected INTEGER NOT NULL,
            PRIMARY KEY(session_id, position));").map_err(|e| e.to_string())
    })
}

pub fn list(conn: &Connection) -> Result<Vec<SavedSession>, String> {
    let mut statement = conn
        .prepare("SELECT id, name FROM saved_sessions ORDER BY id DESC")
        .map_err(|e| e.to_string())?;
    let headers = statement
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    headers
        .into_iter()
        .map(|(id, name)| {
            Ok(SavedSession {
                id,
                name,
                pages: pages(conn, id)?,
            })
        })
        .collect()
}

pub fn pages(conn: &Connection, id: i64) -> Result<Vec<SessionPage>, String> {
    let mut statement = conn.prepare("SELECT title,url,is_pinned,is_selected FROM saved_session_pages WHERE session_id=?1 ORDER BY position").map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([id], |row| {
            Ok(SessionPage {
                title: row.get(0)?,
                url: row.get(1)?,
                is_pinned: row.get(2)?,
                is_selected: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if rows.is_empty() {
        return Err("Saved session was not found or is empty".into());
    }
    Ok(rows)
}

pub fn save(
    conn: &Connection,
    id: Option<i64>,
    name: &str,
    tabs: &[TabInfo],
) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err("Session name must contain 1–100 characters".into());
    }
    if tabs.is_empty() {
        return Err("There are no tabs to save".into());
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let id = if let Some(id) = id {
        if tx
            .execute(
                "UPDATE saved_sessions SET name=?1 WHERE id=?2",
                params![name, id],
            )
            .map_err(|e| e.to_string())?
            == 0
        {
            return Err("Saved session was not found".into());
        }
        tx.execute("DELETE FROM saved_session_pages WHERE session_id=?1", [id])
            .map_err(|e| e.to_string())?;
        id
    } else {
        tx.execute("INSERT INTO saved_sessions(name) VALUES (?1)", [name])
            .map_err(|e| e.to_string())?;
        tx.last_insert_rowid()
    };
    {
        let mut insert = tx
            .prepare("INSERT INTO saved_session_pages VALUES (?1,?2,?3,?4,?5,?6)")
            .map_err(|e| e.to_string())?;
        for (position, tab) in tabs.iter().enumerate() {
            insert
                .execute(params![
                    id,
                    position as i64,
                    tab.title,
                    tab.url,
                    tab.is_pinned,
                    tab.is_active
                ])
                .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())
}

pub fn rename(conn: &Connection, id: i64, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err("Session name must contain 1–100 characters".into());
    }
    if conn
        .execute(
            "UPDATE saved_sessions SET name=?1 WHERE id=?2",
            params![name, id],
        )
        .map_err(|e| e.to_string())?
        == 0
    {
        return Err("Saved session was not found".into());
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM saved_session_pages WHERE session_id=?1", [id])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM saved_sessions WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn session_updates_are_atomic_and_keep_order_and_selection() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        let mut tabs = vec![
            TabInfo::new_default("one".into(), "https://one.test/".into(), false),
            TabInfo::new_default("two".into(), "https://two.test/".into(), true),
        ];
        tabs[0].is_pinned = true;
        save(&conn, None, "Study", &tabs).unwrap();
        let first = list(&conn).unwrap().remove(0);
        assert!(first.pages[0].is_pinned);
        assert!(first.pages[1].is_selected);
        conn.execute_batch("CREATE TRIGGER reject_page BEFORE INSERT ON saved_session_pages WHEN NEW.position=1 BEGIN SELECT RAISE(ABORT,'fixture failure'); END;").unwrap();
        assert!(save(&conn, Some(first.id), "Changed", &tabs).is_err());
        assert_eq!(list(&conn).unwrap()[0].name, "Study");
        assert_eq!(pages(&conn, first.id).unwrap().len(), 2);
        rename(&conn, first.id, "Renamed").unwrap();
        delete(&conn, first.id).unwrap();
        assert!(list(&conn).unwrap().is_empty());
    }
}
