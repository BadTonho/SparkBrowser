//! Sign-in to Nextcloud for sync: start the login, wait for the person to approve Spark on the
//! server's page, then save the account's "Spark" folder as the WebDAV sync target.
use crate::backup::{self, nextcloud::NextcloudLogin};
use tauri::{AppHandle, Manager};

async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| e.to_string())?
}

/// Asks the server for its login page; the interface opens it in a tab.
#[tauri::command]
pub async fn nextcloud_login_start(
    app: AppHandle,
    server: String,
) -> Result<backup::nextcloud::LoginStart, String> {
    blocking(move || app.state::<NextcloudLogin>().start(&server))
        .await
        .inspect_err(|error| {
            crate::error_log::warn(&format!("backup: Nextcloud login could not start: {error}"));
        })
}

/// `None` while the person has not approved Spark yet; the new sync state once they did.
#[tauri::command]
pub async fn nextcloud_login_poll(
    app: AppHandle,
    flow_id: u64,
    auto_save: bool,
    apply_preferences_on_direct_pull: bool,
) -> Result<Option<backup::sync::SyncStateSnapshot>, String> {
    let worker = app.clone();
    let account = blocking(move || {
        let Some(account) = worker.state::<NextcloudLogin>().poll(flow_id)? else {
            return Ok(None);
        };
        let folder = backup::nextcloud::webdav_folder(&account.server, &account.login_name)?;
        backup::nextcloud::ensure_folder(&folder, &account.login_name, &account.app_password)?;
        Ok(Some((account, folder)))
    })
    .await
    .inspect_err(|error| {
        crate::error_log::warn(&format!("backup: Nextcloud login failed: {error}"));
    })?;
    let Some((account, folder)) = account else {
        return Ok(None);
    };
    let service = app.state::<backup::service::BackupService>();
    let _operation = service.operation.lock().await;
    let worker = app.clone();
    crate::database::executor::run(app.clone(), move |conn| {
        let service = worker.state::<backup::service::BackupService>();
        backup::sync::save_config(
            conn,
            &service.sync,
            backup::sync::SyncTargetConfig::DirectWebdav {
                endpoint_url: folder.to_string(),
                username: account.login_name,
                remember_secret: true,
                auto_save,
                apply_preferences_on_direct_pull,
                nextcloud_server: Some(account.server.to_string()),
            },
            Some(account.app_password),
        )
    })
    .await
    .map(Some)
}

/// Forgets a login still waiting for approval.
#[tauri::command]
pub fn nextcloud_login_cancel(app: AppHandle) {
    app.state::<NextcloudLogin>().cancel();
}
