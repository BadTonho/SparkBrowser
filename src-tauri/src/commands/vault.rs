//! Commands of the password vault (Settings → Passwords). The rules live in `crate::vault::ops`.
//!
//! Only the trusted shell reaches these (`shell_only`), private windows refuse them, and secrets
//! passed in or out are never written to the log.
use crate::database::executor;
use crate::error_log::{self, Level};
use crate::vault::hello::{self, HelloError};
use crate::vault::csv;
use crate::vault::model::{EntryData, Revealed, Status, Summary};
use crate::vault::ops::{self, ImportResult, Listing};
use crate::vault::service::VaultService;
use std::io::Read;
use std::time::Instant;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
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

/// The Windows Hello proof for an action on a password, asked only when the vault wants it each time.
/// An open vault is required first, so a locked one never shows a prompt it cannot use.
async fn proof_if_required(
    app: &AppHandle,
    service: &VaultService,
    what: &'static str,
) -> Result<Option<Zeroizing<Vec<u8>>>, String> {
    service.key(Instant::now())?;
    if executor::run(app.clone(), |conn| ops::requires_hello(conn)).await? {
        Ok(Some(hello_step(what, hello::secret).await?))
    } else {
        Ok(None)
    }
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
    let proof = proof_if_required(&app, &service, "showing a vault password").await?;
    executor::run(app, move |conn| {
        ops::confirm_if_required(conn, proof.as_deref().map(|p| p.as_slice()))?;
        ops::reveal(conn, &service, &id, Instant::now())
    })
    .await
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

/// Turns "ask for Windows Hello every time a password is shown or copied" on or off.
#[tauri::command]
pub async fn vault_set_require_hello(
    app: AppHandle,
    service: State<'_, VaultService>,
    enabled: bool,
) -> Result<(), String> {
    allowed(&app)?;
    let service = service.inner().clone();
    // Changing it while it is on needs Windows Hello once more; turning it on does not.
    let proof = proof_if_required(&app, &service, "changing the Windows Hello requirement").await?;
    executor::run(app, move |conn| {
        ops::set_require_hello(conn, &service, enabled, proof.as_deref().map(|p| p.as_slice()), Instant::now())
    })
    .await
}

/// Reads a file the person picked, bounded, and wipes it from memory when dropped.
fn read_import_file(path: &std::path::Path) -> Result<Zeroizing<Vec<u8>>, String> {
    let describe = |error: std::io::Error| {
        error_log::report(Level::Error, "vault", "the file to import could not be read", &error.to_string());
        "vault.import_failed".to_string()
    };
    let file = std::fs::File::open(path).map_err(describe)?;
    if file.metadata().map_err(describe)?.len() > csv::MAX_IMPORT_BYTES {
        error_log::report(Level::Warning, "vault", "the file to import is too large", "");
        return Err("vault.import_too_large".into());
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(csv::MAX_IMPORT_BYTES + 1).read_to_end(&mut bytes).map_err(describe)?;
    Ok(bytes)
}

/// Adds the passwords of a CSV file (Chrome, Edge and similar). The file is chosen and read here, so
/// its contents never pass through the interface. `None` means the dialog was cancelled.
#[tauri::command]
pub async fn vault_import_csv(
    app: AppHandle,
    service: State<'_, VaultService>,
) -> Result<Option<ImportResult>, String> {
    allowed(&app)?;
    let service = service.inner().clone();
    service.key(Instant::now())?;
    let picker = app.clone();
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        let Some(path) = picker.dialog().file().add_filter("CSV", &["csv"]).blocking_pick_file() else {
            return Ok(None);
        };
        let path = path.into_path().map_err(|e| {
            error_log::report(Level::Error, "vault", "the chosen import file has no usable path", &e.to_string());
            "vault.import_failed".to_string()
        })?;
        read_import_file(&path).map(Some)
    })
    .await
    .map_err(|error| {
        error_log::report(Level::Error, "vault", "reading the import file stopped unexpectedly", &error.to_string());
        "vault.import_failed".to_string()
    })??;
    let Some(bytes) = bytes else { return Ok(None) };
    let parsed = csv::parse(&bytes).map_err(|code| {
        error_log::report(Level::Warning, "vault", "the file to import is not a password export", "");
        code
    })?;
    drop(bytes);
    let left_out = parsed.skipped;
    let mut result = executor::run(app, move |conn| {
        ops::import_entries(conn, &service, parsed.entries, timestamp(), Instant::now())
    })
    .await?;
    result.skipped += left_out;
    Ok(Some(result))
}

