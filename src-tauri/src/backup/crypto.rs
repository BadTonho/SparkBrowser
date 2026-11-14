//! Password-protected backup envelope: Argon2id key derivation + AES-256-GCM.
//!
//! The envelope is itself a JSON document so encrypted backups keep the
//! `.spark-backup.json` extension. KDF parameters travel inside the envelope
//! (bounded on read) so they can be strengthened later without breaking old files.
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};

pub const ENCRYPTED_FORMAT: &str = "spark-backup-encrypted";
/// Envelope tag written before the app was renamed; still read so older encrypted backups open.
pub const LEGACY_ENCRYPTED_FORMAT: &str = "veloce-backup-encrypted";
pub fn is_encrypted_format(tag: &str) -> bool {
    tag == ENCRYPTED_FORMAT || tag == LEGACY_ENCRYPTED_FORMAT
}
const ENVELOPE_VERSION: u32 = 1;
const KDF_NAME: &str = "argon2id";
const CIPHER_NAME: &str = "aes-256-gcm";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
/// AES-GCM associated data. It is part of every encrypted file's authentication, so it keeps the
/// value it had before the rename: changing it would make existing encrypted backups undecryptable.
const AAD: &[u8] = b"veloce-backup-encrypted:1";

pub const MIN_PASSWORD_CHARS: usize = 8;
pub const MAX_PASSWORD_CHARS: usize = 256;

// Debug/test builds are unoptimized, so keep the work factor small there. The
// parameters are stored in every envelope, so release builds still read them.
#[cfg(not(test))]
const MEMORY_KIB: u32 = 64 * 1024;
#[cfg(test)]
const MEMORY_KIB: u32 = 256;
#[cfg(not(test))]
const ITERATIONS: u32 = 3;
#[cfg(test)]
const ITERATIONS: u32 = 1;
const PARALLELISM: u32 = 1;

// Upper bounds protect against a hostile file demanding unbounded work/memory.
const MAX_MEMORY_KIB: u32 = 256 * 1024;
const MAX_ITERATIONS: u32 = 10;
const MAX_PARALLELISM: u32 = 4;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Kdf {
    algorithm: String,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
    salt: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    version: u32,
    kdf: Kdf,
    cipher: String,
    nonce: String,
    ciphertext: String,
}

#[derive(Deserialize)]
struct Probe {
    format: String,
}

/// True when the bytes are a JSON document declaring the encrypted format.
pub fn is_encrypted(bytes: &[u8]) -> bool {
    serde_json::from_slice::<Probe>(bytes).is_ok_and(|probe| is_encrypted_format(&probe.format))
}

pub fn validate_password(password: &str) -> Result<(), String> {
    let chars = password.chars().count();
    if !(MIN_PASSWORD_CHARS..=MAX_PASSWORD_CHARS).contains(&chars) {
        return Err("backup.password_invalid".into());
    }
    Ok(())
}

fn derive_key(
    password: &str,
    salt: &[u8],
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<[u8; KEY_LEN], String> {
    let params = Params::new(memory_kib, iterations, parallelism, Some(KEY_LEN))
        .map_err(|_| "backup.invalid_document: invalid KDF parameters".to_string())?;
    let mut key = [0u8; KEY_LEN];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|_| "backup.crypto_failed".to_string())?;
    Ok(key)
}

pub fn encrypt(plain: &[u8], password: &str) -> Result<Vec<u8>, String> {
    validate_password(password)?;
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut salt).map_err(|_| "backup.crypto_failed".to_string())?;
    getrandom::fill(&mut nonce).map_err(|_| "backup.crypto_failed".to_string())?;
    let key = derive_key(password, &salt, MEMORY_KIB, ITERATIONS, PARALLELISM)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "backup.crypto_failed".to_string())?;
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: plain,
                aad: AAD,
            },
        )
        .map_err(|_| "backup.crypto_failed".to_string())?;
    let envelope = Envelope {
        format: ENCRYPTED_FORMAT.into(),
        version: ENVELOPE_VERSION,
        kdf: Kdf {
            algorithm: KDF_NAME.into(),
            memory_kib: MEMORY_KIB,
            iterations: ITERATIONS,
            parallelism: PARALLELISM,
            salt: B64.encode(salt),
        },
        cipher: CIPHER_NAME.into(),
        nonce: B64.encode(nonce),
        ciphertext: B64.encode(ciphertext),
    };
    serde_json::to_vec_pretty(&envelope).map_err(|e| e.to_string())
}

