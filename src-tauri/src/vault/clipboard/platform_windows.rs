//! Windows clipboard through the Win32 API. Any thread can use it.
use crate::error_log::{self, Level};
use std::time::Duration;
use tauri::AppHandle;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardSequenceNumber, OpenClipboard,
    RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use zeroize::Zeroizing;

/// The clipboard sequence number right after the vault wrote to it.
pub type Mark = u32;

/// The clipboard keeps its contents after the process exits, so a pending copy is cleared then.
pub const CLEAR_ON_EXIT: bool = true;

const CF_UNICODETEXT: u32 = 13;
/// Formats that ask Windows (and well-behaved clipboard tools) not to keep or sync the copy.
const PRIVATE_FORMATS: [PCWSTR; 3] = [
    w!("ExcludeClipboardContentFromMonitorProcessing"),
    w!("CanIncludeInClipboardHistory"),
    w!("CanUploadToCloudClipboard"),
];
const OPEN_ATTEMPTS: u32 = 10;

pub async fn run<T: Send + 'static>(
    _app: &AppHandle,
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| format!("clipboard task stopped: {error}"))
}

/// The clipboard is open while this lives.
struct Open;

impl Drop for Open {
    fn drop(&mut self) {
        if let Err(error) = unsafe { CloseClipboard() } {
            error_log::report(
                Level::Warning,
                "vault",
                "closing the clipboard failed",
                &error.to_string(),
            );
        }
    }
}

/// Another program may hold the clipboard for a moment, so opening it is retried briefly.
fn open() -> Result<Open, String> {
    let mut last = String::new();
    for _ in 0..OPEN_ATTEMPTS {
        match unsafe { OpenClipboard(None) } {
            Ok(()) => return Ok(Open),
            Err(error) => last = error.to_string(),
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    Err(format!(
        "the clipboard is in use by another program: {last}"
    ))
}

/// GlobalUnlock and GlobalFree report success as a "failure" whose code is S_OK.
fn real_failure(result: windows::core::Result<impl Sized>) -> Option<String> {
    match result {
        Err(error) if error.code().is_err() => Some(error.to_string()),
        _ => None,
    }
}

fn free(handle: HGLOBAL) {
    if let Some(error) = real_failure(unsafe { GlobalFree(Some(handle)) }) {
        error_log::report(
            Level::Warning,
            "vault",
            "freeing clipboard memory failed",
            &error,
        );
    }
}

/// Hands `bytes` to the clipboard under `format`. The clipboard owns the memory once this succeeds.
fn set(format: u32, bytes: &[u8]) -> Result<(), String> {
    let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1)) }
        .map_err(|error| format!("allocating clipboard memory failed: {error}"))?;
    let target = unsafe { GlobalLock(handle) }.cast::<u8>();
    if target.is_null() {
        free(handle);
        return Err("locking clipboard memory failed".into());
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), target, bytes.len()) };
    if let Some(error) = real_failure(unsafe { GlobalUnlock(handle) }) {
        error_log::report(
            Level::Warning,
            "vault",
            "unlocking clipboard memory failed",
            &error,
        );
    }
    match unsafe { SetClipboardData(format, Some(HANDLE(handle.0))) } {
        Ok(_) => Ok(()),
        Err(error) => {
            free(handle);
            Err(format!("setting clipboard data failed: {error}"))
        }
    }
}

pub fn write(text: &str) -> Result<Mark, String> {
    let wide: Zeroizing<Vec<u16>> =
        Zeroizing::new(text.encode_utf16().chain(std::iter::once(0)).collect());
    let bytes: Zeroizing<Vec<u8>> =
        Zeroizing::new(wide.iter().flat_map(|unit| unit.to_ne_bytes()).collect());
    let opened = open()?;
    unsafe { EmptyClipboard() }
        .map_err(|error| format!("emptying the clipboard failed: {error}"))?;
    set(CF_UNICODETEXT, &bytes)?;
    for name in PRIVATE_FORMATS {
        let format = unsafe { RegisterClipboardFormatW(name) };
        let marked = if format == 0 {
            Err("registering a private clipboard format failed".to_string())
        } else {
            set(format, &0u32.to_ne_bytes())
        };
        if let Err(error) = marked {
            // The copy still works; it may only show up in the clipboard history.
            error_log::report(
                Level::Warning,
                "vault",
                "marking the clipboard copy as private failed",
                &error,
            );
        }
    }
    drop(opened);
    Ok(unsafe { GetClipboardSequenceNumber() })
}

/// Empties the clipboard if nothing changed it since `mark`. Returns whether it did.
pub fn clear_if_unchanged(mark: Mark) -> Result<bool, String> {
    if unsafe { GetClipboardSequenceNumber() } != mark {
        return Ok(false);
    }
    let _opened = open()?;
    // Checked again now that no one else can change it.
    if unsafe { GetClipboardSequenceNumber() } != mark {
        return Ok(false);
    }
    unsafe { EmptyClipboard() }
        .map_err(|error| format!("emptying the clipboard failed: {error}"))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uses the real clipboard (and overwrites it), so it only runs when asked for:
    /// `cargo test --lib clipboard -- --ignored`.
    #[test]
    #[ignore]
    fn a_copy_is_cleared_only_while_it_is_still_on_the_clipboard() {
        let mark = write("spark vault clipboard test").unwrap();
        assert!(clear_if_unchanged(mark).unwrap(), "our own copy is cleared");
        let first = write("first").unwrap();
        write("something the user copied later").unwrap();
        assert!(
            !clear_if_unchanged(first).unwrap(),
            "a newer copy is left alone"
        );
    }
}
