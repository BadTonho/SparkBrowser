use super::{model::*, validation};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub fn read(path: &Path) -> Result<BackupDocument, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_BYTES {
        return Err("backup.file_too_large".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    decode(&bytes)
}
pub fn decode(bytes: &[u8]) -> Result<BackupDocument, String> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err("backup.file_too_large".into());
    }
    let doc = serde_json::from_slice(bytes).map_err(|e| format!("backup.invalid_document: {e}"))?;
    validation::document(&doc)?;
    Ok(doc)
}
pub fn write(path: &Path, document: &BackupDocument) -> Result<(), String> {
    validation::document(document)?;
    let bytes = serde_json::to_vec_pretty(document).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("backup.file_too_large".into());
    }
    let parent = path.parent().ok_or("backup.invalid_path")?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".veloce-backup-")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|e| e.to_string())?;
    temporary.write_all(&bytes).map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(path).map_err(|e| e.error.to_string())?;
    Ok(())
}
