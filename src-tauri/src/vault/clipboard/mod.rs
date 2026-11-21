//! Copies vault secrets to the system clipboard and empties it again after `CLEAR_AFTER`.
//!
//! The clean-up runs in the backend, so it still happens when Settings is closed or the window has
//! no focus. It only clears the clipboard when it still holds what the vault put there: if the user
//! copied something else in the meantime, that is left alone. The secret itself is not kept for the
//! check (Windows compares the clipboard sequence number, Linux a SHA-256 digest of the text).
//!
//! On Windows the copy is also kept out of the clipboard history and cloud clipboard, and a copy
//! still on the clipboard when the browser exits is cleared (there it outlives the process).
use crate::error_log::{self, Level};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use zeroize::Zeroizing;

#[cfg(target_os = "linux")]
#[path = "platform_linux.rs"]
mod platform;
#[cfg(not(any(windows, target_os = "linux")))]
#[path = "platform_other.rs"]
mod platform;
#[cfg(windows)]
#[path = "platform_windows.rs"]
mod platform;

pub const CLEAR_AFTER: Duration = Duration::from_secs(30);
/// Longest text the vault copies (the longest entry field it lets the interface copy).
pub const MAX_COPY_CHARS: usize = crate::vault::model::MAX_PASSWORD;

#[derive(Default)]
struct State {
    /// Bumped on every copy, so only the clean-up of the latest copy runs.
    generation: u64,
    mark: Option<platform::Mark>,
}

/// What the last vault copy left on the clipboard. Managed as Tauri state.
#[derive(Default)]
pub struct VaultClipboard(Mutex<State>);

impl VaultClipboard {
    fn state(&self) -> MutexGuard<'_, State> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn remember(&self, mark: platform::Mark) -> u64 {
        let mut state = self.state();
        state.generation = state.generation.wrapping_add(1);
        state.mark = Some(mark);
        state.generation
    }

    /// The mark of the copy to clear: the latest one, and only if `generation` is still the latest.
    fn take(&self, generation: Option<u64>) -> Option<platform::Mark> {
        let mut state = self.state();
        match generation {
            Some(generation) if generation != state.generation => None,
            _ => state.mark.take(),
        }
    }
}

fn report_clear(result: Result<bool, String>, place: &str) {
    if let Err(error) = result {
        error_log::report(
            Level::Error,
            "vault",
            &format!("clearing the clipboard {place} failed"),
            &error,
        );
    }
}

/// Puts `text` on the clipboard and schedules its removal.
pub async fn copy(app: &AppHandle, text: Zeroizing<String>) -> Result<(), String> {
    if text.chars().count() > MAX_COPY_CHARS {
        return Err("vault.invalid_entry".into());
    }
    let mark = platform::run(app, move || platform::write(&text))
        .await
        .and_then(|written| written)
        .map_err(|error| {
            error_log::report(
                Level::Error,
                "vault",
                "copying to the clipboard failed",
                &error,
            );
            "vault.clipboard_failed".to_string()
        })?;
    let generation = app.state::<VaultClipboard>().remember(mark);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(CLEAR_AFTER).await;
        let Some(mark) = app.state::<VaultClipboard>().take(Some(generation)) else {
            return;
        };
        let cleared = platform::run(&app, move || platform::clear_if_unchanged(mark))
            .await
            .and_then(|cleared| cleared);
        report_clear(cleared, "after the copy timed out");
    });
    Ok(())
}

/// Called on the main thread when the browser exits.
pub fn clear_on_exit(app: &AppHandle) {
    if !platform::CLEAR_ON_EXIT {
        return;
    }
    if let Some(mark) = app
        .try_state::<VaultClipboard>()
        .and_then(|clipboard| clipboard.take(None))
    {
        report_clear(
            platform::clear_if_unchanged(mark),
            "when the browser exited",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_latest_copy_is_cleared_and_only_once() {
        let clipboard = VaultClipboard::default();
        let first = clipboard.remember(platform::Mark::default());
        let second = clipboard.remember(platform::Mark::default());
        assert!(
            clipboard.take(Some(first)).is_none(),
            "a newer copy restarts the wait"
        );
        assert!(clipboard.take(Some(second)).is_some());
        assert!(
            clipboard.take(Some(second)).is_none(),
            "nothing left to clear"
        );
        clipboard.remember(platform::Mark::default());
        assert!(
            clipboard.take(None).is_some(),
            "exit clears whatever is pending"
        );
    }
}
