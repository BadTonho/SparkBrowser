use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Resolves the SQLite database path inside the OS-specific application data directory
/// so user databases are never created inside the source repository.
pub fn resolve_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve OS app_data_dir: {}", e))?;

    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app_data_dir: {}", e))?;
    }

    crate::legacy::rename_database_files(&app_data_dir);
    Ok(app_data_dir.join("spark_browser.db"))
}

/// Opens the SQLite connection and ensures all required tables exist.
pub fn init_database(app: &AppHandle) -> Result<Connection, String> {
    let db_path = resolve_db_path(app)?;
    let conn =
        Connection::open(db_path).map_err(|e| format!("Failed to open SQLite database: {}", e))?;

    initialize_schema(&conn)?;
    Ok(conn)
}

pub fn initialize_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA temp_store = MEMORY;
        PRAGMA cache_size = -8000;
        PRAGMA mmap_size = 268435456;

        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            url TEXT NOT NULL,
            visited_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS bookmarks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            url TEXT NOT NULL UNIQUE,
            icon TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS bookmark_folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL COLLATE NOCASE UNIQUE,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_migrations (
            name TEXT PRIMARY KEY
        );

        CREATE TABLE IF NOT EXISTS session_tabs (
            tab_id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            url TEXT NOT NULL,
            workspace_id TEXT NOT NULL DEFAULT 'ws-personal',
            is_active INTEGER NOT NULL,
            is_hibernated INTEGER NOT NULL,
            position INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS downloads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_name TEXT NOT NULL,
            url TEXT NOT NULL,
            save_path TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );


        CREATE TABLE IF NOT EXISTS tab_usage_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tab_id TEXT NOT NULL,
            activated_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_tab_usage_events_tab_time
            ON tab_usage_events(tab_id, activated_at);
        ",
    )
    .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

    crate::database::migrations::apply(conn, "bookmark-folders-column-v1", |conn| {
        if !crate::database::migrations::has_column(conn, "bookmarks", "folder_id")? {
            conn.execute_batch("ALTER TABLE bookmarks ADD COLUMN folder_id INTEGER REFERENCES bookmark_folders(id) ON DELETE SET NULL").map_err(|e| e.to_string())?;
        }
        Ok(())
    })?;
    crate::database::migrations::upgrade_session(conn)?;
    crate::database::account_containers_repo::migrate(conn)?;
    crate::database::migrations::apply(conn, "chronological-indexes-v1", |conn| {
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_history_recent ON history(visited_at DESC, id DESC);
         CREATE INDEX IF NOT EXISTS idx_downloads_recent ON downloads(created_at DESC, id DESC);
         CREATE INDEX IF NOT EXISTS idx_tab_usage_time ON tab_usage_events(activated_at);",
        )
        .map_err(|e| e.to_string())
    })?;

    crate::database::bookmarks_repo::migrate_default_bookmarks_once(conn)?;
    crate::database::bookmarks_repo::migrate_trailing_slash_default_bookmarks_once(conn)?;
    crate::database::workspace_repo::initialize_workspace_table(conn)?;

    crate::database::saved_sessions_repo::migrate(conn)?;
    crate::database::site_search_repo::migrate(conn)?;
    crate::database::notes_repo::migrate(conn)?;
    crate::feeds::migrate(conn)?;
    crate::database::split_repo::migrate(conn)?;
    crate::database::backup_repo::migrate(conn)?;
    crate::database::closed_tabs_repo::migrate(conn)?;
    crate::database::recovery_repo::migrate(conn)?;
    crate::database::command_search_repo::register(conn)?;
    // After the search functions exist: updating saved session pages fires index triggers that call them.
    migrate_legacy_new_tab_url(conn)?;
    crate::database::site_zoom_repo::migrate(conn)?;
    crate::database::site_icons_repo::migrate(conn)?;
    crate::database::new_tab_repo::migrate(conn)?;
    crate::database::download_jobs_repo::migrate(conn)?;
    crate::database::extensions_repo::migrate(conn)?;
    Ok(())
}

