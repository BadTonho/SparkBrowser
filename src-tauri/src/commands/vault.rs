//! Commands of the password vault (Settings → Passwords). The rules live in `crate::vault::ops`.
//!
//! Only the trusted shell reaches these (`shell_only`), private windows refuse them, and secrets
//! passed in or out are never written to the log.
use crate::database::executor;
use crate::vault::model::{EntryData, Revealed, Status, Summary};
use crate::vault::ops::{self, Listing};
use crate::vault::service::VaultService;
use std::time::Instant;
use tauri::{AppHandle, Manager, State};
use zeroize::Zeroizing;

fn allowed(app: &AppHandle) -> Result<(), String> {
    let private = app
        .try_state::<crate::private_session::PrivateRole>()
        .is_some_and(|role| role.is_private());
    if private {
        crate::error_log::report(
            crate::error_log::Level::Info,
            "vault",
            "the password vault was refused in a private window",
            "",
        );
        return Err("vault.private_window".into());
    }
    Ok(())
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

#[tauri::command]
pub async fn vault_status(app: AppHandle, service: State<'_, VaultService>) -> Result<Status, String> {
    allowed(&app)?;
    let service = service.inner().clone();
    executor::run(app, move |conn| ops::status(conn, &service, Instant::now())).await
}

/// Creates the vault and returns the recovery key, which is shown once and never stored.
#[tauri::command]
pub async fn vault_create(
    app: AppHandle,
    service: State<'_, VaultService>,
    master_password: String,
) -> Result<String, String> {
    allowed(&app)?;
    let service = service.inner().clone();
    let password = Zeroizing::new(master_password);
    executor::run(app, move |conn| {
        ops::create(conn, &service, &password, Instant::now()).map(|key| (*key).clone())
    })
    .await
}

#[tauri::command]
pub async fn vault_unlock(
    app: AppHandle,
    service: State<'_, VaultService>,
    password: String,
) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    let password = Zeroizing::new(password);
    executor::run(app, move |conn| ops::unlock(conn, &service, &password, Instant::now())).await
}

#[tauri::command]
pub async fn vault_unlock_recovery(
    app: AppHandle,
    service: State<'_, VaultService>,
    recovery_key: String,
) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    let key = Zeroizing::new(recovery_key);
    executor::run(app, move |conn| {
        ops::unlock_with_recovery(conn, &service, &key, Instant::now())
    })
    .await
}

#[tauri::command]
pub fn vault_lock(app: AppHandle, service: State<'_, VaultService>) -> Result<(), String> {
    allowed(&app)?;
    service.lock();
    Ok(())
}

#[tauri::command]
pub async fn vault_list(app: AppHandle, service: State<'_, VaultService>) -> Result<Listing, String> {
    allowed(&app)?;
    let service = service.inner().clone();
    executor::run(app, move |conn| ops::list(conn, &service, Instant::now())).await
}

#[tauri::command]
pub async fn vault_reveal(
    app: AppHandle,
    service: State<'_, VaultService>,
    id: String,
) -> Result<Revealed, String> {
    allowed(&app)?;
    let service = service.inner().clone();
    executor::run(app, move |conn| ops::reveal(conn, &service, &id, Instant::now())).await
}

#[tauri::command]
pub async fn vault_save_entry(
    app: AppHandle,
    service: State<'_, VaultService>,
    id: Option<String>,
    entry: EntryData,
) -> Result<Summary, String> {
    allowed(&app)?;
    let service = service.inner().clone();
    executor::run(app, move |conn| {
        ops::save(conn, &service, id.as_deref(), &entry, timestamp(), Instant::now())
    })
    .await
}

#[tauri::command]
pub async fn vault_delete_entry(
    app: AppHandle,
    service: State<'_, VaultService>,
    id: String,
) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    executor::run(app, move |conn| ops::delete(conn, &service, &id, Instant::now())).await
}

#[tauri::command]
pub async fn vault_change_master_password(
    app: AppHandle,
    service: State<'_, VaultService>,
    current: String,
    new_password: String,
) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    let (current, new_password) = (Zeroizing::new(current), Zeroizing::new(new_password));
    executor::run(app, move |conn| {
        ops::change_password(conn, &service, &current, &new_password, Instant::now())
    })
    .await
}

#[tauri::command]
pub async fn vault_set_auto_lock(
    app: AppHandle,
    service: State<'_, VaultService>,
    minutes: u32,
) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    executor::run(app, move |conn| ops::set_auto_lock(conn, &service, minutes, Instant::now())).await
}

/// Erases the vault after both secrets were lost. The interface asks for an explicit confirmation.
#[tauri::command]
pub async fn vault_reset(
    app: AppHandle,
    service: State<'_, VaultService>,
    confirm: bool,
) -> Result<(), String> {
    allowed(&app)?;
    if !confirm {
        return Err("vault.not_confirmed".into());
    }
    let service = service.inner().clone();
    executor::run(app, move |conn| ops::reset(conn, &service)).await
}
