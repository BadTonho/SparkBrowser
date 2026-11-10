use crate::{
    database::site_search_repo::{self as repo, SiteSearchShortcut},
    state::browser_state::BrowserState,
};
use tauri::State;
#[tauri::command]
pub async fn get_site_search_shortcuts(
    state: State<'_, BrowserState>,
) -> Result<Vec<SiteSearchShortcut>, String> {
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    repo::list(db.as_ref().ok_or("Database is not ready")?)
}
#[tauri::command]
pub async fn save_site_search_shortcut(
    state: State<'_, BrowserState>,
    previous: Option<String>,
    item: SiteSearchShortcut,
) -> Result<(), String> {
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    repo::save(
        db.as_ref().ok_or("Database is not ready")?,
        previous.as_deref(),
        &item,
    )
}
#[tauri::command]
pub async fn delete_site_search_shortcut(
    state: State<'_, BrowserState>,
    keyword: String,
) -> Result<(), String> {
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    db.as_ref()
        .ok_or("Database is not ready")?
        .execute(
            "DELETE FROM site_search_shortcuts WHERE keyword=?1",
            [keyword],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