/// Saved tabs, sessions and recently closed tabs that still hold the pre-rename new tab address.
fn migrate_legacy_new_tab_url(conn: &Connection) -> Result<(), String> {
    crate::database::migrations::apply(conn, "spark-rename-new-tab-url-v1", |conn| {
        for (table, column) in [
            ("session_tabs", "url"),
            ("saved_session_pages", "url"),
            ("closed_tabs", "url"),
        ] {
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
                    [table],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            if exists {
                conn.execute(
                    &format!("UPDATE {table} SET {column}=?1 WHERE {column}=?2"),
                    [
                        crate::window::new_tab::URL,
                        crate::window::new_tab::LEGACY_URL,
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod rename_tests {
    use super::*;

    #[test]
    fn the_old_new_tab_address_is_rewritten_in_saved_data_exactly_once() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();
        conn.execute_batch(
            "INSERT INTO session_tabs(tab_id,title,url,workspace_id,is_active,is_hibernated,position,is_pinned) VALUES('a','New tab','veloce://newtab','ws-personal',0,1,0,0);
             INSERT INTO session_tabs(tab_id,title,url,workspace_id,is_active,is_hibernated,position,is_pinned) VALUES('b','Site','https://example.test/veloce://newtab','ws-personal',0,1,1,0);
             INSERT INTO closed_tabs(url,title,workspace_id,workspace_name,position,is_pinned,closed_at) VALUES('veloce://newtab','t','ws-personal','Personal',0,0,1);
             DELETE FROM app_migrations WHERE name='spark-rename-new-tab-url-v1';",
        )
        .unwrap();
        migrate_legacy_new_tab_url(&conn).unwrap();
        let url = |id: &str| -> String {
            conn.query_row("SELECT url FROM session_tabs WHERE tab_id=?1", [id], |r| {
                r.get(0)
            })
            .unwrap()
        };
        assert_eq!(url("a"), "spark://newtab");
        assert_eq!(
            url("b"),
            "https://example.test/veloce://newtab",
            "only the exact address changes"
        );
        let closed: String = conn
            .query_row("SELECT url FROM closed_tabs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(closed, "spark://newtab");
        // The marker makes a second call a no-op, even if the old address reappears.
        conn.execute(
            "UPDATE session_tabs SET url='veloce://newtab' WHERE tab_id='a'",
            [],
        )
        .unwrap();
        migrate_legacy_new_tab_url(&conn).unwrap();
        assert_eq!(url("a"), "veloce://newtab");
    }

    #[test]
    fn opening_an_old_database_file_rewrites_the_address_even_though_index_triggers_fire() {
        // Regression: the rewrite used to run before the search functions were registered, so a
        // saved session page made the old trigger fail with "no such function: veloce_search_fold".
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("spark_browser.db");
        {
            let conn = Connection::open(&path).unwrap();
            initialize_schema(&conn).unwrap();
            conn.execute_batch(
                "INSERT INTO saved_sessions(id,name) VALUES(1,'Work');
                 INSERT INTO saved_session_pages(session_id,position,title,url,is_pinned,is_selected) VALUES(1,0,'New tab','veloce://newtab',0,0);
                 DROP TRIGGER session_page_search_update;
                 CREATE TRIGGER session_page_search_update AFTER UPDATE OF title,url ON saved_session_pages BEGIN
                   UPDATE session_page_search SET title=veloce_search_fold(new.title), address=veloce_search_fold(new.url) WHERE rowid=old.rowid;
                 END;
                 DELETE FROM app_migrations WHERE name='spark-rename-new-tab-url-v1';",
            )
            .unwrap();
        }
        // A fresh connection starts with no functions registered, exactly like an app launch.
        let conn = Connection::open(&path).unwrap();
        initialize_schema(&conn).unwrap();
        let url: String = conn
            .query_row("SELECT url FROM saved_session_pages", [], |r| r.get(0))
            .unwrap();
        assert_eq!(url, "spark://newtab");
    }

    #[test]
    fn databases_created_before_the_rename_keep_working_through_the_old_function_name() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();
        // A trigger as an old database stores it: it calls the pre-rename function.
        conn.execute_batch(
            "CREATE TABLE legacy_probe(text TEXT, folded TEXT);
             CREATE TRIGGER legacy_probe_fold AFTER INSERT ON legacy_probe BEGIN
               UPDATE legacy_probe SET folded = veloce_search_fold(new.text) WHERE rowid = new.rowid;
             END;
             INSERT INTO legacy_probe(text) VALUES('Café Spark');",
        )
        .unwrap();
        let folded: String = conn
            .query_row("SELECT folded FROM legacy_probe", [], |r| r.get(0))
            .unwrap();
        assert_eq!(folded, "cafe spark");
        let current: String = conn
            .query_row("SELECT spark_search_fold('Café')", [], |r| r.get(0))
            .unwrap();
        assert_eq!(current, "cafe");
    }
}
