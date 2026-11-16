use crate::updates::{
    model::{UpdateChannel, UpdateState},
    UpdateService,
};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_update_state(state: State<'_, UpdateService>) -> Result<UpdateState, String> {
    state.snapshot()
}
#[tauri::command]
pub fn set_update_channel(
    app: AppHandle,
    state: State<'_, UpdateService>,
    channel: UpdateChannel,
) -> Result<UpdateState, String> {
    state.set_channel(&app, channel)
}
#[tauri::command]
pub fn set_auto_update(
    app: AppHandle,
    state: State<'_, UpdateService>,
    enabled: bool,
) -> Result<UpdateState, String> {
    let snapshot = state.set_auto_update(&app, enabled)?;
    download_in_background(&app, &snapshot);
    Ok(snapshot)
}
/// With automatic updates on, an update that was just found is downloaded and verified without asking.
fn download_in_background(app: &AppHandle, snapshot: &UpdateState) {
    #[cfg(windows)]
    if snapshot.supported && snapshot.auto_update && snapshot.status == "available" {
        if let Some(update) = &snapshot.update {
            let (app, version) = (app.clone(), update.version.clone());
            tauri::async_runtime::spawn(async move {
                let _ = crate::updates::download(&app, &version).await;
            });
        }
    }
    #[cfg(not(windows))]
    let _ = (app, snapshot);
}
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    state: State<'_, UpdateService>,
    startup: bool,
) -> Result<UpdateState, String> {
    let snapshot = state.check(&app, startup).await?;
    download_in_background(&app, &snapshot);
    Ok(snapshot)
}
#[tauri::command]
pub async fn download_update(
    app: AppHandle,
    expected_version: String,
) -> Result<UpdateState, String> {
    #[cfg(windows)]
    {
        crate::updates::download(&app, &expected_version).await
    }
    #[cfg(not(windows))]
    {
        let _ = (app, expected_version);
        Err("Updates are only supported on Windows x64 releases".into())
    }
}
#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    expected_version: String,
    confirmed_activity: bool,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        use tauri::Manager;
        let _ = confirmed_activity; // Activity approval belongs to the revisioned shutdown review.
        let current = app.state::<UpdateService>().snapshot()?;
        if current.status != "ready"
            || current.update.as_ref().map(|u| u.version.as_str()) != Some(&expected_version)
        {
            return Err("shutdown.update_unavailable".into());
        }
        app.state::<crate::shutdown::ShutdownService>()
            .request(&app, Some(expected_version))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (app, expected_version, confirmed_activity);
        Err("Updates are only supported on Windows x64 releases".into())
    }
}
