//! Lets the browser shell report script errors so they reach the same log and notification as
//! native ones.
use crate::error_log::Level;

/// Inputs are cut before any processing so a script cannot make the report expensive.
const MAX_INPUT: usize = 8_000;

/// What the settings screen shows: the log file's path and its latest entries, newest first.
#[derive(serde::Serialize)]
pub struct RecentErrors {
    path: String,
    entries: Vec<crate::error_log::LogEntry>,
}

/// Entries returned to the settings screen at most.
const MAX_ENTRIES: usize = 150;

#[tauri::command]
pub fn get_recent_errors() -> RecentErrors {
    let (path, entries) = crate::error_log::read_recent(MAX_ENTRIES);
    RecentErrors { path, entries }
}

/// `level` is `"warning"` for a failure the interface ignored (logged only); anything else is an
/// `Error`, which also shows the notification. The interface cannot send `Info` or `Critical`.
#[tauri::command]
pub fn report_frontend_error(summary: String, details: String, level: Option<String>) {
    let cut = |text: String| text.chars().take(MAX_INPUT).collect::<String>();
    let level = if level.as_deref() == Some("warning") {
        Level::Warning
    } else {
        Level::Error
    };
    crate::error_log::report(level, "interface", &cut(summary), &cut(details));
}