pub fn decrypt(bytes: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let envelope: Envelope =
        serde_json::from_slice(bytes).map_err(|e| format!("backup.invalid_document: {e}"))?;
    if !is_encrypted_format(&envelope.format)
        || envelope.version != ENVELOPE_VERSION
        || envelope.kdf.algorithm != KDF_NAME
        || envelope.cipher != CIPHER_NAME
    {
        return Err("backup.unsupported_version".into());
    }
    let kdf = &envelope.kdf;
    if kdf.memory_kib == 0
        || kdf.memory_kib > MAX_MEMORY_KIB
        || kdf.iterations == 0
        || kdf.iterations > MAX_ITERATIONS
        || kdf.parallelism == 0
        || kdf.parallelism > MAX_PARALLELISM
    {
        return Err("backup.invalid_document: KDF parameters out of range".into());
    }
    let invalid = || "backup.invalid_document: malformed encrypted payload".to_string();
    let salt = B64.decode(&kdf.salt).map_err(|_| invalid())?;
    let nonce = B64.decode(&envelope.nonce).map_err(|_| invalid())?;
    let ciphertext = B64.decode(&envelope.ciphertext).map_err(|_| invalid())?;
    if salt.len() < 8 || nonce.len() != NONCE_LEN {
        return Err(invalid());
    }
    let key = derive_key(
        password,
        &salt,
        kdf.memory_kib,
        kdf.iterations,
        kdf.parallelism,
    )?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "backup.crypto_failed".to_string())?;
    cipher
        .decrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &ciphertext,
                aad: AAD,
            },
        )
        // GCM authentication fails for both a wrong password and tampering.
        .map_err(|_| "backup.password_wrong".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_detection() {
        let sealed = encrypt(b"{\"hello\":1}", "correct horse").unwrap();
        assert!(is_encrypted(&sealed));
        assert!(!is_encrypted(b"{\"format\":\"spark-backup\"}"));
        assert!(!is_encrypted(b"not json"));
        assert!(!String::from_utf8_lossy(&sealed).contains("hello"));
        assert_eq!(decrypt(&sealed, "correct horse").unwrap(), b"{\"hello\":1}");
    }

    #[test]
    fn wrong_password_and_tampering_are_rejected() {
        let sealed = encrypt(b"secret", "correct horse").unwrap();
        assert_eq!(
            decrypt(&sealed, "wrong horse!").unwrap_err(),
            "backup.password_wrong"
        );
        let mut envelope: Envelope = serde_json::from_slice(&sealed).unwrap();
        let mut raw = B64.decode(&envelope.ciphertext).unwrap();
        raw[0] ^= 1;
        envelope.ciphertext = B64.encode(raw);
        let tampered = serde_json::to_vec(&envelope).unwrap();
        assert_eq!(
            decrypt(&tampered, "correct horse").unwrap_err(),
            "backup.password_wrong"
        );
    }

    #[test]
    fn each_encryption_uses_fresh_salt_and_nonce() {
        let a = encrypt(b"same", "correct horse").unwrap();
        let b = encrypt(b"same", "correct horse").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn password_length_and_hostile_kdf_parameters_are_bounded() {
        assert_eq!(
            encrypt(b"x", "short").unwrap_err(),
            "backup.password_invalid"
        );
        let sealed = encrypt(b"x", "correct horse").unwrap();
        let mut envelope: Envelope = serde_json::from_slice(&sealed).unwrap();
        envelope.kdf.memory_kib = u32::MAX;
        let hostile = serde_json::to_vec(&envelope).unwrap();
        assert!(decrypt(&hostile, "correct horse")
            .unwrap_err()
            .starts_with("backup.invalid_document"));
    }
}
