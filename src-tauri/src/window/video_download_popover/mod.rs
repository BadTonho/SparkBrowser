//! The video download panel as a native popover under its button in the top bar.
pub use super::panel_popover::Presentation;
use super::panel_popover::{self, VideoDownload};
use tauri::AppHandle;
pub type VideoDownloadPopover = panel_popover::Popover<VideoDownload>;
pub fn resize(app: &AppHandle) -> Result<(), String> {
    panel_popover::resize::<VideoDownload>(app)
}
pub fn focus_lost(app: &AppHandle) {
    panel_popover::focus_lost::<VideoDownload>(app)
}
pub fn dismiss(app: &AppHandle) {
    panel_popover::dismiss::<VideoDownload>(app)
}
pub fn sync(
    app: &AppHandle,
    state: &crate::state::browser_state::BrowserState,
) -> Result<(), String> {
    panel_popover::sync::<VideoDownload>(app, state)
}
pub fn set(
    app: &AppHandle,
    window_id: &str,
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    panel_popover::set::<VideoDownload>(
        app,
        window_id,
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}
