//! Cryptography of the password vault.
//!
//! A random 32-byte data key (DEK) encrypts every entry with AES-256-GCM. The DEK is stored once per
//! unlock method, each copy wrapped by its own key: the master password and the recovery key (shown
//! once at creation) go through Argon2id, and Windows Hello supplies a secret that only the operating
//! system can produce after the user proves who they are. The master password and Windows Hello are
//! optional and independent; the recovery key always exists. Nothing here touches the disk or logs;
//! callers get opaque errors that never contain secret material.
//!
//! This code has not been independently audited (see `docs/en/security/account-containers-vault-review.md`).
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    /// The master password copy. None when the vault is opened with Windows Hello instead.
    #[serde(default)]
    pub password: Option<Wrapped>,
    /// The Windows Hello copy. None when Windows Hello is not used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hello: Option<Wrapped>,
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
    /// The vault has no copy for this unlock method (it was never turned on, or was turned off).
    MethodUnavailable,
    /// Turning the method off would leave only the recovery key to open the vault.
    LastMethod,
}

impl CryptoError {
    pub fn code(&self) -> &'static str {
        match self {
            CryptoError::WrongSecret => "vault.wrong_secret",
            CryptoError::Corrupt => "vault.corrupt",
            CryptoError::Unavailable => "vault.crypto_failed",
            CryptoError::InvalidPassword => "vault.password_invalid",
            CryptoError::MethodUnavailable => "vault.method_unavailable",
            CryptoError::LastMethod => "vault.last_method",
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
    let params = Params::new(
        kdf.memory_kib,
        kdf.iterations,
        kdf.parallelism,
        Some(KEY_LEN),
    )
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
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
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

/// The wrapping key of the Windows Hello copy. The secret is a signature only the operating system can
/// produce (after Windows Hello), already high entropy, so a hash with a label is enough: no slow KDF.
fn hello_kek(secret: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    if secret.len() < 16 {
        return Err(CryptoError::Unavailable);
    }
    let mut hasher = Sha256::new();
    hasher.update(b"spark-vault-hello-kek-v1");
    hasher.update(secret);
    Ok(Zeroizing::new(hasher.finalize().into()))
}

fn wrap_hello(secret: &[u8], dek: &Key) -> Result<Wrapped, CryptoError> {
    let kek = hello_kek(secret)?;
    let (nonce, ciphertext) = seal(kek.as_ref(), &wrap_aad("hello"), dek.0.as_ref())?;
    Ok(Wrapped {
        salt: String::new(),
        nonce: B64.encode(nonce),
        ciphertext: B64.encode(ciphertext),
    })
}

fn unwrap_hello(secret: &[u8], wrapped: &Wrapped) -> Result<Key, CryptoError> {
    let decode = |text: &str| B64.decode(text).map_err(|_| CryptoError::Corrupt);
    let (nonce, ciphertext) = (decode(&wrapped.nonce)?, decode(&wrapped.ciphertext)?);
    let kek = hello_kek(secret)?;
    let plain = Zeroizing::new(
        open(kek.as_ref(), &wrap_aad("hello"), &nonce, &ciphertext)
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

/// True when the text has the shape of a recovery key this vault could have issued. It says nothing
/// about whether the key opens a vault; it only keeps other text from being written as one.
pub fn is_recovery_key_shape(text: &str) -> bool {
    decode_recovery(text).is_some()
}

/// Creates a vault: returns its header, the open data key and the recovery key (shown to the user once).
/// At least one of the master password and the Windows Hello secret is required.
pub fn create(
    master_password: Option<&str>,
    hello_secret: Option<&[u8]>,
) -> Result<(Header, Key, String), CryptoError> {
    if master_password.is_none() && hello_secret.is_none() {
        return Err(CryptoError::LastMethod);
    }
    if let Some(password) = master_password {
        validate_password(password)?;
    }
    let dek = Key(Zeroizing::new(random::<KEY_LEN>()?));
    let recovery = Zeroizing::new(random::<RECOVERY_BYTES>()?);
    let kdf = Kdf {
        memory_kib: MEMORY_KIB,
        iterations: ITERATIONS,
        parallelism: PARALLELISM,
    };
    let header = Header {
        version: HEADER_VERSION,
        password: master_password
            .map(|password| wrap("password", password.as_bytes(), &dek, &kdf))
            .transpose()?,
        hello: hello_secret
            .map(|secret| wrap_hello(secret, &dek))
            .transpose()?,
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
    let wrapped = header
        .password
        .as_ref()
        .ok_or(CryptoError::MethodUnavailable)?;
    unwrap("password", password.as_bytes(), wrapped, &header.kdf)
}

pub fn unlock_with_hello(header: &Header, secret: &[u8]) -> Result<Key, CryptoError> {
    check(header)?;
    let wrapped = header
        .hello
        .as_ref()
        .ok_or(CryptoError::MethodUnavailable)?;
    unwrap_hello(secret, wrapped)
}

pub fn unlock_with_recovery(header: &Header, recovery_key: &str) -> Result<Key, CryptoError> {
    check(header)?;
    let bytes = decode_recovery(recovery_key).ok_or(CryptoError::WrongSecret)?;
    unwrap("recovery", bytes.as_ref(), &header.recovery, &header.kdf)
}

/// A new header whose password copy opens with `new_password` (adding the copy if there was none).
/// The data key, the other copies and therefore every stored entry stay the same.
pub fn change_password(
    header: &Header,
    dek: &Key,
    new_password: &str,
) -> Result<Header, CryptoError> {
    validate_password(new_password)?;
    let mut updated = header.clone();
    updated.password = Some(wrap("password", new_password.as_bytes(), dek, &header.kdf)?);
    Ok(updated)
}

/// A new header without the password copy. Refused unless Windows Hello still opens the vault.
pub fn remove_password(header: &Header) -> Result<Header, CryptoError> {
    if header.hello.is_none() {
        return Err(CryptoError::LastMethod);
    }
    let mut updated = header.clone();
    updated.password = None;
    Ok(updated)
}

/// A new header with a Windows Hello copy of the data key (replacing any earlier one).
pub fn set_hello(header: &Header, dek: &Key, secret: &[u8]) -> Result<Header, CryptoError> {
    let mut updated = header.clone();
    updated.hello = Some(wrap_hello(secret, dek)?);
    Ok(updated)
}

/// A new header without the Windows Hello copy. Refused unless the master password still opens the vault.
pub fn remove_hello(header: &Header) -> Result<Header, CryptoError> {
    if header.password.is_none() {
        return Err(CryptoError::LastMethod);
    }
    let mut updated = header.clone();
    updated.hello = None;
    Ok(updated)
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
        let (header, dek, recovery) = create(Some(PASSWORD), None).unwrap();
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
        let (header, _, recovery) = create(Some(PASSWORD), None).unwrap();
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
    fn only_text_shaped_like_a_recovery_key_passes_the_shape_check() {
        let (_, _, recovery) = create(Some(PASSWORD), None).unwrap();
        assert!(is_recovery_key_shape(&recovery));
        for bad in [
            "",
            "short",
            PASSWORD,
            "AAAA-AAAA-AAAA-AAAA-AAAA-AAAA-AAAA-AAA1",
            &format!("{recovery}\nextra"),
        ] {
            assert!(!is_recovery_key_shape(bad), "{bad:?}");
        }
    }

    #[test]
    fn the_recovery_key_has_the_expected_shape_and_is_random() {
        let (_, _, one) = create(Some(PASSWORD), None).unwrap();
        let (_, _, two) = create(Some(PASSWORD), None).unwrap();
        assert_eq!(one.len(), 32 + 7);
        assert!(one.split('-').all(|group| group.len() == 4));
        assert_ne!(one, two);
    }

    #[test]
    fn password_length_is_bounded() {
        assert_eq!(
            create(Some("short"), None).err(),
            Some(CryptoError::InvalidPassword)
        );
        assert_eq!(
            create(Some(&"a".repeat(MAX_PASSWORD_CHARS + 1)), None).err(),
            Some(CryptoError::InvalidPassword)
        );
        assert!(create(Some(&"a".repeat(MAX_PASSWORD_CHARS)), None).is_ok());
    }

    #[test]
    fn entries_round_trip_and_use_a_new_nonce_each_time() {
        let (_, dek, _) = create(Some(PASSWORD), None).unwrap();
        let a = seal_entry(&dek, "id1", b"secret").unwrap();
        let b = seal_entry(&dek, "id1", b"secret").unwrap();
        assert_ne!(a, b);
        assert_ne!(&a[..NONCE_LEN], &b[..NONCE_LEN]);
        assert!(!a.windows(6).any(|w| w == b"secret"));
        assert_eq!(open_entry(&dek, "id1", &a).unwrap().as_slice(), b"secret");
    }

    #[test]
    fn tampering_or_moving_an_entry_is_detected() {
        let (_, dek, _) = create(Some(PASSWORD), None).unwrap();
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
        let (_, other, _) = create(Some(PASSWORD), None).unwrap();
        assert!(open_entry(&other, "id1", &blob).is_err());
    }

    #[test]
    fn changing_the_password_keeps_the_key_the_recovery_and_the_entries() {
        let (header, dek, recovery) = create(Some(PASSWORD), None).unwrap();
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
        let (mut header, _, _) = create(Some(PASSWORD), None).unwrap();
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
        let (mut header, _, _) = create(Some(PASSWORD), None).unwrap();
        // Swapping the two wrapped copies must not let the password open the recovery slot.
        let password = header.password.take().unwrap();
        header.password = Some(std::mem::replace(&mut header.recovery, password));
        assert!(unlock_with_password(&header, PASSWORD).is_err());
    }

    const HELLO: &[u8] =
        b"a signature produced by windows hello after the user proved who they are";

    #[test]
    fn the_vault_can_open_with_windows_hello_alone_or_next_to_the_password() {
        let (only_hello, dek, recovery) = create(None, Some(HELLO)).unwrap();
        assert!(only_hello.password.is_none());
        assert_eq!(
            unlock_with_hello(&only_hello, HELLO).unwrap().0.as_ref(),
            dek.0.as_ref()
        );
        assert_eq!(
            unlock_with_password(&only_hello, PASSWORD).unwrap_err(),
            CryptoError::MethodUnavailable
        );
        assert!(unlock_with_recovery(&only_hello, &recovery).is_ok());
        let (both, dek, _) = create(Some(PASSWORD), Some(HELLO)).unwrap();
        assert_eq!(
            unlock_with_hello(&both, HELLO).unwrap().0.as_ref(),
            dek.0.as_ref()
        );
        assert_eq!(
            unlock_with_password(&both, PASSWORD).unwrap().0.as_ref(),
            dek.0.as_ref()
        );
        let (only_password, _, _) = create(Some(PASSWORD), None).unwrap();
        assert_eq!(
            unlock_with_hello(&only_password, HELLO).unwrap_err(),
            CryptoError::MethodUnavailable
        );
        assert_eq!(create(None, None).err(), Some(CryptoError::LastMethod));
    }

    #[test]
    fn a_wrong_hello_secret_or_a_swapped_copy_never_opens_the_vault() {
        let (mut header, _, _) = create(Some(PASSWORD), Some(HELLO)).unwrap();
        assert_eq!(
            unlock_with_hello(&header, b"another signature that is long enough").unwrap_err(),
            CryptoError::WrongSecret
        );
        assert_eq!(
            unlock_with_hello(&header, b"short").unwrap_err(),
            CryptoError::Unavailable
        );
        // The hello copy is bound to its kind: it cannot stand in for the recovery copy.
        header.recovery = header.hello.clone().unwrap();
        assert!(unlock_with_recovery(&header, "AAAA-AAAA-AAAA-AAAA-AAAA-AAAA-AAAA-AAAA").is_err());
    }

    #[test]
    fn each_method_can_be_added_and_removed_but_one_always_remains() {
        let (header, dek, _) = create(Some(PASSWORD), None).unwrap();
        assert_eq!(
            remove_password(&header).unwrap_err(),
            CryptoError::LastMethod
        );
        let with_hello = set_hello(&header, &dek, HELLO).unwrap();
        assert!(unlock_with_hello(&with_hello, HELLO).is_ok());
        assert!(unlock_with_password(&with_hello, PASSWORD).is_ok());
        let without_password = remove_password(&with_hello).unwrap();
        assert!(unlock_with_password(&without_password, PASSWORD).is_err());
        assert!(unlock_with_hello(&without_password, HELLO).is_ok());
        assert_eq!(
            remove_hello(&without_password).unwrap_err(),
            CryptoError::LastMethod
        );
        // A password can be added back to a vault that only had Windows Hello.
        let again = change_password(&without_password, &dek, "a brand new password").unwrap();
        assert!(unlock_with_password(&again, "a brand new password").is_ok());
        let without_hello = remove_hello(&again).unwrap();
        assert_eq!(
            unlock_with_hello(&without_hello, HELLO).unwrap_err(),
            CryptoError::MethodUnavailable
        );
    }

    #[test]
    fn a_header_written_before_windows_hello_still_reads() {
        let (header, _, _) = create(Some(PASSWORD), None).unwrap();
        let text = serde_json::to_string(&header).unwrap();
        assert!(!text.contains("hello"), "an unused method leaves no trace");
        let read: Header = serde_json::from_str(&text).unwrap();
        assert_eq!(read, header);
        assert!(unlock_with_password(&read, PASSWORD).is_ok());
    }
}
