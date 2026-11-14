use crate::{
    graphics::{self, GraphicsService, GraphicsStatus},
    shutdown::{model::ShutdownState, ShutdownService},
};
use tauri::{AppHandle, Manager, State};

fn directory(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|_| "graphics.write_failed".to_string())
}

#[tauri::command]
pub fn get_graphics_status(
    app: AppHandle,
    service: State<'_, GraphicsService>,
) -> Result<GraphicsStatus, String> {
    Ok(service.status(graphics::load(&directory(&app)?)))
}

/// Saves the choice. It applies on the next start; the status says whether a restart is pending.
#[tauri::command]
pub fn set_hardware_acceleration(
    app: AppHandle,
    service: State<'_, GraphicsService>,
    enabled: bool,
) -> Result<GraphicsStatus, String> {
    graphics::save(&directory(&app)?, enabled)?;
    Ok(service.status(enabled))
}

/// Restarts through the same protected shutdown as a WebView2 runtime update, only when the saved
/// choice differs from what this process started with.
#[tauri::command]
pub fn restart_for_graphics(
    app: AppHandle,
    service: State<'_, GraphicsService>,
    shutdown: State<'_, ShutdownService>,
) -> Result<ShutdownState, String> {
    if !service
        .status(graphics::load(&directory(&app)?))
        .restart_required
    {
        return Err("graphics.no_change".into());
    }
    shutdown.request_restart(&app)
}
