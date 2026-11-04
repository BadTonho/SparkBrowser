pub use super::panel_popover::Presentation;
use super::panel_popover::{self, Tools};
use tauri::AppHandle;
pub type ToolsPopover = panel_popover::Popover<Tools>;
pub fn resize(app: &AppHandle) -> Result<(), String> {
    panel_popover::resize::<Tools>(app)
}
pub fn is_open(app: &AppHandle) -> bool {
    panel_popover::is_open::<Tools>(app)
}
pub fn focus_lost(app: &AppHandle) {
    panel_popover::focus_lost::<Tools>(app)
}
pub fn dismiss(app: &AppHandle) {
    panel_popover::dismiss::<Tools>(app)
}
pub fn sync(
    app: &AppHandle,
    state: &crate::state::browser_state::BrowserState,
) -> Result<(), String> {
    panel_popover::sync::<Tools>(app, state)
}
pub fn set(
    app: &AppHandle,
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    panel_popover::set::<Tools>(app, visible, session_id, presentation, restore_focus)
}