/// Writes every password to a CSV file, in plain text, after the person confirms who they are: with
/// the master password when one is given, with Windows Hello otherwise. `None` means the dialog was
/// cancelled. Otherwise the number of passwords written.
#[tauri::command]
pub async fn vault_export_csv(
    app: AppHandle,
    service: State<'_, VaultService>,
    current_password: Option<String>,
) -> Result<Option<usize>, String> {
    allowed(&app)?;
    let service = service.inner().clone();
    service.key(Instant::now())?;
    match current_password {
        Some(password) => {
            let (password, service) = (Zeroizing::new(password), service.clone());
            executor::run(app.clone(), move |conn| {
                ops::verify_password(conn, &service, &password, Instant::now()).map(|_| ())
            })
            .await?;
        }
        None => {
            let secret = hello_step("confirming Windows Hello for the export", hello::secret).await?;
            executor::run(app.clone(), move |conn| ops::confirm_hello(conn, &secret)).await?;
        }
    }
    let (entries, unreadable) =
        executor::run(app.clone(), move |conn| ops::export_entries(conn, &service, Instant::now())).await?;
    let count = entries.len();
    let bytes = csv::render(&entries);
    drop(entries);
    let saver = app.clone();
    let written = tauri::async_runtime::spawn_blocking(move || {
        let Some(path) = saver
            .dialog()
            .file()
            .add_filter("CSV", &["csv"])
            .set_file_name("spark-passwords.csv")
            .blocking_save_file()
        else {
            return Ok(false);
        };
        let path = path.into_path().map_err(|e| {
            error_log::report(Level::Error, "vault", "the chosen export file has no usable path", &e.to_string());
            "vault.export_failed".to_string()
        })?;
        if !path.to_string_lossy().to_lowercase().ends_with(".csv") {
            return Err("vault.export_failed".to_string());
        }
        crate::backup::files::write_bytes(&path, &bytes).map_err(|detail| {
            error_log::report(Level::Error, "vault", "the password export file could not be written", &detail);
            "vault.export_failed".to_string()
        })?;
        Ok(true)
    })
    .await
    .map_err(|error| {
        error_log::report(Level::Error, "vault", "writing the password export stopped unexpectedly", &error.to_string());
        "vault.export_failed".to_string()
    })??;
    if !written {
        return Ok(None);
    }
    error_log::report(
        Level::Info,
        "vault",
        &format!("{count} passwords were exported to a CSV file ({unreadable} unreadable entries left out)"),
        "",
    );
    Ok(Some(count))
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
    let proof = proof_if_required(&app, &service, "copying a vault password").await?;
    let password = executor::run(app.clone(), move |conn| {
        ops::confirm_if_required(conn, proof.as_deref().map(|p| p.as_slice()))?;
        ops::password(conn, &service, &id, Instant::now())
    })
    .await?;
    crate::vault::clipboard::copy(&app, password).await
}

/// What the recovery key file says (in both languages the interface offers). The key sits on its own line.
fn recovery_file(recovery_key: &str) -> String {
    format!(
        "Spark Browser - password vault recovery key\r\n\
         Spark Browser - chave de recuperacao do cofre de senhas\r\n\
         \r\n\
         {recovery_key}\r\n\
         \r\n\
         EN: This key opens the vault if you forget the master password or Windows Hello stops working. \
         Keep this file somewhere safe, away from this computer (for example a USB drive), and do not share it: \
         anyone who has it and your vault data can open the vault.\r\n\
         PT: Esta chave abre o cofre se voce esquecer a senha mestra ou o Windows Hello deixar de funcionar. \
         Guarde este arquivo em um lugar seguro, longe deste computador (por exemplo um pendrive), e nao o compartilhe: \
         quem tiver o arquivo e os dados do seu cofre consegue abri-lo.\r\n"
    )
}

/// Saves the recovery key shown after creating the vault to a text file the person chooses. The text
/// must have the shape of a recovery key, so this cannot be used to write anything else. The file is
/// not encrypted, which the interface says. `false` means the dialog was cancelled.
#[tauri::command]
pub async fn vault_save_recovery_file(app: AppHandle, recovery_key: String) -> Result<bool, String> {
    allowed(&app)?;
    let recovery_key = Zeroizing::new(recovery_key);
    if !crate::vault::crypto::is_recovery_key_shape(&recovery_key) {
        error_log::report(Level::Warning, "vault", "saving the recovery key file was refused: the text is not a recovery key", "");
        return Err("vault.save_failed".into());
    }
    let contents = Zeroizing::new(recovery_file(recovery_key.trim()));
    tauri::async_runtime::spawn_blocking(move || {
        let Some(path) = app
            .dialog()
            .file()
            .add_filter("Text", &["txt"])
            .set_file_name("spark-recovery-key.txt")
            .blocking_save_file()
        else {
            return Ok(false);
        };
        let path = path.into_path().map_err(|e| {
            error_log::report(Level::Error, "vault", "the chosen recovery key file has no usable path", &e.to_string());
            "vault.save_failed".to_string()
        })?;
        if !path.to_string_lossy().to_lowercase().ends_with(".txt") {
            return Err("vault.save_failed".to_string());
        }
        crate::backup::files::write_bytes(&path, contents.as_bytes()).map_err(|detail| {
            error_log::report(Level::Error, "vault", "the recovery key file could not be written", &detail);
            "vault.save_failed".to_string()
        })?;
        error_log::report(Level::Info, "vault", "the recovery key was saved to a file", "");
        Ok(true)
    })
    .await
    .map_err(|error| {
        error_log::report(Level::Error, "vault", "saving the recovery key file stopped unexpectedly", &error.to_string());
        "vault.save_failed".to_string()
    })?
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
