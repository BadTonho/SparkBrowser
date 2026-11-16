//! Lets the browser shell report script errors so they reach the same log and notification as
//! native ones.

/// Inputs are cut before any processing so a script cannot make the report expensive.
const MAX_INPUT: usize = 8_000;

#[tauri::command]
pub fn report_frontend_error(summary: String, details: String) {
    let cut = |text: String| text.chars().take(MAX_INPUT).collect::<String>();
    crate::error_log::report(
        crate::error_log::Level::Error,
        "interface",
        &cut(summary),
        &cut(details),
    );
}
