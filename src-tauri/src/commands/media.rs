use crate::media::{MediaService, MediaSnapshot};
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
    service: State<'_, MediaService>,
    tab_id: String,
    muted: bool,
) -> Result<MediaSnapshot, String> {
    let _operation = service.operations.lock().await;
    service.mute(&app, &tab_id, muted).await
}

#[tauri::command]
pub async fn refresh_tab_media(
    app: AppHandle,
    service: State<'_, MediaService>,
    tab_id: String,
) -> Result<MediaSnapshot, String> {
    let _operation = service.operations.lock().await;
    crate::media::volume::refresh(&app, &tab_id).await
}
#[tauri::command]
pub async fn set_tab_volume(
    app: AppHandle,
    service: State<'_, MediaService>,
    tab_id: String,
    volume: u16,
    document_key: String,
) -> Result<MediaSnapshot, String> {
    let _operation = service.operations.lock().await;
    crate::media::volume::set(&app, &tab_id, volume, &document_key).await
}

#[tauri::command]
pub async fn enter_video_pip(
    app: AppHandle,
    service: State<'_, MediaService>,
    tab_id: String,
    context: String,
    document: String,
    video: String,
) -> Result<MediaSnapshot, String> {
    let _operation = service.operations.lock().await;
    crate::media::pip::enter(&app, &tab_id, &context, &document, &video).await
}
#[tauri::command]
pub async fn exit_video_pip(
    app: AppHandle,
    service: State<'_, MediaService>,
) -> Result<MediaSnapshot, String> {
    let _operation = service.operations.lock().await;
    crate::media::pip::exit(&app).await
}
