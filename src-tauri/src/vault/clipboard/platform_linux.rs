//! Linux clipboard through GTK (X11 and Wayland). GTK must be used from the main thread.
use sha2::{Digest, Sha256};
use tauri::AppHandle;

/// A SHA-256 digest of what the vault copied, so the text itself is not kept.
pub type Mark = [u8; 32];

/// GTK serves the copy from this process, so it disappears when the browser exits (unless a
/// clipboard manager took a copy, which the vault cannot reach once the process is gone).
pub const CLEAR_ON_EXIT: bool = false;

fn digest(text: &str) -> Mark {
    Sha256::digest(text.as_bytes()).into()
}

fn clipboard() -> gtk::Clipboard {
    gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD)
}

pub async fn run<T: Send + 'static>(
    app: &AppHandle,
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        if sender.send(work()).is_err() {
            crate::error_log::report(
                crate::error_log::Level::Warning,
                "vault",
                "the clipboard result arrived after its caller stopped waiting",
                "",
            );
        }
    })
    .map_err(|error| format!("reaching the main thread failed: {error}"))?;
    receiver
        .await
        .map_err(|_| "the main thread dropped the clipboard task".to_string())
}

pub fn write(text: &str) -> Result<Mark, String> {
    clipboard().set_text(text);
    Ok(digest(text))
}

/// Empties the clipboard if it still holds what the vault copied. Returns whether it did.
pub fn clear_if_unchanged(mark: Mark) -> Result<bool, String> {
    let clipboard = clipboard();
    match clipboard.wait_for_text() {
        Some(current) if digest(current.as_str()) == mark => {
            clipboard.set_text("");
            Ok(true)
        }
        _ => Ok(false),
    }
}
