use crate::{
    backup::{
        bookmark_export::{self, Format},
        files,
    },
    database::bookmarks_repo,
    state::browser_state::BrowserState,
};
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize)]
pub struct BookmarkExportResult {
    pub count: usize,
    /// Bookmarks left out because their address is not plain http(s).
    pub skipped: usize,
}

/// Writes every bookmark to a file other browsers can import. `None` means the dialog was cancelled.
#[tauri::command]
pub async fn export_bookmarks(
    app: AppHandle,
    state: State<'_, BrowserState>,
    format: Format,
) -> Result<Option<BookmarkExportResult>, String> {
    let exported = {
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("backup.database_unavailable")?;
        bookmark_export::render(format, &bookmarks_repo::get_bookmark_data(conn)?)
    };
    if exported.count == 0 {
        return Err("bookmarks.nothing_to_export".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let Some(path) = app
            .dialog()
            .file()
            .add_filter(
                match format {
                    Format::Html => "HTML bookmarks",
                    Format::Csv => "CSV",
                },
                format.extensions(),
            )
            .set_file_name(format.default_name())
            .blocking_save_file()
        else {
            return Ok(None);
        };
        let path = path.into_path().map_err(|e| e.to_string())?;
        let name = path.to_string_lossy().to_lowercase();
        // The path is never adjusted after the user confirmed it, so a wrong name is refused.
        if !format
            .extensions()
            .iter()
            .any(|extension| name.ends_with(&format!(".{extension}")))
        {
            return Err("bookmarks.invalid_extension".into());
        }
        files::write_bytes(&path, &exported.bytes)?;
        Ok(Some(BookmarkExportResult {
            count: exported.count,
            skipped: exported.skipped,
        }))
    })
    .await
    .map_err(|e| e.to_string())?
}
