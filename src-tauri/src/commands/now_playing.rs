//! Commands for the "now playing" card shown at the bottom of the vertical-tabs sidebar.
use crate::window::new_tab::widgets::{now_playing, now_playing_control, NowPlaying};
use tauri::AppHandle;

#[tauri::command]
pub fn get_now_playing(app: AppHandle) -> Option<NowPlaying> {
    now_playing(&app)
}
/// `operation` is one of `toggle`, `next`, `previous` or `open`.
#[tauri::command]
pub async fn control_now_playing(app: AppHandle, operation: String) -> Result<(), String> {
    now_playing_control(&app, &operation).await
}
