//! Two-factor codes for the logins in the vault. The code is computed here, from the secret of a
//! login the person already opened (which asked who they are) or is typing in the editor.
use crate::error_log::{self, Level};
use crate::vault::qr;
use crate::vault::service::VaultService;
use crate::vault::totp::{self, TotpCode};
use std::io::Read;
use std::time::Instant;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroizing;

/// The six-digit code for a 2FA secret (an `otpauth://` link or a setup key) and the seconds left
/// before it changes. It needs an open vault and never writes the secret to the log.
#[tauri::command]
pub async fn vault_totp_code(
    app: AppHandle,
    service: State<'_, VaultService>,
    secret: String,
) -> Result<TotpCode, String> {
    super::vault::allowed_read(&app)?;
    service.key(Instant::now())?;
    let secret = Zeroizing::new(secret);
    totp::code_now(&secret).inspect_err(|code| {
        error_log::report(
            Level::Warning,
            "vault",
            "a 2FA code could not be computed: the secret is not valid",
            code,
        );
    })
}

/// Reads a picture file for the 2FA QR code. The size is checked before the file is read.
fn read_picture(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let describe = |error: std::io::Error| {
        error_log::report(
            Level::Error,
            "vault",
            "the 2FA QR picture could not be read",
            &error.to_string(),
        );
        "vault.qr_unreadable".to_string()
    };
    let file = std::fs::File::open(path).map_err(describe)?;
    if file.metadata().map_err(describe)?.len() > qr::MAX_IMAGE_BYTES as u64 {
        error_log::report(
            Level::Warning,
            "vault",
            "the 2FA QR picture is too large",
            "",
        );
        return Err("vault.qr_unreadable".into());
    }
    let mut bytes = Vec::new();
    file.take(qr::MAX_IMAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(describe)?;
    Ok(bytes)
}

/// Lets the person pick a picture of the 2FA QR code (a screenshot, say) and reads it. The file is
/// chosen and read here, so it never passes through the interface; only a valid 2FA secret comes
/// back, and what any other QR code says never does. `None` means the dialog was cancelled.
/// It needs an open vault and never writes the picture or the secret to the log.
#[tauri::command]
pub async fn vault_scan_qr(
    app: AppHandle,
    service: State<'_, VaultService>,
) -> Result<Option<String>, String> {
    super::vault::allowed(&app)?;
    service.key(Instant::now())?;
    tauri::async_runtime::spawn_blocking(move || {
        let Some(path) = app
            .dialog()
            .file()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = path.into_path().map_err(|e| {
            error_log::report(
                Level::Error,
                "vault",
                "the chosen 2FA QR picture has no usable path",
                &e.to_string(),
            );
            "vault.qr_unreadable".to_string()
        })?;
        let bytes = read_picture(&path)?;
        qr::read(&bytes).map(Some).inspect_err(|code| {
            error_log::report(
                Level::Info,
                "vault",
                "a 2FA QR picture could not be used",
                code,
            );
        })
    })
    .await
    .map_err(|error| {
        error_log::report(
            Level::Error,
            "vault",
            "reading a 2FA QR picture stopped unexpectedly",
            &error.to_string(),
        );
        "vault.qr_unreadable".to_string()
    })?
}
