use crate::database::{
    command_search_repo::{self, CommandSearchResult},
    executor,
    saved_sessions_repo::{self, SavedSession},
};
use tauri::AppHandle;
#[tauri::command]
pub async fn search_command_items(
    app: AppHandle,
    query: String,
) -> Result<CommandSearchResult, String> {
    executor::run(app, move |conn| command_search_repo::search(conn, &query)).await
}
#[tauri::command]
pub async fn get_saved_session(app: AppHandle, id: i64) -> Result<SavedSession, String> {
    executor::run(app, move |conn| saved_sessions_repo::get(conn, id)).await
}
