//! Commands of the app lock. The ones that open a locked profile (or quit, or erase it) answer before
//! start-up is ready, because that is exactly when they are needed; everything else about the lock is
//! managed from Settings, in a profile that is already open. Secrets passed in are never logged.
use crate::app_lock::ops::{self, First, Proof, Status};
use crate::app_lock::{erase, AppLockService, Method};
use crate::database::executor;
use crate::error_log::{self, Level};
use crate::profile::ProfileRole;
use crate::vault::hello;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use zeroize::Zeroizing;

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

#[derive(Serialize)]
pub struct LockState {
    pub locked: bool,
    #[serde(flatten)]
    pub status: Status,
    /// This computer offers Windows Hello (always false outside Windows).
    pub hello_available: bool,
}

/// How the profile is locked, and whether it is locked right now. It answers before start-up is ready.
#[tauri::command]
pub async fn get_lock_state(
    app: AppHandle,
    service: State<'_, AppLockService>,
) -> Result<LockState, String> {
    let locked = service.is_locked();
    let status = executor::run(app, |conn| ops::status(conn, unix_now())).await?;
    let hello_available = tauri::async_runtime::spawn_blocking(hello::is_available)
        .await
        .unwrap_or(false);
    Ok(LockState {
        locked,
        status,
        hello_available,
    })
}

fn require_locked(service: &AppLockService) -> Result<(), String> {
    if service.is_locked() {
        return Ok(());
    }
    error_log::report(
        Level::Warning,
        "app-lock",
        "an unlock command was refused: the profile is not locked",
        "",
    );
    Err("lock.not_locked".into())
}

/// The secret was accepted: the profile shows itself. Only one of several simultaneous answers does it.
async fn finish_unlock(app: &AppHandle, service: &AppLockService, how: &str) -> Result<(), String> {
    if !service.open() {
        return Ok(());
    }
    if let Err(details) = crate::session_start::start(app) {
        service.set_locked(true);
        error_log::report(
            Level::Critical,
            "app-lock",
            "the profile was unlocked but could not be started",
            &details,
        );
        return Err("lock.start_failed".into());
    }
    error_log::report(
        Level::Info,
        "app-lock",
        &format!("the profile was unlocked with {how}"),
        "",
    );
    Ok(())
}

async fn unlock_with(
    app: AppHandle,
    service: &AppLockService,
    how: &'static str,
    proof: impl FnOnce(&Connection) -> Result<Method, String> + Send + 'static,
) -> Result<(), String> {
    require_locked(service)?;
    executor::run(app.clone(), move |conn| proof(conn)).await?;
    finish_unlock(&app, service, how).await
}

#[tauri::command]
pub async fn app_unlock_password(
    app: AppHandle,
    service: State<'_, AppLockService>,
    password: String,
) -> Result<(), String> {
    let password = Zeroizing::new(password);
    unlock_with(app, &service, "the app lock password", move |conn| {
        ops::verify(conn, &Proof::Password(&password), unix_now())
    })
    .await
}

#[tauri::command]
pub async fn app_unlock_vault_password(
    app: AppHandle,
    service: State<'_, AppLockService>,
    password: String,
) -> Result<(), String> {
    let password = Zeroizing::new(password);
    unlock_with(app, &service, "the vault master password", move |conn| {
        ops::verify(conn, &Proof::Vault(&password), unix_now())
    })
    .await
}

/// Windows asks for the PIN, fingerprint or face; nothing secret passes through the interface.
#[tauri::command]
pub async fn app_unlock_hello(
    app: AppHandle,
    service: State<'_, AppLockService>,
) -> Result<(), String> {
    require_locked(&service)?;
    let secret =
        super::vault::hello_step("unlocking Spark with Windows Hello", hello::secret_app_lock)
            .await?;
    unlock_with(app, &service, "Windows Hello", move |conn| {
        ops::verify(conn, &Proof::Hello(&secret), unix_now())
    })
    .await
}

/// Quits from the lock screen (there is no browser behind it to ask any question).
#[tauri::command]
pub fn request_exit(app: AppHandle, service: State<'_, AppLockService>) -> Result<(), String> {
    require_locked(&service)?;
    crate::app_lock::exit_locked(&app);
    Ok(())
}

/// "Forgot the password": erases everything of this profile and starts it again, empty and without a
/// lock. The interface asks for an explicit confirmation first.
#[tauri::command]
pub fn erase_locked_profile(
    app: AppHandle,
    service: State<'_, AppLockService>,
    role: State<'_, ProfileRole>,
    confirm: bool,
) -> Result<(), String> {
    require_locked(&service)?;
    if !confirm {
        return Err("lock.not_confirmed".into());
    }
    error_log::report(
        Level::Warning,
        "app-lock",
        "the data of a locked profile is being erased because its secret was forgotten",
        role.id(),
    );
    erase::start_helper(role.id())?;
    crate::app_lock::exit_locked(&app);
    Ok(())
}

// ---- Settings: turning the lock on and off and changing how it opens ------------------------------

fn allowed(app: &AppHandle) -> Result<(), String> {
    super::vault::allowed(app)
}

/// What the person presents to prove it is them. For Windows Hello `secret` is empty: Windows asks.
#[derive(Deserialize)]
pub struct ProofInput {
    pub method: Method,
    pub secret: Option<String>,
}

