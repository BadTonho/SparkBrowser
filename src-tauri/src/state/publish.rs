//! Tells every window about the tabs it shows.
use super::browser_state::{BrowserState, TabSnapshot};
use tauri::AppHandle;

/// Sends each open window its own tab snapshot and returns the one for `window_id`, so a command
/// can answer its caller with exactly what was published.
pub fn publish_tabs(app: &AppHandle, state: &BrowserState, window_id: &str) -> TabSnapshot {
    let mut answer = None;
    for id in state.session().window_ids() {
        let snapshot = state.snapshot_packet_for(&id);
        crate::error_log::ignored(crate::security::emit_shell_to(
            app,
            &id,
            "tabs-updated",
            &snapshot,
        ));
        if id == window_id {
            answer = Some(snapshot);
        }
    }
    answer.unwrap_or_else(|| state.snapshot_packet_for(window_id))
}
