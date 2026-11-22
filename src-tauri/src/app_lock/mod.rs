//! The app lock: an optional secret asked when a profile opens, so nobody uses that profile's Spark
//! without it. It only gates the interface: **it does not encrypt anything on disk** (history,
//! favorites and the like stay readable to whoever opens the data folder), and Spark says so.
//!
//! Each profile has its own lock, stored in its own database (`app_lock` table, outside backups and
//! private-window copies). It can be opened with any of three methods: a password of its own, Windows
//! Hello, or the master password of the profile's password vault (checked without opening the vault).
//! Wrong guesses are slowed down by a brake that survives a restart.
pub mod erase;
pub mod ops;
pub mod verifier;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
pub use verifier::Verifier;

/// How a profile may be opened. A lock exists while at least one method is on.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub version: u32,
    /// A password of the lock's own.
    pub password: Option<Verifier>,
    /// Windows Hello: a hash of the signature Windows produces for this profile's lock key.
    pub hello: Option<String>,
    /// The master password of the profile's password vault opens Spark too.
    pub vault: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            version: 1,
            ..Self::default()
        }
    }

    pub fn method_count(&self) -> usize {
        usize::from(self.password.is_some())
            + usize::from(self.hello.is_some())
            + usize::from(self.vault)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Password,
    Hello,
    Vault,
}

/// Managed state: whether this process is locked right now. While it is, no command except the ones
/// that unlock (or quit) works, and no tab or page content exists.
#[derive(Default)]
pub struct AppLockService {
    locked: AtomicBool,
}

impl AppLockService {
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Acquire)
    }

    /// Opens the lock: true for the one caller that changed it (several answers can arrive together).
    pub fn open(&self) -> bool {
        self.locked
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub fn set_locked(&self, locked: bool) {
        self.locked.store(locked, Ordering::Release);
    }
}

/// Whether the running process is locked (false where no lock state exists).
pub fn is_locked(app: &tauri::AppHandle) -> bool {
    use tauri::Manager;
    app.try_state::<AppLockService>()
        .is_some_and(|service| service.is_locked())
}

/// Ends the process of a locked profile. Its session was restored but never shown, so the run is marked
/// as ended normally (otherwise the next start would report a crash), and nothing else is saved.
pub fn exit_locked(app: &tauri::AppHandle) {
    use tauri::Manager;
    if let Some(state) = app.try_state::<crate::state::browser_state::BrowserState>() {
        crate::error_log::ignored(
            state.persist_run_checkpoint(crate::database::recovery_repo::RunCheckpoint::CleanExit),
        );
    }
    app.exit(0);
}
