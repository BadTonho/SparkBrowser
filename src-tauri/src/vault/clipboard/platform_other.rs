//! Platforms Spark does not target yet: copying from the vault is refused instead of half working.
use tauri::AppHandle;

pub type Mark = ();
pub const CLEAR_ON_EXIT: bool = false;

pub async fn run<T: Send + 'static>(
    _app: &AppHandle,
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    Ok(work())
}

pub fn write(_text: &str) -> Result<Mark, String> {
    Err("copying from the vault is not supported on this platform".into())
}

pub fn clear_if_unchanged(_mark: Mark) -> Result<bool, String> {
    Ok(false)
}
