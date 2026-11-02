use crate::database::{
    bookmarks_repo::{self, BookmarkData},
    executor,
    history_repo::{self, HistoryCursor, HistoryEntry},
};
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
pub async fn get_history(
    app: AppHandle,
    limit: Option<usize>,
    query: Option<String>,
    before: Option<HistoryCursor>,
) -> Result<Vec<HistoryEntry>, String> {
    flush_navigation(app.clone()).await?;
    executor::run(app, move |conn| {
        history_repo::search_history(
            conn,
            limit.unwrap_or(50).clamp(1, 200),
            &query.unwrap_or_default(),
            before.as_ref(),
        )
    })
    .await
}
#[tauri::command]
pub async fn clear_browsing_history(app: AppHandle) -> Result<(), String> {
    flush_navigation(app.clone()).await?;
    executor::run(app.clone(), |conn| {
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        history_repo::clear_all_history(&tx)?;
        crate::database::closed_tabs_repo::clear(&tx)?;
        tx.commit().map_err(|e| e.to_string())
    })
    .await?;
    let _ = app.emit_to(
        "main",
        "closed-tabs-updated",
        Vec::<crate::database::closed_tabs_repo::ClosedTab>::new(),
    );
    Ok(())
}
#[tauri::command]
pub async fn get_bookmark_data(app: AppHandle) -> Result<BookmarkData, String> {
    executor::run(app, bookmarks_repo::get_bookmark_data).await
}
#[tauri::command]
pub async fn save_bookmark(
    app: AppHandle,
    bookmark_id: Option<i64>,
    title: String,
    url: String,
    icon: Option<String>,
    folder_id: Option<i64>,
) -> Result<BookmarkData, String> {
    executor::run(app, move |conn| {
        bookmarks_repo::save_bookmark(
            conn,
            bookmark_id,
            &title,
            &url,
            icon.as_deref().unwrap_or("\u{2b50}"),
            folder_id,
        )
    })
    .await
}
#[tauri::command]
pub async fn remove_bookmark(app: AppHandle, bookmark_id: i64) -> Result<BookmarkData, String> {
    executor::run(app, move |conn| {
        bookmarks_repo::remove_bookmark(conn, bookmark_id)
    })
    .await
}
#[tauri::command]
pub async fn create_bookmark_folder(app: AppHandle, name: String) -> Result<BookmarkData, String> {
    executor::run(app, move |conn| bookmarks_repo::create_folder(conn, &name)).await
}
#[tauri::command]
pub async fn rename_bookmark_folder(
    app: AppHandle,
    folder_id: i64,
    name: String,
) -> Result<BookmarkData, String> {
    executor::run(app, move |conn| {
        bookmarks_repo::rename_folder(conn, folder_id, &name)
    })
    .await
}
#[tauri::command]
pub async fn delete_bookmark_folder(
    app: AppHandle,
    folder_id: i64,
) -> Result<BookmarkData, String> {
    executor::run(app, move |conn| {
        bookmarks_repo::delete_folder(conn, folder_id)
    })
    .await
}

async fn flush_navigation(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<crate::database::navigation_writer::NavigationWriter>()
            .flush()
    })
    .await
    .map_err(|e| e.to_string())?
}
