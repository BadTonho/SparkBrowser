use crate::{
    database::{
        command_search_repo::{self, CommandSearchResult},
        saved_sessions_repo::{self, SavedSession},
    },
    state::browser_state::BrowserState,
};
use tauri::State;
#[tauri::command]
pub async fn search_command_items(
    state: State<'_, BrowserState>,
    query: String,
) -> Result<CommandSearchResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    command_search_repo::search(
        db.as_ref().ok_or("command_search.database_unavailable")?,
        &query,
    )
}
#[tauri::command]
pub async fn get_saved_session(
    state: State<'_, BrowserState>,
    id: i64,
) -> Result<SavedSession, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    saved_sessions_repo::get(
        db.as_ref().ok_or("command_search.database_unavailable")?,
        id,
    )
}
