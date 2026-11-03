use crate::backup::model::*;
use rusqlite::{params, Connection, OptionalExtension};

pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "backup-preference-outbox-v1", |conn| {
        conn.execute_batch(
        "CREATE TABLE backup_preference_tasks(id TEXT PRIMARY KEY, preferences TEXT NOT NULL, error TEXT);"
    ).map_err(|e|e.to_string())
    })
}
pub fn collect(conn: &Connection) -> Result<BackupData, String> {
    let notes = conn
        .prepare("SELECT url,title,text,updated_at FROM page_notes ORDER BY url")
        .map_err(|e| e.to_string())?
        .query_map([], |r| {
            Ok(super::notes_repo::PageNote {
                url: r.get(0)?,
                title: r.get(1)?,
                text: r.get(2)?,
                updated_at: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut split_layouts: Vec<_> = super::split_repo::load(conn)?.into_values().collect();
    let tabs = super::session_repo::load_session_tabs(conn)?;
    for layout in &mut split_layouts {
        layout.normalize(&tabs);
    }
    split_layouts.sort_by(|a, b| a.workspace_id.cmp(&b.workspace_id));
    Ok(BackupData {
        bookmarks: super::bookmarks_repo::get_bookmark_data(conn)?,
        workspaces: super::workspace_repo::list_workspaces(conn)?,
        sessions: super::saved_sessions_repo::list(conn)?,
        notes,
        site_search: super::site_search_repo::list(conn)?,
        tabs: tabs.iter().map(BackupTab::from).collect(),
        split_layouts,
    })
}
pub fn pending(conn: &Connection) -> Result<Option<PreferenceTask>, String> {
    let row: Option<(String, String, Option<String>)> = conn
        .query_row(
            "SELECT id,preferences,error FROM backup_preference_tasks ORDER BY rowid LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    row.map(|(id, json, error)| {
        Ok(PreferenceTask {
            id,
            preferences: serde_json::from_str(&json).map_err(|e| e.to_string())?,
            error,
        })
    })
    .transpose()
}
pub fn acknowledge(conn: &Connection, id: &str, error: Option<&str>) -> Result<(), String> {
    let changed = if let Some(error) = error {
        conn.execute(
            "UPDATE backup_preference_tasks SET error=?2 WHERE id=?1",
            params![id, error],
        )
    } else {
        conn.execute("DELETE FROM backup_preference_tasks WHERE id=?1", [id])
    }
    .map_err(|e| e.to_string())?;
    if changed != 1 {
        return Err("backup.preference_task_missing".into());
    }
    Ok(())
}

/// Data and the optional preference task commit together. The caller still owns the browser operation lock.
pub fn commit(
    conn: &Connection,
    current: &BackupData,
    merged: &BackupData,
    task: Option<&PreferenceTask>,
) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let workspace_ids: std::collections::HashSet<_> =
        current.workspaces.iter().map(|v| v.id.as_str()).collect();
    let folder_ids: std::collections::HashSet<_> =
        current.bookmarks.folders.iter().map(|v| v.id).collect();
    let bookmark_ids: std::collections::HashSet<_> =
        current.bookmarks.bookmarks.iter().map(|v| v.id).collect();
    let note_urls: std::collections::HashSet<_> =
        current.notes.iter().map(|v| v.url.as_str()).collect();
    let keywords: std::collections::HashSet<_> = current
        .site_search
        .iter()
        .map(|v| v.keyword.to_lowercase())
        .collect();
    let session_ids: std::collections::HashSet<_> = current.sessions.iter().map(|v| v.id).collect();
    for ws in &merged.workspaces {
        if !workspace_ids.contains(ws.id.as_str()) {
            super::workspace_repo::create_workspace(&tx, ws)?;
        }
    }
    for folder in &merged.bookmarks.folders {
        if !folder_ids.contains(&folder.id) {
            tx.execute(
                "INSERT INTO bookmark_folders(id,name,created_at) VALUES(?1,?2,unixepoch())",
                params![folder.id, folder.name],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    for item in &merged.bookmarks.bookmarks {
        if !bookmark_ids.contains(&item.id) {
            tx.execute("INSERT INTO bookmarks(id,title,url,icon,folder_id,created_at) VALUES(?1,?2,?3,?4,?5,unixepoch())",params![item.id,item.title,item.url,item.icon,item.folder_id]).map_err(|e|e.to_string())?;
        }
    }
    for note in &merged.notes {
        if !note_urls.contains(note.url.as_str()) {
            tx.execute(
                "INSERT INTO page_notes VALUES(?1,?2,?3,?4)",
                params![note.url, note.title, note.text, note.updated_at],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    for search in &merged.site_search {
        if !keywords.contains(&search.keyword.to_lowercase()) {
            tx.execute(
                "INSERT INTO site_search_shortcuts VALUES(?1,?2,?3)",
                params![search.keyword, search.name, search.template],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    for session in &merged.sessions {
        if !session_ids.contains(&session.id) {
            tx.execute(
                "INSERT INTO saved_sessions VALUES(?1,?2)",
                params![session.id, session.name],
            )
            .map_err(|e| e.to_string())?;
            for (position, page) in session.pages.iter().enumerate() {
                tx.execute(
                    "INSERT INTO saved_session_pages VALUES(?1,?2,?3,?4,?5,?6)",
                    params![
                        session.id,
                        position as i64,
                        page.title,
                        page.url,
                        page.is_pinned,
                        page.is_selected
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    super::session_repo::save_session_tabs(
        &tx,
        &merged.tabs.iter().map(BackupTab::tab).collect::<Vec<_>>(),
    )?;
    super::split_repo::save(
        &tx,
        &merged
            .split_layouts
            .iter()
            .cloned()
            .map(|l| (l.workspace_id.clone(), l))
            .collect(),
    )?;
    if let Some(task) = task {
        tx.execute(
            "INSERT INTO backup_preference_tasks(id,preferences) VALUES(?1,?2)",
            params![
                task.id,
                serde_json::to_string(&task.preferences).map_err(|e| e.to_string())?
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}
