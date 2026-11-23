use crate::database::{
    executor,
    tab_groups_repo::{self as repo, TabGroups},
};
use tauri::AppHandle;
#[tauri::command]
pub async fn get_tab_groups(app: AppHandle) -> Result<TabGroups, String> {
    executor::run(app, repo::load).await
}
#[tauri::command]
pub async fn set_tab_groups(app: AppHandle, groups: TabGroups) -> Result<(), String> {
    executor::run(app, move |conn| repo::save(conn, &groups)).await
}
