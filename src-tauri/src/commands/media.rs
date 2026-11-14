use crate::media::{MediaService, MediaSnapshot};
use tauri::{AppHandle, State};
#[tauri::command]
pub async fn set_pip_window_shadow(
    app: AppHandle,
    service: State<'_, MediaService>,
    enabled: bool,
) -> Result<bool, String> {
    let _operation = service.operations.lock().await;
    #[cfg(any(windows, target_os = "linux"))]
    {
        crate::media::pip_window::set_shadow(&app, enabled).await
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = (app, enabled);
        Err("media.unavailable".into())
    }
}
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
    pip_session: Option<String>,
    pip_sequence: Option<u64>,
) -> Result<MediaSnapshot, String> {
    let _operation = service.operations.lock().await;
    let result =
        crate::media::volume::set(&app, &tab_id, volume, &document_key, pip_session.as_deref())
            .await;
    #[cfg(any(windows, target_os = "linux"))]
    if let (Some(session), Some(sequence)) = (pip_session, pip_sequence) {
        crate::media::pip_window::acknowledge(&app, &session, sequence);
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    let _ = (pip_session, pip_sequence);
    result
}

#[tauri::command]
pub async fn enter_video_pip(
    app: AppHandle,
    service: State<'_, MediaService>,
    tab_id: String,
    context: String,
    document: String,
    video: String,
    presentation: Option<crate::media::pip::Presentation>,
) -> Result<MediaSnapshot, String> {
    let _operation = service.operations.lock().await;
    crate::media::pip::enter(&app, &tab_id, &context, &document, &video, presentation).await
}
#[tauri::command]
pub async fn exit_video_pip(
    app: AppHandle,
    service: State<'_, MediaService>,
) -> Result<MediaSnapshot, String> {
    let _operation = service.operations.lock().await;
    crate::media::pip::exit(&app).await
}
