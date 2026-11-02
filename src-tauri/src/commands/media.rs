use crate::{
    media::{MediaService, MediaSnapshot},
    state::browser_state::BrowserState,
};
use tauri::{AppHandle, State};
#[tauri::command]
pub fn get_tab_media(
    app: AppHandle,
    service: State<'_, MediaService>,
) -> Result<MediaSnapshot, String> {
    service.snapshot(&app)
}
#[tauri::command]
pub async fn set_tab_muted(
    app: AppHandle,
    state: State<'_, BrowserState>,
    service: State<'_, MediaService>,
    tab_id: String,
    muted: bool,
) -> Result<MediaSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    service.mute(&app, &tab_id, muted).await
}
