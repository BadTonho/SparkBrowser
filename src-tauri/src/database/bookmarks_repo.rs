use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkItem {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub icon: String,
    pub folder_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkFolder {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkData {
    pub bookmarks: Vec<BookmarkItem>,
    pub folders: Vec<BookmarkFolder>,
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn migrate_default_bookmarks_once(conn: &Connection) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to start bookmark migration: {e}"))?;
    let migrated: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM app_migrations WHERE name = 'remove_default_bookmarks_v1')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if migrated {
        return Ok(());
    }

    // These exact URLs were reserved by the original seeded bookmarks because
    // bookmarks had a UNIQUE URL constraint and no title/icon editor.
    let defaults = [
        ("DuckDuckGo", "https://duckduckgo.com", "\u{1F986}"),
        ("GitHub", "https://github.com", "\u{1F419}"),
        ("Rust Lang", "https://www.rust-lang.org", "\u{1F980}"),
        ("Wikipedia", "https://www.wikipedia.org", "\u{1F310}"),
    ];
    for (title, url, icon) in defaults {
        tx.execute(
            "DELETE FROM bookmarks WHERE title = ?1 AND url = ?2 AND icon = ?3",
            params![title, url, icon],
        )
        .map_err(|e| format!("Failed to remove a default bookmark: {e}"))?;
    }
    tx.execute(
        "INSERT INTO app_migrations (name) VALUES ('remove_default_bookmarks_v1')",
        [],
    )
    .map_err(|e| format!("Failed to record default bookmark migration: {e}"))?;
    tx.commit()
        .map_err(|e| format!("Failed to finish bookmark migration: {e}"))?;
    Ok(())
}

/// Removes legacy seeded bookmarks that used a trailing slash in their URL.
/// The title and icon must also match the original seed so edited/custom items
/// are left untouched, even when they point at one of the same sites.
pub fn migrate_trailing_slash_default_bookmarks_once(conn: &Connection) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to start trailing-slash bookmark migration: {e}"))?;
    let migrated: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM app_migrations WHERE name = 'remove_default_bookmarks_trailing_slash_v1')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if migrated {
        return Ok(());
    }

    let defaults = [
        ("DuckDuckGo", "https://duckduckgo.com/", "\u{1F986}"),
        ("GitHub", "https://github.com/", "\u{1F419}"),
        ("Rust Lang", "https://www.rust-lang.org/", "\u{1F980}"),
        ("Wikipedia", "https://www.wikipedia.org/", "\u{1F310}"),
    ];
    for (title, url, icon) in defaults {
        tx.execute(
            "DELETE FROM bookmarks WHERE title = ?1 AND url = ?2 AND icon = ?3",
            params![title, url, icon],
        )
        .map_err(|e| format!("Failed to remove a trailing-slash default bookmark: {e}"))?;
    }
    tx.execute(
        "INSERT INTO app_migrations (name) VALUES ('remove_default_bookmarks_trailing_slash_v1')",
        [],
    )
    .map_err(|e| format!("Failed to record trailing-slash bookmark migration: {e}"))?;
    tx.commit()
        .map_err(|e| format!("Failed to finish trailing-slash bookmark migration: {e}"))?;
    Ok(())
}

