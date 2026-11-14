pub use super::panel_popover::Presentation;
use super::panel_popover::{self, Downloads};
use tauri::AppHandle;
pub type DownloadsPopover = panel_popover::Popover<Downloads>;
pub fn resize(app: &AppHandle) -> Result<(), String> {
    panel_popover::resize::<Downloads>(app)
}
pub fn focus_lost(app: &AppHandle) {
    panel_popover::focus_lost::<Downloads>(app)
}
pub fn dismiss(app: &AppHandle) {
    panel_popover::dismiss::<Downloads>(app)
}
pub fn sync(
    app: &AppHandle,
    state: &crate::state::browser_state::BrowserState,
) -> Result<(), String> {
    panel_popover::sync::<Downloads>(app, state)
}
pub fn set(
    app: &AppHandle,
    window_id: &str,
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    panel_popover::set::<Downloads>(
        app,
        window_id,
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}