enum Owned {
    Password(Zeroizing<String>),
    Hello(Zeroizing<Vec<u8>>),
    Vault(Zeroizing<String>),
}

impl Owned {
    fn proof(&self) -> Proof<'_> {
        match self {
            Owned::Password(secret) => Proof::Password(secret),
            Owned::Hello(secret) => Proof::Hello(secret),
            Owned::Vault(secret) => Proof::Vault(secret),
        }
    }
}

async fn gather(input: ProofInput, what: &'static str) -> Result<Owned, String> {
    let typed = || {
        input
            .secret
            .clone()
            .map(Zeroizing::new)
            .ok_or_else(|| "lock.wrong_secret".to_string())
    };
    Ok(match input.method {
        Method::Password => Owned::Password(typed()?),
        Method::Vault => Owned::Vault(typed()?),
        Method::Hello => {
            Owned::Hello(super::vault::hello_step(what, hello::secret_app_lock).await?)
        }
    })
}

/// Best effort: the key pair of Windows Hello for the lock is no longer needed.
async fn forget_hello_key() {
    if !hello::is_available() {
        return;
    }
    match tauri::async_runtime::spawn_blocking(hello::forget_app_lock).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => error_log::report(
            Level::Warning,
            "app-lock",
            "the Windows Hello key of the app lock could not be deleted",
            &format!("{error:?}"),
        ),
        Err(error) => error_log::report(
            Level::Warning,
            "app-lock",
            "deleting the Windows Hello key of the app lock stopped unexpectedly",
            &error.to_string(),
        ),
    }
}

fn changed(what: &str) {
    error_log::report(Level::Info, "app-lock", what, "");
}

/// Turns the lock on with its first method: `password` (the new password in `secret`), `hello`
/// (Windows asks) or `vault` (the vault master password in `secret`).
#[tauri::command]
pub async fn app_lock_enable(
    app: AppHandle,
    method: Method,
    secret: Option<String>,
) -> Result<(), String> {
    allowed(&app)?;
    let typed = secret.map(Zeroizing::new);
    let hello_secret = if method == Method::Hello {
        Some(
            super::vault::hello_step(
                "turning on Windows Hello for the app lock",
                hello::enroll_app_lock,
            )
            .await?,
        )
    } else {
        None
    };
    let result = executor::run(app, move |conn| {
        let typed = typed.as_deref().map(String::as_str);
        match (method, typed, hello_secret.as_deref()) {
            (Method::Password, Some(password), _) => ops::enable(conn, First::Password(password)),
            (Method::Vault, Some(password), _) => ops::enable(conn, First::Vault(password)),
            (Method::Hello, _, Some(secret)) => ops::enable(conn, First::Hello(secret)),
            _ => Err("lock.invalid_request".into()),
        }
    })
    .await;
    if result.is_err() && method == Method::Hello {
        forget_hello_key().await;
    }
    result?;
    changed("the app lock was turned on");
    Ok(())
}

/// Adds a way in. `proof` is a way in that works now.
#[tauri::command]
pub async fn app_lock_add_method(
    app: AppHandle,
    method: Method,
    secret: Option<String>,
    proof: ProofInput,
) -> Result<(), String> {
    allowed(&app)?;
    let proof = gather(proof, "confirming Windows Hello to add a way to open Spark").await?;
    let typed = secret.map(Zeroizing::new);
    let hello_secret = if method == Method::Hello {
        Some(
            super::vault::hello_step(
                "turning on Windows Hello for the app lock",
                hello::enroll_app_lock,
            )
            .await?,
        )
    } else {
        None
    };
    executor::run(app, move |conn| {
        let typed = typed.as_deref().map(String::as_str);
        let now = unix_now();
        match (method, typed, hello_secret.as_deref()) {
            (Method::Password, Some(password), _) => {
                ops::set_password(conn, &proof.proof(), password, now)
            }
            (Method::Vault, Some(password), _) => {
                ops::add_vault(conn, &proof.proof(), password, now)
            }
            (Method::Hello, _, Some(secret)) => ops::add_hello(conn, &proof.proof(), secret, now),
            _ => Err("lock.invalid_request".into()),
        }
    })
    .await?;
    changed("a way to open Spark was added to the app lock");
    Ok(())
}

/// Turns one way in off (not the last one).
#[tauri::command]
pub async fn app_lock_remove_method(
    app: AppHandle,
    method: Method,
    proof: ProofInput,
) -> Result<(), String> {
    allowed(&app)?;
    let proof = gather(
        proof,
        "confirming Windows Hello to turn off a way to open Spark",
    )
    .await?;
    executor::run(app, move |conn| {
        ops::remove_method(conn, &proof.proof(), method, unix_now())
    })
    .await?;
    if method == Method::Hello {
        forget_hello_key().await;
    }
    changed("a way to open Spark was turned off in the app lock");
    Ok(())
}

/// Turns the lock off.
#[tauri::command]
pub async fn app_lock_disable(app: AppHandle, proof: ProofInput) -> Result<(), String> {
    allowed(&app)?;
    let proof = gather(proof, "confirming Windows Hello to turn the app lock off").await?;
    executor::run(app, move |conn| {
        ops::disable(conn, &proof.proof(), unix_now())
    })
    .await?;
    forget_hello_key().await;
    changed("the app lock was turned off");
    Ok(())
}
