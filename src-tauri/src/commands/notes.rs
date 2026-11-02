use crate::database::{
    executor,
    notes_repo::{self as repo, NoteSummary, PageNote},
};
use tauri::AppHandle;
#[tauri::command]
pub async fn get_page_note(app: AppHandle, url: String) -> Result<Option<PageNote>, String> {
    executor::run(app, move |conn| repo::get(conn, &url)).await
}
#[tauri::command]
pub async fn search_page_notes(app: AppHandle, query: String) -> Result<Vec<NoteSummary>, String> {
    executor::run(app, move |conn| repo::search(conn, &query)).await
}
#[tauri::command]
pub async fn save_page_note(
    app: AppHandle,
    url: String,
    title: String,
    text: String,
) -> Result<(), String> {
    executor::run(app, move |conn| repo::save(conn, &url, &title, &text)).await
}
#[tauri::command]
pub async fn delete_page_note(app: AppHandle, url: String) -> Result<(), String> {
    executor::run(app, move |conn| repo::delete(conn, &url)).await
}
