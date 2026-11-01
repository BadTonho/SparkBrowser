use crate::database::bookmarks_repo::{list_bookmarks, toggle_bookmark as repo_toggle_bookmark, BookmarkItem};
use crate::database::history_repo::{clear_all_history, get_recent_history, HistoryEntry};
use crate::state::browser_state::BrowserState;
use tauri::State;

#[tauri::command]
pub fn get_history(
    state: State<'_, BrowserState>,
    limit: Option<usize>,
) -> Result<Vec<HistoryEntry>, String> {
    let max_items = limit.unwrap_or(50);
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return get_recent_history(conn, max_items);
    }
    Ok(Vec::new())
}

#[tauri::command]
pub fn clear_browsing_history(state: State<'_, BrowserState>) -> Result<(), String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        clear_all_history(conn)?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_bookmarks(state: State<'_, BrowserState>) -> Result<Vec<BookmarkItem>, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return list_bookmarks(conn);
    }
    Ok(Vec::new())
}

#[tauri::command]
pub fn toggle_bookmark(
    state: State<'_, BrowserState>,
    title: String,
    url: String,
    icon: Option<String>,
) -> Result<Vec<BookmarkItem>, String> {
    let glyph = icon.unwrap_or_else(|| "⭐".to_string());
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_ref() {
        return repo_toggle_bookmark(conn, &title, &url, &glyph);
    }
    Ok(Vec::new())
}

