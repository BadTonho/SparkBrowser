use crate::database::executor;
use crate::feeds::{self, Entry, Feed};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_feeds(app: AppHandle) -> Result<Vec<Feed>, String> {
    executor::run(app, feeds::list).await
}
#[tauri::command]
pub async fn get_feed_entries(app: AppHandle, id: i64) -> Result<Vec<Entry>, String> {
    executor::run(app, move |db| feeds::entries(db, id)).await
}
#[tauri::command]
pub async fn save_feed(
    app: AppHandle,
    id: Option<i64>,
    title: String,
    url: String,
    interval_minutes: u32,
) -> Result<(), String> {
    executor::run(app, move |db| {
        feeds::save(db, id, &title, &url, interval_minutes)
    })
    .await
}
#[tauri::command]
pub async fn delete_feed(app: AppHandle, id: i64) -> Result<(), String> {
    executor::run(app, move |db| feeds::remove(db, id)).await
}
#[tauri::command]
pub async fn refresh_feed(app: AppHandle, id: i64) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || feeds::refresh(&app, id))
        .await
        .map_err(|e| e.to_string())?
}
