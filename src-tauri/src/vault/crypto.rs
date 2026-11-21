//! Cryptography of the password vault.
//!
//! A random 32-byte data key (DEK) encrypts every entry with AES-256-GCM. The DEK is stored twice,
//! each copy wrapped by a key derived with Argon2id: one from the master password and one from the
//! recovery key shown once at creation. Nothing here touches the disk or logs; callers get opaque
//! errors that never contain secret material.
//!
//! This code has not been independently audited (see `docs/en/security/account-containers-vault-review.md`).
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub const HEADER_VERSION: u32 = 1;
const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const RECOVERY_BYTES: usize = 20;
pub const MIN_PASSWORD_CHARS: usize = 8;
pub const MAX_PASSWORD_CHARS: usize = 256;

// Debug and test builds are unoptimized, so the work factor is small there. The parameters travel in
// the header, so release builds still read what a smaller build wrote.
#[cfg(not(test))]
const MEMORY_KIB: u32 = 64 * 1024;
#[cfg(test)]
const MEMORY_KIB: u32 = 256;
#[cfg(not(test))]
const ITERATIONS: u32 = 3;
#[cfg(test)]
const ITERATIONS: u32 = 1;
const PARALLELISM: u32 = 1;
// A tampered header must not be able to demand unbounded work or memory.
const MAX_MEMORY_KIB: u32 = 256 * 1024;
const MAX_ITERATIONS: u32 = 10;
const MAX_PARALLELISM: u32 = 4;

/// The data key. It is wiped from memory when dropped.
#[derive(Clone)]
pub struct Key(Zeroizing<[u8; KEY_LEN]>);

// Never prints the key, even by accident in a log line or a failed assertion.
impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Key(<redacted>)")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Kdf {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Wrapped {
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

/// Everything needed to open the vault again, none of it secret on its own.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub version: u32,
    pub kdf: Kdf,
    pub password: Wrapped,
    pub recovery: Wrapped,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CryptoError {
    /// The password or recovery key does not open the vault (or the header was altered).
    WrongSecret,
    /// The stored data is damaged, altered or from a format this version cannot read.
    Corrupt,
    /// The system could not provide the randomness or memory needed.
    Unavailable,
    InvalidPassword,
}

impl CryptoError {
    pub fn code(&self) -> &'static str {
        match self {
            CryptoError::WrongSecret => "vault.wrong_secret",
            CryptoError::Corrupt => "vault.corrupt",
            CryptoError::Unavailable => "vault.crypto_failed",
            CryptoError::InvalidPassword => "vault.password_invalid",
        }
    }
}

pub fn validate_password(password: &str) -> Result<(), CryptoError> {
    let chars = password.chars().count();
    if (MIN_PASSWORD_CHARS..=MAX_PASSWORD_CHARS).contains(&chars) {
        Ok(())
    } else {
        Err(CryptoError::InvalidPassword)
    }
}

fn random<const N: usize>() -> Result<[u8; N], CryptoError> {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes).map_err(|_| CryptoError::Unavailable)?;
    Ok(bytes)
}

fn derive(secret: &[u8], salt: &[u8], kdf: &Kdf) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    if kdf.memory_kib == 0
        || kdf.memory_kib > MAX_MEMORY_KIB
        || kdf.iterations == 0
        || kdf.iterations > MAX_ITERATIONS
        || kdf.parallelism == 0
        || kdf.parallelism > MAX_PARALLELISM
        || salt.len() < 8
    {
        return Err(CryptoError::Corrupt);
    }
    let params = Params::new(kdf.memory_kib, kdf.iterations, kdf.parallelism, Some(KEY_LEN))
        .map_err(|_| CryptoError::Corrupt)?;
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(secret, salt, key.as_mut())
        .map_err(|_| CryptoError::Unavailable)?;
    Ok(key)
}

fn seal(key: &[u8], aad: &[u8], plain: &[u8]) -> Result<([u8; NONCE_LEN], Vec<u8>), CryptoError> {
    let nonce = random::<NONCE_LEN>()?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::Unavailable)?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), Payload { msg: plain, aad })
        .map_err(|_| CryptoError::Unavailable)?;
    Ok((nonce, ciphertext))
}

