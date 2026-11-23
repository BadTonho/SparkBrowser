use crate::database::{
    executor,
    tab_dividers_repo::{self as repo, TabDivider},
};
use tauri::AppHandle;
#[tauri::command]
pub async fn get_tab_dividers(app: AppHandle) -> Result<Vec<TabDivider>, String> {
    executor::run(app, repo::load).await
}
#[tauri::command]
pub async fn set_tab_dividers(app: AppHandle, dividers: Vec<TabDivider>) -> Result<(), String> {
    executor::run(app, move |conn| repo::save(conn, &dividers)).await
}
