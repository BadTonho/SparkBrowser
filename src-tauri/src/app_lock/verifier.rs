//! What the app lock stores to recognise a secret without being able to give it back: a hash of the
//! secret stretched with Argon2id (the password), or of the signature Windows Hello produced (that
//! signature is already a long secret, so it needs no stretching). Nothing here encrypts anything:
//! the lock only decides whether Spark opens.
use crate::vault::crypto::{self, CryptoError, Kdf};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const PASSWORD_LABEL: &[u8] = b"spark-app-lock-password-v1";
const HELLO_LABEL: &[u8] = b"spark-app-lock-hello-v1";

/// How a password is recognised: the work factor and salt it was stretched with, and the hash.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Verifier {
    kdf: Kdf,
    salt: String,
    hash: String,
}

fn fingerprint(label: &[u8], bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(label);
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Compares in time that does not depend on where the texts differ.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |difference, (x, y)| difference | (x ^ y))
            == 0
}

fn code(error: CryptoError) -> String {
    match error {
        CryptoError::InvalidPassword => "lock.password_invalid".into(),
        CryptoError::Corrupt => "lock.corrupt".into(),
        _ => "lock.crypto_failed".into(),
    }
}

impl Verifier {
    /// Makes the verifier of a new password (8 to 256 characters, like the vault's).
    pub fn new(password: &str) -> Result<Self, String> {
        crypto::validate_password(password).map_err(code)?;
        let kdf = crypto::default_kdf();
        let salt = crypto::random_salt().map_err(code)?;
        let key = crypto::derive_secret(password.as_bytes(), &salt, &kdf).map_err(code)?;
        Ok(Self {
            kdf,
            salt: base64::engine::general_purpose::STANDARD.encode(salt),
            hash: fingerprint(PASSWORD_LABEL, key.as_ref()),
        })
    }

    /// Whether `password` is the one this was made from. A damaged record is an error, not a "no".
    pub fn matches(&self, password: &str) -> Result<bool, String> {
        let salt = base64::engine::general_purpose::STANDARD
            .decode(&self.salt)
            .map_err(|_| "lock.corrupt".to_string())?;
        let key = crypto::derive_secret(password.as_bytes(), &salt, &self.kdf).map_err(code)?;
        Ok(same(&self.hash, &fingerprint(PASSWORD_LABEL, key.as_ref())))
    }
}

/// The record of a Windows Hello secret: a hash of the signature Windows produced.
pub fn hello_fingerprint(secret: &[u8]) -> String {
    fingerprint(HELLO_LABEL, secret)
}

pub fn hello_matches(stored: &str, secret: &[u8]) -> bool {
    same(stored, &hello_fingerprint(secret))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_is_recognised_and_nothing_readable_is_stored() {
        let verifier = Verifier::new("correct horse battery").unwrap();
        assert!(verifier.matches("correct horse battery").unwrap());
        assert!(!verifier.matches("correct horse batterz").unwrap());
        assert!(!verifier.matches("").unwrap());
        let stored = serde_json::to_string(&verifier).unwrap();
        assert!(!stored.contains("correct") && !stored.contains("horse"));
        // The same password gets a new salt every time.
        assert_ne!(verifier, Verifier::new("correct horse battery").unwrap());
    }

    #[test]
    fn short_and_long_passwords_are_refused_with_a_lock_code() {
        assert_eq!(Verifier::new("short").unwrap_err(), "lock.password_invalid");
        assert_eq!(
            Verifier::new(&"a".repeat(257)).unwrap_err(),
            "lock.password_invalid"
        );
        assert!(Verifier::new(&"a".repeat(256)).is_ok());
    }

    #[test]
    fn a_damaged_record_is_an_error_and_a_tampered_work_factor_is_bounded() {
        let mut verifier = Verifier::new("correct horse battery").unwrap();
        verifier.salt = "!!not base64!!".into();
        assert_eq!(verifier.matches("x").unwrap_err(), "lock.corrupt");
        let mut greedy = Verifier::new("correct horse battery").unwrap();
        greedy.kdf.memory_kib = u32::MAX;
        assert!(
            greedy.matches("x").is_err(),
            "a tampered record cannot demand unbounded memory"
        );
    }

    #[test]
    fn a_windows_hello_secret_is_recognised_by_its_hash() {
        let stored = hello_fingerprint(b"signature bytes");
        assert!(hello_matches(&stored, b"signature bytes"));
        assert!(!hello_matches(&stored, b"other bytes"));
        assert!(!stored.contains("signature"));
        assert_ne!(stored, hello_fingerprint(b"other bytes"));
    }

    #[test]
    fn the_comparison_rejects_different_lengths() {
        assert!(same("abc", "abc"));
        assert!(!same("abc", "abd"));
        assert!(!same("abc", "abcd"));
        assert!(!same("", "a"));
    }
}
