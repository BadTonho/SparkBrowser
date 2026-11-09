pub use super::panel_popover::Presentation;
use super::panel_popover::{self, Settings};
use tauri::AppHandle;

pub type SettingsOverlay = panel_popover::Popover<Settings>;
pub fn resize(app: &AppHandle) -> Result<(), String> {
    panel_popover::resize::<Settings>(app)
}
pub fn is_open(app: &AppHandle) -> bool {
    panel_popover::is_open::<Settings>(app)
}
pub fn sync(
    app: &AppHandle,
    state: &crate::state::browser_state::BrowserState,
) -> Result<(), String> {
    panel_popover::sync::<Settings>(app, state)
}
pub fn set(
    app: &AppHandle,
    visible: bool,
    session: u64,
    presentation: Option<Presentation>,
    focus: bool,
) -> Result<(), String> {
    panel_popover::set::<Settings>(app, visible, session, presentation, focus)
}
