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

    Ok(app_data_dir.join("veloce_browser.db"))
}

/// Opens the SQLite connection and ensures all required tables exist.
pub fn init_database(app: &AppHandle) -> Result<Connection, String> {
    let db_path = resolve_db_path(app)?;
    let conn =
        Connection::open(db_path).map_err(|e| format!("Failed to open SQLite database: {}", e))?;

    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;

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

    crate::database::migrations::apply(&conn, "bookmark-folders-column-v1", |conn| {
        if !crate::database::migrations::has_column(conn, "bookmarks", "folder_id")? {
            conn.execute_batch("ALTER TABLE bookmarks ADD COLUMN folder_id INTEGER REFERENCES bookmark_folders(id) ON DELETE SET NULL").map_err(|e| e.to_string())?;
        }
        Ok(())
    })?;
    crate::database::migrations::upgrade_session(&conn)?;
    crate::database::migrations::apply(&conn, "chronological-indexes-v1", |conn| {
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_history_recent ON history(visited_at DESC, id DESC);
         CREATE INDEX IF NOT EXISTS idx_downloads_recent ON downloads(created_at DESC, id DESC);
         CREATE INDEX IF NOT EXISTS idx_tab_usage_time ON tab_usage_events(activated_at);",
        )
        .map_err(|e| e.to_string())
    })?;

    crate::database::bookmarks_repo::migrate_default_bookmarks_once(&conn)?;
    crate::database::bookmarks_repo::migrate_trailing_slash_default_bookmarks_once(&conn)?;
    crate::database::workspace_repo::initialize_workspace_table(&conn)?;

    crate::database::saved_sessions_repo::migrate(&conn)?;
    Ok(conn)
}
