//! Commands of the password vault (Settings → Passwords). The rules live in `crate::vault::ops`.
//!
//! Only the trusted shell reaches these (`shell_only`), private windows refuse them, and secrets
//! passed in or out are never written to the log.
use crate::database::executor;
use crate::error_log::{self, Level};
use crate::vault::hello::{self, HelloError};
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

/// Runs one Windows Hello step off the interface and database threads: it waits for the person to
/// answer the Windows prompt. Failures are logged here; only a stable code goes back to the interface.
async fn hello_step(
    what: &'static str,
    work: fn() -> Result<Zeroizing<Vec<u8>>, HelloError>,
) -> Result<Zeroizing<Vec<u8>>, String> {
    let outcome = tauri::async_runtime::spawn_blocking(work).await.map_err(|error| {
        error_log::report(Level::Error, "vault", &format!("{what} stopped unexpectedly"), &error.to_string());
        HelloError::Failed(String::new()).code().to_string()
    })?;
    outcome.map_err(|error| {
        let (level, summary, detail) = match &error {
            HelloError::Cancelled => (Level::Info, format!("{what} was cancelled in Windows Hello"), String::new()),
            HelloError::Unavailable => {
                (Level::Warning, format!("{what} needs Windows Hello, which is not available here"), String::new())
            }
            HelloError::Failed(detail) => (Level::Error, format!("{what} failed in Windows Hello"), detail.clone()),
        };
        error_log::report(level, "vault", &summary, &detail);
        error.code().to_string()
    })
}

/// Deletes the Windows Hello key pair of the vault (best effort: the vault no longer uses it).
async fn forget_hello() {
    if !hello::is_available() {
        return;
    }
    match tauri::async_runtime::spawn_blocking(hello::forget).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => error_log::report(
            Level::Warning,
            "vault",
            "the Windows Hello key of the vault could not be deleted",
            &format!("{error:?}"),
        ),
        Err(error) => error_log::report(
            Level::Warning,
            "vault",
            "deleting the Windows Hello key of the vault stopped unexpectedly",
            &error.to_string(),
        ),
    }
}

#[tauri::command]
pub async fn vault_status(app: AppHandle, service: State<'_, VaultService>) -> Result<Status, String> {
    allowed(&app)?;
    let service = service.inner().clone();
    let available = tauri::async_runtime::spawn_blocking(hello::is_available).await.unwrap_or(false);
    let mut status = executor::run(app, move |conn| ops::status(conn, &service, Instant::now())).await?;
    status.hello_available = available;
    Ok(status)
}

/// Creates the vault and returns the recovery key, which is shown once and never stored.
#[tauri::command]
pub async fn vault_create(
    app: AppHandle,
    service: State<'_, VaultService>,
    master_password: Option<String>,
    use_hello: bool,
) -> Result<String, String> {
    allowed(&app)?;
    let service = service.inner().clone();
    let password = master_password.map(Zeroizing::new);
    let secret = if use_hello {
        Some(hello_step("turning on Windows Hello for the new vault", hello::enroll).await?)
    } else {
        None
    };
    let created = executor::run(app, move |conn| {
        ops::create(
            conn,
            &service,
            password.as_ref().map(|p| p.as_str()),
            secret.as_ref().map(|s| s.as_slice()),
            Instant::now(),
        )
        .map(|key| (*key).clone())
    })
    .await;
    if created.is_err() && use_hello {
        forget_hello().await;
    }
    created
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

/// Opens the vault after the person proves who they are to Windows (PIN, fingerprint or face).
#[tauri::command]
pub async fn vault_unlock_hello(app: AppHandle, service: State<'_, VaultService>) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    let secret = hello_step("unlocking the vault with Windows Hello", hello::secret).await?;
    executor::run(app, move |conn| ops::unlock_with_hello(conn, &service, &secret, Instant::now())).await
}

#[tauri::command]
pub async fn vault_hello_enable(app: AppHandle, service: State<'_, VaultService>) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    // Ask Windows only for an open vault: asking replaces the key pair, so a locked vault must not get that far.
    service.key(Instant::now())?;
    let secret = hello_step("turning on Windows Hello", hello::enroll).await?;
    executor::run(app, move |conn| ops::enable_hello(conn, &service, &secret, Instant::now())).await
}

#[tauri::command]
pub async fn vault_hello_disable(app: AppHandle, service: State<'_, VaultService>) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    executor::run(app, move |conn| ops::disable_hello(conn, &service, Instant::now())).await?;
    forget_hello().await;
    Ok(())
}

#[tauri::command]
pub async fn vault_password_enable(
    app: AppHandle,
    service: State<'_, VaultService>,
    new_password: String,
) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    let new_password = Zeroizing::new(new_password);
    executor::run(app, move |conn| ops::enable_password(conn, &service, &new_password, Instant::now())).await
}

#[tauri::command]
pub async fn vault_password_disable(
    app: AppHandle,
    service: State<'_, VaultService>,
    current: String,
) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    let current = Zeroizing::new(current);
    executor::run(app, move |conn| ops::disable_password(conn, &service, &current, Instant::now())).await
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

/// Copies the password of one entry straight from the backend, so it never reaches the interface.
/// The clipboard is emptied again after 30 seconds if it still holds it.
#[tauri::command]
pub async fn vault_copy_password(
    app: AppHandle,
    service: State<'_, VaultService>,
    id: String,
) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    let password =
        executor::run(app.clone(), move |conn| ops::password(conn, &service, &id, Instant::now())).await?;
    crate::vault::clipboard::copy(&app, password).await
}

/// Copies text the interface already shows (a username, the recovery key) with the same clean-up.
#[tauri::command]
pub async fn vault_copy_text(app: AppHandle, text: String) -> Result<(), String> {
    allowed(&app)?;
    crate::vault::clipboard::copy(&app, Zeroizing::new(text)).await
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
    executor::run(app, move |conn| ops::reset(conn, &service)).await?;
    forget_hello().await;
    Ok(())
}
