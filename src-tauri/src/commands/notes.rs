use crate::{
    database::notes_repo::{self as repo, PageNote},
    state::browser_state::BrowserState,
};
use tauri::State;
#[tauri::command]
pub async fn get_page_note(
    state: State<'_, BrowserState>,
    url: String,
) -> Result<Option<PageNote>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    repo::get(db.as_ref().ok_or("Database is not ready")?, &url)
}
#[tauri::command]
pub async fn search_page_notes(
    state: State<'_, BrowserState>,
    query: String,
) -> Result<Vec<PageNote>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    repo::search(db.as_ref().ok_or("Database is not ready")?, &query)
}
#[tauri::command]
pub async fn save_page_note(
    state: State<'_, BrowserState>,
    url: String,
    title: String,
    text: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    repo::save(
        db.as_ref().ok_or("Database is not ready")?,
        &url,
        &title,
        &text,
    )
}
#[tauri::command]
pub async fn delete_page_note(state: State<'_, BrowserState>, url: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    repo::delete(db.as_ref().ok_or("Database is not ready")?, &url)
}
