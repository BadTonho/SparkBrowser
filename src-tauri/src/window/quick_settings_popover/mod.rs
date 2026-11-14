pub use super::panel_popover::Presentation;
use super::panel_popover::{self, QuickSettings};
use tauri::AppHandle;

pub type QuickSettingsPopover = panel_popover::Popover<QuickSettings>;

pub fn resize(app: &AppHandle) -> Result<(), String> {
    panel_popover::resize::<QuickSettings>(app)
}
pub fn is_open(app: &AppHandle) -> bool {
    panel_popover::is_open::<QuickSettings>(app)
}
pub fn focus_lost(app: &AppHandle) {
    panel_popover::focus_lost::<QuickSettings>(app)
}
pub fn dismiss(app: &AppHandle) {
    panel_popover::dismiss::<QuickSettings>(app)
}
pub fn sync(
    app: &AppHandle,
    state: &crate::state::browser_state::BrowserState,
) -> Result<(), String> {
    panel_popover::sync::<QuickSettings>(app, state)
}
pub fn set(
    app: &AppHandle,
    window_id: &str,
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    panel_popover::set::<QuickSettings>(
        app,
        window_id,
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}
