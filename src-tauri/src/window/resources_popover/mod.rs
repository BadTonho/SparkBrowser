//! The resources panel as a native popover under its button in the top bar.
pub use super::panel_popover::Presentation;
use super::panel_popover::{self, Resources};
use tauri::AppHandle;
pub type ResourcesPopover = panel_popover::Popover<Resources>;
pub fn resize(app: &AppHandle) -> Result<(), String> {
    panel_popover::resize::<Resources>(app)
}
pub fn focus_lost(app: &AppHandle) {
    panel_popover::focus_lost::<Resources>(app)
}
pub fn dismiss(app: &AppHandle) {
    panel_popover::dismiss::<Resources>(app)
}
pub fn sync(
    app: &AppHandle,
    state: &crate::state::browser_state::BrowserState,
) -> Result<(), String> {
    panel_popover::sync::<Resources>(app, state)
}
pub fn set(
    app: &AppHandle,
    window_id: &str,
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    panel_popover::set::<Resources>(
        app,
        window_id,
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}
