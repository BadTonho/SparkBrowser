use crate::{
    database::site_zoom_repo,
    state::browser_state::BrowserState,
    zoom::{SiteZoom, ZoomService, ZoomSnapshot},
};
use tauri::{AppHandle, State};
#[tauri::command]
pub fn get_site_zoom(service: State<'_, ZoomService>) -> Result<ZoomSnapshot, String> {
    service.snapshot()
}
#[tauri::command]
pub async fn set_site_zoom(
    app: AppHandle,
    state: State<'_, BrowserState>,
    service: State<'_, ZoomService>,
    origin: String,
    factor: f64,
) -> Result<ZoomSnapshot, String> {
    let _operation = state.session().operation().await;
    service.set(
        &app,
        SiteZoom {
            origin: site_zoom_repo::origin(&origin)?,
            factor,
        },
    )
}
#[tauri::command]
pub async fn apply_backup_zoom(
    app: AppHandle,
    state: State<'_, BrowserState>,
    service: State<'_, ZoomService>,
    task_id: String,
) -> Result<ZoomSnapshot, String> {
    let _operation = state.session().operation().await;
    service.import_pending(&app, &task_id)
}
