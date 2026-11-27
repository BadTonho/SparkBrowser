use crate::database::{
    bookmarks_repo::{self, BookmarkData},
    executor,
    history_repo::{self, HistoryCursor, HistoryEntry},
};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn get_history(
    app: AppHandle,
    limit: Option<usize>,
    query: Option<String>,
    before: Option<HistoryCursor>,
    grouped: Option<bool>,
) -> Result<Vec<HistoryEntry>, String> {
    flush_navigation(app.clone()).await?;
    executor::run(app, move |conn| {
        let limit = limit.unwrap_or(50).clamp(1, 200);
        let query = query.unwrap_or_default();
        // Suggestion lists want one row per address with its visit count; it has no pages.
        if grouped.unwrap_or(false) {
            return history_repo::suggest_history(conn, limit, &query);
        }
        history_repo::search_history(conn, limit, &query, before.as_ref())
    })
    .await
}
#[tauri::command]
/// Clears the browsing history. With `older_than_days` only what is older than that many days goes.
pub async fn clear_browsing_history(
    app: AppHandle,
    older_than_days: Option<u32>,
) -> Result<(), String> {
    flush_navigation(app.clone()).await?;
    let state = app.state::<crate::state::browser_state::BrowserState>();
    let _operation = state.session().operation().await;
    let clear_app = app.clone();
    executor::run(app.clone(), move |conn| {
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        match older_than_days {
            Some(days) => {
                history_repo::clear_history_older_than(&tx, history_repo::days_ago(days))?
            }
            None => history_repo::clear_history_data(&tx)?,
        }
        tx.commit().map_err(|e| e.to_string())?;
        clear_app
            .state::<crate::window::new_tab::NewTabService>()
            .history_epoch
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    })
    .await?;
    crate::commands::closed_tabs::notify(&app, &state);
    crate::window::new_tab::invalidate(&app);
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

pub(crate) async fn flush_navigation(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<crate::database::navigation_writer::NavigationWriter>()
            .flush()
    })
    .await
    .map_err(|e| e.to_string())?
}
