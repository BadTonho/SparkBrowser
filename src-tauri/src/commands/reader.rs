use crate::window::reader::{extract_article, ReaderArticle};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_reader_article(
    app: AppHandle,
    webview: tauri::Webview,
    tab_id: Option<String>,
) -> Result<ReaderArticle, String> {
    extract_article(&app, &super::access::caller_window(&webview), tab_id).await
}
