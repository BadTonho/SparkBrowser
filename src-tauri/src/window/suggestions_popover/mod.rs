//! The address bar's suggestion list as a native popover: it sits over the page, below the address bar,
//! and never takes the keyboard, so typing carries on in the address bar.
pub use super::panel_popover::Presentation;
use super::panel_popover::{self, Suggestions};
use tauri::AppHandle;
pub type SuggestionsPopover = panel_popover::Popover<Suggestions>;
pub fn resize(app: &AppHandle) -> Result<(), String> {
    panel_popover::resize::<Suggestions>(app)
}
pub fn focus_lost(app: &AppHandle) {
    panel_popover::focus_lost::<Suggestions>(app)
}
pub fn dismiss(app: &AppHandle) {
    panel_popover::dismiss::<Suggestions>(app)
}
pub fn sync(
    app: &AppHandle,
    state: &crate::state::browser_state::BrowserState,
) -> Result<(), String> {
    panel_popover::sync::<Suggestions>(app, state)
}
pub fn set(
    app: &AppHandle,
    window_id: &str,
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    panel_popover::set::<Suggestions>(
        app,
        window_id,
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}