fn open(key: &[u8], aad: &[u8], nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    if nonce.len() != NONCE_LEN {
        return Err(());
    }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| ())?;
    cipher
        .decrypt(Nonce::from_slice(nonce), Payload { msg: ciphertext, aad })
        .map_err(|_| ())
}

fn wrap_aad(kind: &str) -> Vec<u8> {
    format!("spark-vault-dek-{kind}:{HEADER_VERSION}").into_bytes()
}

fn wrap(kind: &str, secret: &[u8], dek: &Key, kdf: &Kdf) -> Result<Wrapped, CryptoError> {
    let salt = random::<SALT_LEN>()?;
    let kek = derive(secret, &salt, kdf)?;
    let (nonce, ciphertext) = seal(kek.as_ref(), &wrap_aad(kind), dek.0.as_ref())?;
    Ok(Wrapped {
        salt: B64.encode(salt),
        nonce: B64.encode(nonce),
        ciphertext: B64.encode(ciphertext),
    })
}

fn unwrap(kind: &str, secret: &[u8], wrapped: &Wrapped, kdf: &Kdf) -> Result<Key, CryptoError> {
    let decode = |text: &str| B64.decode(text).map_err(|_| CryptoError::Corrupt);
    let (salt, nonce, ciphertext) = (
        decode(&wrapped.salt)?,
        decode(&wrapped.nonce)?,
        decode(&wrapped.ciphertext)?,
    );
    let kek = derive(secret, &salt, kdf)?;
    let plain = Zeroizing::new(
        open(kek.as_ref(), &wrap_aad(kind), &nonce, &ciphertext)
            .map_err(|_| CryptoError::WrongSecret)?,
    );
    let bytes: [u8; KEY_LEN] = plain
        .as_slice()
        .try_into()
        .map_err(|_| CryptoError::Corrupt)?;
    Ok(Key(Zeroizing::new(bytes)))
}

// RFC 4648 base32 without padding, in groups of four, so the recovery key is easy to copy by hand.
const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn encode_recovery(bytes: &[u8; RECOVERY_BYTES]) -> String {
    let mut out = String::new();
    let (mut buffer, mut bits) = (0u32, 0u32);
    for byte in bytes {
        buffer = (buffer << 8) | u32::from(*byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(ALPHABET[((buffer >> bits) & 31) as usize] as char);
        }
        buffer &= (1 << bits) - 1;
    }
    out.as_bytes()
        .chunks(4)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("-")
}

fn decode_recovery(text: &str) -> Option<Zeroizing<Vec<u8>>> {
    let cleaned: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if cleaned.len() != RECOVERY_BYTES * 8 / 5 {
        return None;
    }
    let mut out = Zeroizing::new(Vec::with_capacity(RECOVERY_BYTES));
    let (mut buffer, mut bits) = (0u32, 0u32);
    for c in cleaned.bytes() {
        let value = ALPHABET.iter().position(|a| *a == c)? as u32;
        buffer = (buffer << 5) | value;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 255) as u8);
        }
        buffer &= (1 << bits) - 1;
    }
    (out.len() == RECOVERY_BYTES).then_some(out)
}

/// Creates a vault: returns its header, the open data key and the recovery key (shown to the user once).
pub fn create(master_password: &str) -> Result<(Header, Key, String), CryptoError> {
    validate_password(master_password)?;
    let dek = Key(Zeroizing::new(random::<KEY_LEN>()?));
    let recovery = Zeroizing::new(random::<RECOVERY_BYTES>()?);
    let kdf = Kdf {
        memory_kib: MEMORY_KIB,
        iterations: ITERATIONS,
        parallelism: PARALLELISM,
    };
    let header = Header {
        version: HEADER_VERSION,
        password: wrap("password", master_password.as_bytes(), &dek, &kdf)?,
        recovery: wrap("recovery", recovery.as_ref(), &dek, &kdf)?,
        kdf,
    };
    Ok((header, dek, encode_recovery(&recovery)))
}

