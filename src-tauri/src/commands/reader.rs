use crate::window::reader::{extract_article, ReaderArticle};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_reader_article(
    app: AppHandle,
    tab_id: Option<String>,
) -> Result<ReaderArticle, String> {
    extract_article(&app, tab_id).await
}

