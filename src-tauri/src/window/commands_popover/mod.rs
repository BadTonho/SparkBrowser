//! The command bar as a native layer: it covers the whole window above the page, which stays
//! visible, dimmed, behind it. The bar itself is drawn from the shell's presentation, like the other panels.
pub use super::panel_popover::Presentation;
use super::panel_popover::{self, Commands};
use tauri::AppHandle;
pub type CommandsPopover = panel_popover::Popover<Commands>;
pub fn resize(app: &AppHandle) -> Result<(), String> {
    panel_popover::resize::<Commands>(app)
}
pub fn focus_lost(app: &AppHandle) {
    panel_popover::focus_lost::<Commands>(app)
}
pub fn dismiss(app: &AppHandle) {
    panel_popover::dismiss::<Commands>(app)
}
pub fn sync(
    app: &AppHandle,
    state: &crate::state::browser_state::BrowserState,
) -> Result<(), String> {
    panel_popover::sync::<Commands>(app, state)
}
pub fn set(
    app: &AppHandle,
    window_id: &str,
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    panel_popover::set::<Commands>(
        app,
        window_id,
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}