fn check(header: &Header) -> Result<(), CryptoError> {
    if header.version == HEADER_VERSION {
        Ok(())
    } else {
        Err(CryptoError::Corrupt)
    }
}

pub fn unlock_with_password(header: &Header, password: &str) -> Result<Key, CryptoError> {
    check(header)?;
    unwrap("password", password.as_bytes(), &header.password, &header.kdf)
}

pub fn unlock_with_recovery(header: &Header, recovery_key: &str) -> Result<Key, CryptoError> {
    check(header)?;
    let bytes = decode_recovery(recovery_key).ok_or(CryptoError::WrongSecret)?;
    unwrap("recovery", bytes.as_ref(), &header.recovery, &header.kdf)
}

/// A new header whose password copy opens with `new_password`. The data key, the recovery copy and
/// therefore every stored entry stay the same.
pub fn change_password(
    header: &Header,
    dek: &Key,
    new_password: &str,
) -> Result<Header, CryptoError> {
    validate_password(new_password)?;
    let kdf = header.kdf.clone();
    Ok(Header {
        version: header.version,
        password: wrap("password", new_password.as_bytes(), dek, &kdf)?,
        recovery: header.recovery.clone(),
        kdf,
    })
}

fn entry_aad(id: &str) -> Vec<u8> {
    format!("spark-vault-entry:{HEADER_VERSION}:{id}").into_bytes()
}

/// Encrypts one entry. The id is bound to the ciphertext so a blob cannot be moved to another entry.
pub fn seal_entry(dek: &Key, id: &str, plain: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let (nonce, ciphertext) = seal(dek.0.as_ref(), &entry_aad(id), plain)?;
    let mut blob = nonce.to_vec();
    blob.extend_from_slice(&ciphertext);
    Ok(blob)
}

pub fn open_entry(dek: &Key, id: &str, blob: &[u8]) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if blob.len() <= NONCE_LEN {
        return Err(CryptoError::Corrupt);
    }
    let (nonce, ciphertext) = blob.split_at(NONCE_LEN);
    open(dek.0.as_ref(), &entry_aad(id), nonce, ciphertext)
        .map(Zeroizing::new)
        .map_err(|_| CryptoError::Corrupt)
}

#[cfg(test)]
mod tests {
    use super::*;
    const PASSWORD: &str = "correct horse battery";

    #[test]
    fn a_new_vault_opens_with_the_password_and_with_the_recovery_key() {
        let (header, dek, recovery) = create(PASSWORD).unwrap();
        assert_eq!(
            unlock_with_password(&header, PASSWORD).unwrap().0.as_ref(),
            dek.0.as_ref()
        );
        assert_eq!(
            unlock_with_recovery(&header, &recovery).unwrap().0.as_ref(),
            dek.0.as_ref()
        );
        // Spaces, case and dashes do not matter when the user types the recovery key back.
        let typed = recovery.replace('-', " ").to_lowercase();
        assert!(unlock_with_recovery(&header, &typed).is_ok());
    }

    #[test]
    fn wrong_secrets_never_open_the_vault() {
        let (header, _, recovery) = create(PASSWORD).unwrap();
        assert_eq!(
            unlock_with_password(&header, "not the password").unwrap_err(),
            CryptoError::WrongSecret
        );
        assert_eq!(
            unlock_with_recovery(&header, "AAAA-AAAA-AAAA-AAAA-AAAA-AAAA-AAAA-AAAA").unwrap_err(),
            CryptoError::WrongSecret
        );
        assert_eq!(
            unlock_with_recovery(&header, "short").unwrap_err(),
            CryptoError::WrongSecret
        );
        // The password is not accepted as the recovery key, nor the other way around.
        assert!(unlock_with_recovery(&header, PASSWORD).is_err());
        assert!(unlock_with_password(&header, &recovery).is_err());
    }

