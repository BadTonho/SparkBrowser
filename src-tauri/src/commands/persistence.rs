use crate::database::bookmarks_repo::{self, BookmarkData};
use crate::database::history_repo::{
    clear_all_history, search_history, HistoryCursor, HistoryEntry,
};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn get_history(
    app: AppHandle,
    state: State<'_, BrowserState>,
    limit: Option<usize>,
    query: Option<String>,
    before: Option<HistoryCursor>,
) -> Result<Vec<HistoryEntry>, String> {
    flush_navigation(app).await?;
    let max_items = limit.unwrap_or(50).clamp(1, 200);
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return search_history(conn, max_items, &query.unwrap_or_default(), before.as_ref());
    }
    Ok(Vec::new())
}

#[tauri::command]
pub async fn clear_browsing_history(
    app: AppHandle,
    state: State<'_, BrowserState>,
) -> Result<(), String> {
    flush_navigation(app).await?;
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        clear_all_history(conn)?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_bookmark_data(state: State<'_, BrowserState>) -> Result<BookmarkData, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return bookmarks_repo::get_bookmark_data(conn);
    }
    Ok(BookmarkData {
        bookmarks: Vec::new(),
        folders: Vec::new(),
    })
}

#[tauri::command]
pub fn save_bookmark(
    state: State<'_, BrowserState>,
    bookmark_id: Option<i64>,
    title: String,
    url: String,
    icon: Option<String>,
    folder_id: Option<i64>,
) -> Result<BookmarkData, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return bookmarks_repo::save_bookmark(
            conn,
            bookmark_id,
            &title,
            &url,
            icon.as_deref().unwrap_or("⭐"),
            folder_id,
        );
    }
    Ok(BookmarkData {
        bookmarks: Vec::new(),
        folders: Vec::new(),
    })
}

#[tauri::command]
pub fn remove_bookmark(
    state: State<'_, BrowserState>,
    bookmark_id: i64,
) -> Result<BookmarkData, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return bookmarks_repo::remove_bookmark(conn, bookmark_id);
    }
    Ok(BookmarkData {
        bookmarks: Vec::new(),
        folders: Vec::new(),
    })
}

#[tauri::command]
pub fn create_bookmark_folder(
    state: State<'_, BrowserState>,
    name: String,
) -> Result<BookmarkData, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return bookmarks_repo::create_folder(conn, &name);
    }
    Ok(BookmarkData {
        bookmarks: Vec::new(),
        folders: Vec::new(),
    })
}

#[tauri::command]
pub fn rename_bookmark_folder(
    state: State<'_, BrowserState>,
    folder_id: i64,
    name: String,
) -> Result<BookmarkData, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return bookmarks_repo::rename_folder(conn, folder_id, &name);
    }
    Ok(BookmarkData {
        bookmarks: Vec::new(),
        folders: Vec::new(),
    })
}

#[tauri::command]
pub fn delete_bookmark_folder(
    state: State<'_, BrowserState>,
    folder_id: i64,
) -> Result<BookmarkData, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return bookmarks_repo::delete_folder(conn, folder_id);
    }
    Ok(BookmarkData {
        bookmarks: Vec::new(),
        folders: Vec::new(),
    })
}

async fn flush_navigation(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<crate::database::navigation_writer::NavigationWriter>()
            .flush()
    })
    .await
    .map_err(|e| e.to_string())?
}
