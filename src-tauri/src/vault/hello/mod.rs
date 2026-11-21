//! Windows Hello as an unlock method of the vault.
//!
//! The operating system keeps a key pair for the vault. To open it, the vault asks Windows to sign a
//! fixed message; Windows only does that after the user proves who they are (PIN, fingerprint, face).
//! The signature is deterministic and only that key pair can produce it, so it serves as the secret that
//! wraps the data key (see `crypto::unlock_with_hello`). Nothing secret is stored by Spark.
//!
//! Only available on Windows. Every call can wait for the person to answer the Windows prompt, so
//! callers run them off the database thread and off the interface thread.
use zeroize::Zeroizing;

#[cfg(windows)]
#[path = "platform_windows.rs"]
mod platform;
#[cfg(not(windows))]
#[path = "platform_other.rs"]
mod platform;

#[derive(Debug, PartialEq, Eq)]
pub enum HelloError {
    /// The person closed the Windows prompt, or chose another way to sign in.
    Cancelled,
    /// This computer has no Windows Hello set up (or this is not Windows).
    Unavailable,
    /// Windows refused or failed; the detail goes to the error log, never to the interface.
    Failed(String),
}

impl HelloError {
    pub fn code(&self) -> &'static str {
        match self {
            HelloError::Cancelled => "vault.hello_cancelled",
            HelloError::Unavailable => "vault.hello_unavailable",
            HelloError::Failed(_) => "vault.hello_failed",
        }
    }
}

/// True when Windows Hello can be used on this computer.
pub fn is_available() -> bool {
    platform::is_available()
}

/// Creates (or replaces) the vault's key pair and returns the secret it signs. Shows the Windows prompt.
pub fn enroll() -> Result<Zeroizing<Vec<u8>>, HelloError> {
    platform::enroll()
}

/// The secret of the existing key pair. Shows the Windows prompt.
pub fn secret() -> Result<Zeroizing<Vec<u8>>, HelloError> {
    platform::secret()
}

/// Deletes the vault's key pair. Nothing is left to open the Windows Hello copy afterwards.
pub fn forget() -> Result<(), HelloError> {
    platform::forget()
}