pub fn list_bookmarks(conn: &Connection) -> Result<Vec<BookmarkItem>, String> {
    let mut stmt = conn
        .prepare("SELECT id, title, url, icon, folder_id FROM bookmarks ORDER BY id ASC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BookmarkItem {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                icon: row.get(3)?,
                folder_id: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn list_folders(conn: &Connection) -> Result<Vec<BookmarkFolder>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM bookmark_folders ORDER BY id ASC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BookmarkFolder {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_bookmark_data(conn: &Connection) -> Result<BookmarkData, String> {
    Ok(BookmarkData {
        bookmarks: list_bookmarks(conn)?,
        folders: list_folders(conn)?,
    })
}

fn normalize_folder_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Folder name cannot be empty".to_string());
    }
    if name.chars().count() > 64 {
        return Err("Folder name must be 64 characters or fewer".to_string());
    }
    Ok(name.to_string())
}

fn validate_unique_folder_name(
    conn: &Connection,
    name: &str,
    except_folder_id: Option<i64>,
) -> Result<(), String> {
    let normalized_name = name.to_lowercase();
    let exists = list_folders(conn)?.iter().any(|folder| {
        Some(folder.id) != except_folder_id && folder.name.to_lowercase() == normalized_name
    });
    if exists {
        return Err("A folder with this name already exists".to_string());
    }
    Ok(())
}

fn validate_folder(conn: &Connection, folder_id: Option<i64>) -> Result<(), String> {
    if let Some(folder_id) = folder_id {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM bookmark_folders WHERE id = ?1)",
                [folder_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if !exists {
            return Err("Bookmark folder does not exist".to_string());
        }
    }
    Ok(())
}

pub fn save_bookmark(
    conn: &Connection,
    bookmark_id: Option<i64>,
    title: &str,
    url: &str,
    icon: &str,
    folder_id: Option<i64>,
) -> Result<BookmarkData, String> {
    let title = title.trim();
    let url = url.trim();
    if title.is_empty() || url.is_empty() {
        return Err("Bookmark title and URL are required".to_string());
    }
    validate_folder(conn, folder_id)?;

    if let Some(bookmark_id) = bookmark_id {
        let changed = conn
            .execute(
                "UPDATE bookmarks SET title = ?1, url = ?2, icon = ?3, folder_id = ?4 WHERE id = ?5",
                params![title, url, icon, folder_id, bookmark_id],
            )
            .map_err(|e| format!("Failed to update bookmark: {e}"))?;
        if changed == 0 {
            return Err("Bookmark not found".to_string());
        }
    } else {
        let now = current_unix_timestamp();
        conn.execute(
            "INSERT INTO bookmarks (title, url, icon, created_at, folder_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![title, url, icon, now, folder_id],
        )
        .map_err(|e| format!("Failed to save bookmark: {e}"))?;
    }

    get_bookmark_data(conn)
}

pub fn remove_bookmark(conn: &Connection, bookmark_id: i64) -> Result<BookmarkData, String> {
    conn.execute("DELETE FROM bookmarks WHERE id = ?1", [bookmark_id])
        .map_err(|e| format!("Failed to remove bookmark: {e}"))?;
    get_bookmark_data(conn)
}

pub fn create_folder(conn: &Connection, name: &str) -> Result<BookmarkData, String> {
    let name = normalize_folder_name(name)?;
    validate_unique_folder_name(conn, &name, None)?;
    conn.execute(
        "INSERT INTO bookmark_folders (name, created_at) VALUES (?1, ?2)",
        params![name, current_unix_timestamp()],
    )
    .map_err(|e| format!("Failed to create bookmark folder: {e}"))?;
    get_bookmark_data(conn)
}

pub fn rename_folder(
    conn: &Connection,
    folder_id: i64,
    name: &str,
) -> Result<BookmarkData, String> {
    let name = normalize_folder_name(name)?;
    validate_unique_folder_name(conn, &name, Some(folder_id))?;
    let changed = conn
        .execute(
            "UPDATE bookmark_folders SET name = ?1 WHERE id = ?2",
            params![name, folder_id],
        )
        .map_err(|e| format!("Failed to rename bookmark folder: {e}"))?;
    if changed == 0 {
        return Err("Bookmark folder not found".to_string());
    }
    get_bookmark_data(conn)
}

pub fn delete_folder(conn: &Connection, folder_id: i64) -> Result<BookmarkData, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to start folder deletion: {e}"))?;
    tx.execute(
        "UPDATE bookmarks SET folder_id = NULL WHERE folder_id = ?1",
        [folder_id],
    )
    .map_err(|e| format!("Failed to move folder bookmarks to root: {e}"))?;
    let deleted = tx
        .execute("DELETE FROM bookmark_folders WHERE id = ?1", [folder_id])
        .map_err(|e| format!("Failed to delete bookmark folder: {e}"))?;
    if deleted == 0 {
        return Err("Bookmark folder not found".to_string());
    }
    tx.commit()
        .map_err(|e| format!("Failed to finish folder deletion: {e}"))?;
    get_bookmark_data(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                icon TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                folder_id INTEGER
            );
            CREATE TABLE bookmark_folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL COLLATE NOCASE UNIQUE,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE app_migrations (name TEXT PRIMARY KEY);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn removes_only_original_seeded_bookmarks_once() {
        let conn = database();
        conn.execute_batch(
            "INSERT INTO bookmarks (title, url, icon, created_at) VALUES
                ('DuckDuckGo', 'https://duckduckgo.com', '🦆', 1),
                ('GitHub', 'https://github.com', '🐙', 1),
                ('Rust Lang', 'https://www.rust-lang.org', '🦀', 1),
                ('Wikipedia', 'https://www.wikipedia.org', '🌐', 1),
                ('My GitHub', 'https://github.com/custom', '⭐', 1),
                ('Custom DuckDuckGo', 'https://duckduckgo.com/about', '🔖', 1);",
        )
        .unwrap();

        migrate_default_bookmarks_once(&conn).unwrap();
        migrate_default_bookmarks_once(&conn).unwrap();

        let items = list_bookmarks(&conn).unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|item| item.title == "My GitHub"));
        assert!(items.iter().any(|item| item.title == "Custom DuckDuckGo"));
    }

    #[test]
    fn deleting_folder_moves_bookmarks_to_root() {
        let conn = database();
        let data = create_folder(&conn, "Reading").unwrap();
        let folder_id = data.folders[0].id;
        save_bookmark(
            &conn,
            None,
            "Example",
            "https://example.com",
            "⭐",
            Some(folder_id),
        )
        .unwrap();

        let data = delete_folder(&conn, folder_id).unwrap();
        assert!(data.folders.is_empty());
        assert_eq!(data.bookmarks.len(), 1);
        assert_eq!(data.bookmarks[0].folder_id, None);
    }

    #[test]
    fn folder_names_are_case_insensitively_unique_and_nonempty() {
        let conn = database();
        create_folder(&conn, "Reading").unwrap();
        assert!(create_folder(&conn, " reading ").is_err());
        assert!(create_folder(&conn, "   ").is_err());
        create_folder(&conn, "Éstudo").unwrap();
        assert!(create_folder(&conn, "éstudo").is_err());
    }
}
