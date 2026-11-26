use crate::downloads::{
    hls,
    manager::{DownloadManager, DownloadTask, VideoRequest},
    transfer::validate_download_url,
};
use crate::state::browser_state::BrowserState;
use crate::video_download::{self, VideoScan};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn scan_page_videos(app: AppHandle, tab_id: String) -> Result<VideoScan, String> {
    video_download::scan(&app, &tab_id).await
}

/// Returns `None` when the user cancels the save dialog.
#[tauri::command]
pub async fn download_video(
    app: AppHandle,
    manager: State<'_, DownloadManager>,
    state: State<'_, BrowserState>,
    tab_id: String,
    url: String,
    referer: Option<String>,
    kind: String,
) -> Result<Option<DownloadTask>, String> {
    let url = validate_download_url(&url)?;
    let referer = referer
        .map(|page| validate_download_url(&page).map(|page| page.to_string()))
        .transpose()?;
    let title = state
        .snapshot_tabs()
        .into_iter()
        .find(|tab| tab.id == tab_id)
        .map(|tab| tab.title)
        .ok_or("downloads.not_available")?;
    let stream = match kind.as_str() {
        "file" => false,
        "hls" => true,
        _ => return Err("downloads.invalid_action".into()),
    };
    let extension = if stream {
        // Reading it first refuses an unsupported stream before asking where to save it.
        let shield = video_download::shield(&app, &tab_id, url.as_str());
        hls::fetch_media(&url, shield, referer.as_deref())
            .await?
            .extension()
    } else {
        video_download::file_extension(&url)
    };
    let name = video_download::file_name(&title, extension);
    let Some(target) =
        crate::commands::downloads::choose_target(&app, &state, name.clone()).await?
    else {
        return Ok(None);
    };
    manager
        .enqueue_video(
            &app,
            VideoRequest {
                tab_id,
                url,
                name,
                referer,
                stream,
                target,
            },
        )
        .map(Some)
}
