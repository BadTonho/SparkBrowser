use super::{crypto, model::*, validation};
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    fs::File,
    io::{Read, Write},
    path::Path,
};

/// Encrypted envelopes are base64 inside JSON, so they may exceed the plaintext cap.
pub const MAX_FILE_BYTES: u64 = MAX_BYTES + MAX_BYTES / 2 + 4096;

pub fn read(path: &Path) -> Result<BackupDocument, String> {
    read_with(path, None)
}
pub fn read_with(path: &Path, password: Option<&str>) -> Result<BackupDocument, String> {
    decode_with(&read_bytes(path)?, password)
}
/// Raw bytes of a backup file, bounded; used to hold an encrypted file until unlocked.
pub fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_FILE_BYTES {
        return Err("backup.file_too_large".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}
pub fn decode(bytes: &[u8]) -> Result<BackupDocument, String> {
    decode_with(bytes, None)
}
/// Encrypted input without a password fails with `backup.password_required`.
pub fn decode_with(bytes: &[u8], password: Option<&str>) -> Result<BackupDocument, String> {
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("backup.file_too_large".into());
    }
    let plain: Cow<[u8]> = if crypto::is_encrypted(bytes) {
        let password = password.ok_or("backup.password_required")?;
        Cow::Owned(crypto::decrypt(bytes, password)?)
    } else {
        Cow::Borrowed(bytes)
    };
    if plain.len() as u64 > MAX_BYTES {
        return Err("backup.file_too_large".into());
    }
    let mut value: serde_json::Value =
        serde_json::from_slice(&plain).map_err(|e| format!("backup.invalid_document: {e}"))?;
    verify_integrity(&mut value)?;
    let doc: BackupDocument =
        serde_json::from_value(value).map_err(|e| format!("backup.invalid_document: {e}"))?;
    validation::document(&doc)?;
    Ok(doc)
}
const CHECKSUM_KEY: &str = "checksum";
/// SHA-256 of the canonical (key-sorted, compact) JSON, so reformatting a file never invalidates it.
fn digest(value: &serde_json::Value) -> Result<String, String> {
    struct Hasher(Sha256);
    impl Write for Hasher {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut hasher = Hasher(Sha256::new());
    serde_json::to_writer(&mut hasher, value).map_err(|e| e.to_string())?;
    Ok(hasher
        .0
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
/// Rejects files from newer versions and files whose content no longer matches the stored
/// checksum. The checksum key is removed so the strict document parser never sees it.
fn verify_integrity(value: &mut serde_json::Value) -> Result<(), String> {
    let Some(object) = value.as_object_mut() else {
        return Ok(());
    };
    if !object
        .get("format")
        .and_then(|v| v.as_str())
        .is_some_and(is_backup_format)
    {
        return Ok(());
    }
    let version = object.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
    if version > u64::from(CURRENT_VERSION) {
        return Err("backup.newer_version".into());
    }
    if version < 5 {
        return Ok(());
    }
    let stored = object
        .remove(CHECKSUM_KEY)
        .ok_or("backup.checksum_missing")?;
    if stored.as_str() != Some(digest(value)?.as_str()) {
        return Err("backup.checksum_mismatch".into());
    }
    Ok(())
}
/// Serialized (and optionally encrypted) bytes ready to be stored or uploaded.
pub fn encode(document: &BackupDocument, password: Option<&str>) -> Result<Vec<u8>, String> {
    validation::document(document)?;
    let bytes = if document.version >= 5 {
        let mut value = serde_json::to_value(document).map_err(|e| e.to_string())?;
        let checksum = digest(&value)?;
        value
            .as_object_mut()
            .ok_or("backup.invalid_document")?
            .insert(CHECKSUM_KEY.into(), checksum.into());
        serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?
    } else {
        serde_json::to_vec_pretty(document).map_err(|e| e.to_string())?
    };
    if bytes.len() as u64 > MAX_BYTES {
        return Err("backup.file_too_large".into());
    }
    match password {
        Some(password) => crypto::encrypt(&bytes, password),
        None => Ok(bytes),
    }
}
pub fn write(path: &Path, document: &BackupDocument) -> Result<(), String> {
    write_with(path, document, None)
}
pub fn write_with(
    path: &Path,
    document: &BackupDocument,
    password: Option<&str>,
) -> Result<(), String> {
    write_bytes(path, &encode(document, password)?)
}
/// Atomic replacement: temp file in the same directory, fsync, then rename over the target.
pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("backup.invalid_path")?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".spark-backup-")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|e| e.to_string())?;
    temporary.write_all(bytes).map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(path).map_err(|e| e.error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_file_roundtrip_requires_the_password() {
        let (_, doc) = crate::backup::tests::setup();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.spark-backup.json");
        write_with(&path, &doc, Some("correct horse")).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("spark-backup-encrypted"));
        assert!(!raw.contains(&doc.data.tabs[0].url));
        assert_eq!(read(&path).err().unwrap(), "backup.password_required");
        assert_eq!(
            read_with(&path, Some("wrong horse!")).err().unwrap(),
            "backup.password_wrong"
        );
        let restored = read_with(&path, Some("correct horse")).unwrap();
        assert_eq!(restored.data.tabs[0].url, doc.data.tabs[0].url);
        assert_eq!(restored.created_at, doc.created_at);
    }

    #[test]
    fn plaintext_files_still_read_when_a_password_is_supplied() {
        let (_, doc) = crate::backup::tests::setup();
        let bytes = encode(&doc, None).unwrap();
        assert_eq!(
            decode_with(&bytes, Some("anything at all"))
                .unwrap()
                .version,
            doc.version
        );
    }

    #[test]
    fn encrypted_envelopes_may_exceed_the_plaintext_cap_but_not_the_file_cap() {
        assert!(decode_with(&vec![b' '; MAX_FILE_BYTES as usize + 1], None).is_err());
        assert_eq!(
            decode_with(&vec![b' '; MAX_BYTES as usize + 1], None)
                .err()
                .unwrap(),
            "backup.file_too_large"
        );
    }
}