    #[test]
    fn the_recovery_key_has_the_expected_shape_and_is_random() {
        let (_, _, one) = create(PASSWORD).unwrap();
        let (_, _, two) = create(PASSWORD).unwrap();
        assert_eq!(one.len(), 32 + 7);
        assert!(one.split('-').all(|group| group.len() == 4));
        assert_ne!(one, two);
    }

    #[test]
    fn password_length_is_bounded() {
        assert_eq!(create("short").err(), Some(CryptoError::InvalidPassword));
        assert_eq!(
            create(&"a".repeat(MAX_PASSWORD_CHARS + 1)).err(),
            Some(CryptoError::InvalidPassword)
        );
        assert!(create(&"a".repeat(MAX_PASSWORD_CHARS)).is_ok());
    }

    #[test]
    fn entries_round_trip_and_use_a_new_nonce_each_time() {
        let (_, dek, _) = create(PASSWORD).unwrap();
        let a = seal_entry(&dek, "id1", b"secret").unwrap();
        let b = seal_entry(&dek, "id1", b"secret").unwrap();
        assert_ne!(a, b);
        assert_ne!(&a[..NONCE_LEN], &b[..NONCE_LEN]);
        assert!(!a.windows(6).any(|w| w == b"secret"));
        assert_eq!(open_entry(&dek, "id1", &a).unwrap().as_slice(), b"secret");
    }

    #[test]
    fn tampering_or_moving_an_entry_is_detected() {
        let (_, dek, _) = create(PASSWORD).unwrap();
        let blob = seal_entry(&dek, "id1", b"secret").unwrap();
        assert_eq!(
            open_entry(&dek, "id2", &blob).unwrap_err(),
            CryptoError::Corrupt
        );
        let mut altered = blob.clone();
        *altered.last_mut().unwrap() ^= 1;
        assert_eq!(
            open_entry(&dek, "id1", &altered).unwrap_err(),
            CryptoError::Corrupt
        );
        assert_eq!(
            open_entry(&dek, "id1", &blob[..5]).unwrap_err(),
            CryptoError::Corrupt
        );
        let (_, other, _) = create(PASSWORD).unwrap();
        assert!(open_entry(&other, "id1", &blob).is_err());
    }

    #[test]
    fn changing_the_password_keeps_the_key_the_recovery_and_the_entries() {
        let (header, dek, recovery) = create(PASSWORD).unwrap();
        let blob = seal_entry(&dek, "id1", b"secret").unwrap();
        let changed = change_password(&header, &dek, "a brand new password").unwrap();
        assert!(unlock_with_password(&changed, PASSWORD).is_err());
        let reopened = unlock_with_password(&changed, "a brand new password").unwrap();
        assert_eq!(
            open_entry(&reopened, "id1", &blob).unwrap().as_slice(),
            b"secret"
        );
        assert!(unlock_with_recovery(&changed, &recovery).is_ok());
        assert_eq!(
            change_password(&header, &dek, "short").unwrap_err(),
            CryptoError::InvalidPassword
        );
    }

    #[test]
    fn a_header_cannot_demand_unbounded_work() {
        let (mut header, _, _) = create(PASSWORD).unwrap();
        header.kdf.memory_kib = MAX_MEMORY_KIB + 1;
        assert_eq!(
            unlock_with_password(&header, PASSWORD).unwrap_err(),
            CryptoError::Corrupt
        );
        header.kdf.memory_kib = MEMORY_KIB;
        header.kdf.iterations = MAX_ITERATIONS + 1;
        assert_eq!(
            unlock_with_password(&header, PASSWORD).unwrap_err(),
            CryptoError::Corrupt
        );
        header.kdf.iterations = ITERATIONS;
        header.version = 99;
        assert_eq!(
            unlock_with_password(&header, PASSWORD).unwrap_err(),
            CryptoError::Corrupt
        );
    }

    #[test]
    fn the_two_wrapped_copies_are_bound_to_their_kind() {
        let (mut header, _, _) = create(PASSWORD).unwrap();
        // Swapping the two wrapped copies must not let the password open the recovery slot.
        std::mem::swap(&mut header.password, &mut header.recovery);
        assert!(unlock_with_password(&header, PASSWORD).is_err());
    }
}
