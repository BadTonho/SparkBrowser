use super::model::DownloadRecord;
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

pub fn directory(app: &AppHandle) -> Result<PathBuf, String> {
    let path = if app.config().identifier == "org.veloce.browser.native-tests" {
        app.path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .join("downloads")
    } else {
        app.path().download_dir().map_err(|e| e.to_string())?
    };
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    std::fs::canonicalize(path).map_err(|e| e.to_string())
}
pub fn owned_partial(record: &DownloadRecord) -> Result<PathBuf, String> {
    if !record
        .task
        .id
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        || record.task.id.is_empty()
    {
        return Err("downloads.invalid_partial".into());
    }
    let root =
        std::fs::canonicalize(&record.directory).map_err(|_| "downloads.missing_directory")?;
    let folder = root.join(".veloce-partials");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    if std::fs::canonicalize(&folder).map_err(|e| e.to_string())? != folder {
        return Err("downloads.invalid_partial".into());
    }
    let file = record.partial();
    if let Ok(meta) = std::fs::symlink_metadata(&file) {
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err("downloads.invalid_partial".into());
        }
    }
    Ok(file)
}
pub fn hash(path: &Path, limit: Option<u64>) -> Result<(u64, String), String> {
    let file = std::fs::File::open(path).map_err(|_| "downloads.file_missing")?;
    let mut reader = file.take(limit.unwrap_or(u64::MAX));
    let mut count = 0;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let n = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
        count += n as u64;
    }
    Ok((count, format!("{:x}", hash.finalize())))
}
pub fn publish(record: &DownloadRecord) -> Result<(), String> {
    let partial = owned_partial(record)?;
    let target = PathBuf::from(
        record
            .task
            .save_path
            .as_ref()
            .ok_or("downloads.invalid_target")?,
    );
    if target.parent() != Some(record.directory.as_path()) {
        return Err("downloads.invalid_target".into());
    }
    if !partial.exists() {
        let (size, actual) = hash(&target, None)?;
        if size == record.confirmed && Some(&actual) == record.task.actual_sha256.as_ref() {
            return Ok(());
        }
        return Err("downloads.file_changed".into());
    }
    let path = tempfile::TempPath::try_from_path(partial).map_err(|e| e.to_string())?;
    if let Err(error) = path.persist_noclobber(target) {
        let message = error.error.to_string();
        let _ = error.path.keep();
        return Err(format!("downloads.publish_failed: {message}"));
    }
    Ok(())
}
