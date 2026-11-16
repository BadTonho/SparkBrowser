//! Lets the browser shell report script errors so they reach the same log and notification as
//! native ones.
use crate::error_log::Level;

/// Inputs are cut before any processing so a script cannot make the report expensive.
const MAX_INPUT: usize = 8_000;

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
