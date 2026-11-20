use crate::{
    network::{self, NetworkStatus, Provider},
    shutdown::{model::ShutdownState, ShutdownService},
};
use tauri::{AppHandle, Manager, State};

fn directory(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|_| "network.write_failed".to_string())
}

#[tauri::command]
pub fn get_network_status(app: AppHandle) -> Result<NetworkStatus, String> {
    Ok(network::status(network::load(&directory(&app)?)))
}

/// Saves the choice (`None` turns DNS over HTTPS off). It applies on the next start; the status
/// says whether a restart is pending.
#[tauri::command]
pub fn set_doh_provider(
    app: AppHandle,
    provider: Option<String>,
) -> Result<NetworkStatus, String> {
    let provider = match provider {
        Some(id) => Some(Provider::from_id(&id).ok_or("network.unknown_provider")?),
        None => None,
    };
    network::save(&directory(&app)?, provider)?;
    Ok(network::status(provider))
}

/// Restarts through the same protected shutdown as the graphics change, only when the saved choice
/// differs from what this process started with.
#[tauri::command]
pub fn restart_for_network(
    app: AppHandle,
    shutdown: State<'_, ShutdownService>,
) -> Result<ShutdownState, String> {
    if !network::status(network::load(&directory(&app)?)).restart_required {
        return Err("network.no_change".into());
    }
    shutdown.request_restart(&app)
}
