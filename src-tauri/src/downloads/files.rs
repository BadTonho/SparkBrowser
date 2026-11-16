use super::model::DownloadRecord;
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

pub fn directory(app: &AppHandle) -> Result<PathBuf, String> {
    let path = if app.config().identifier == "org.spark.browser.native-tests" {
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
    let folder = root.join(".spark-partials");
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

pub fn available_target(selected: &Path) -> Result<PathBuf, String> {
    if !selected.is_absolute() || selected.file_name().is_none() {
        return Err("downloads.invalid_target".into());
    }
    let parent = selected.parent().ok_or("downloads.invalid_target")?;
    let parent = std::fs::canonicalize(parent).map_err(|_| "downloads.invalid_target")?;
    let name = selected
        .file_name()
        .ok_or("downloads.invalid_target")?
        .to_string_lossy();
    for index in 0..10_000u32 {
        let candidate_name = if index == 0 {
            name.to_string()
        } else {
            let path = Path::new(name.as_ref());
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let extension = path
                .extension()
                .map(|value| format!(".{}", value.to_string_lossy()))
                .unwrap_or_default();
            format!("{stem} ({index}){extension}")
        };
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("downloads.too_many_collisions".into())
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
pub fn publish(record: &DownloadRecord) -> Result<PathBuf, String> {
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
            return Ok(target);
        }
        return Err("downloads.file_changed".into());
    }
    let path = tempfile::TempPath::try_from_path(partial).map_err(|e| e.to_string())?;
    let mut path = path;
    for _ in 0..10_000 {
        let candidate = available_target(&target)?;
        match path.persist_noclobber(&candidate) {
            Ok(_) => return Ok(candidate),
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                path = error.path;
            }
            Err(error) => {
                let message = error.error.to_string();
                crate::error_log::ignored(error.path.keep());
                return Err(format!("downloads.publish_failed: {message}"));
            }
        }
    }
    crate::error_log::ignored(path.keep());
    Err("downloads.too_many_collisions".into())
}
