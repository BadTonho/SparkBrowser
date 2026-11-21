//! Windows Hello through `KeyCredentialManager`.
use super::HelloError;
use windows::Security::Credentials::{
    KeyCredential, KeyCredentialCreationOption, KeyCredentialManager, KeyCredentialStatus,
};
use windows::Security::Cryptography::CryptographicBuffer;
use windows::Win32::Foundation::NTE_NO_KEY;
use windows_core::HSTRING;
use zeroize::Zeroizing;

/// Name of the key pair in the user's Windows profile.
const CREDENTIAL: &str = "SparkBrowserPasswordVault";
/// What Windows signs. It never changes, so the (RSA PKCS#1) signature never changes either.
const CHALLENGE: &[u8] = b"spark-vault-hello-challenge-v1";

fn failed(what: &str, error: impl std::fmt::Display) -> HelloError {
    HelloError::Failed(format!("{what}: {error}"))
}

fn status_error(what: &str, status: KeyCredentialStatus) -> HelloError {
    match status {
        KeyCredentialStatus::UserCanceled | KeyCredentialStatus::UserPrefersPassword => {
            HelloError::Cancelled
        }
        KeyCredentialStatus::NotFound => {
            HelloError::Failed(format!("{what}: the Windows Hello key was not found"))
        }
        KeyCredentialStatus::SecurityDeviceLocked => {
            HelloError::Failed(format!("{what}: the security device is locked"))
        }
        other => HelloError::Failed(format!("{what}: Windows answered with status {}", other.0)),
    }
}

pub fn is_available() -> bool {
    KeyCredentialManager::IsSupportedAsync()
        .and_then(|operation| operation.join())
        .unwrap_or(false)
}

fn sign(credential: &KeyCredential) -> Result<Zeroizing<Vec<u8>>, HelloError> {
    let challenge = CryptographicBuffer::CreateFromByteArray(CHALLENGE)
        .map_err(|e| failed("preparing the challenge", e))?;
    let result = credential
        .RequestSignAsync(&challenge)
        .and_then(|operation| operation.join())
        .map_err(|e| failed("asking Windows to sign", e))?;
    let status = result
        .Status()
        .map_err(|e| failed("reading the signing status", e))?;
    if status != KeyCredentialStatus::Success {
        return Err(status_error("signing with Windows Hello", status));
    }
    let signature = result
        .Result()
        .map_err(|e| failed("reading the signature", e))?;
    let mut bytes = windows_core::Array::<u8>::new();
    CryptographicBuffer::CopyToByteArray(&signature, &mut bytes)
        .map_err(|e| failed("copying the signature", e))?;
    Ok(Zeroizing::new(bytes.to_vec()))
}

pub fn enroll() -> Result<Zeroizing<Vec<u8>>, HelloError> {
    let result = KeyCredentialManager::RequestCreateAsync(
        &HSTRING::from(CREDENTIAL),
        KeyCredentialCreationOption::ReplaceExisting,
    )
    .and_then(|operation| operation.join())
    .map_err(|e| failed("creating the Windows Hello key", e))?;
    let status = result
        .Status()
        .map_err(|e| failed("reading the creation status", e))?;
    if status != KeyCredentialStatus::Success {
        return Err(status_error("creating the Windows Hello key", status));
    }
    sign(
        &result
            .Credential()
            .map_err(|e| failed("reading the new key", e))?,
    )
}

pub fn secret() -> Result<Zeroizing<Vec<u8>>, HelloError> {
    let result = KeyCredentialManager::OpenAsync(&HSTRING::from(CREDENTIAL))
        .and_then(|operation| operation.join())
        .map_err(|e| failed("opening the Windows Hello key", e))?;
    let status = result
        .Status()
        .map_err(|e| failed("reading the open status", e))?;
    if status != KeyCredentialStatus::Success {
        return Err(status_error("opening the Windows Hello key", status));
    }
    sign(
        &result
            .Credential()
            .map_err(|e| failed("reading the key", e))?,
    )
}

pub fn forget() -> Result<(), HelloError> {
    match KeyCredentialManager::DeleteAsync(&HSTRING::from(CREDENTIAL))
        .and_then(|operation| operation.join())
    {
        // The key may already be absent when resetting a password-only vault.
        Err(error) if error.code() == NTE_NO_KEY => Ok(()),
        result => result.map_err(|e| failed("deleting the Windows Hello key", e)),
    }
}
