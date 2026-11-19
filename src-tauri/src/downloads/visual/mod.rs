//! Presentation-only reads of registered downloads. No caller-supplied filesystem paths.
#[cfg(target_os = "linux")]
mod linux;
#[cfg(test)]
mod tests;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(windows)]
use windows as platform;

use super::{files, manager::DownloadManager};
use crate::database::downloads_repo;
#[cfg(any(windows, target_os = "linux"))]
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    path::Path,
    sync::{Arc, OnceLock},
};
use tauri::{AppHandle, Manager};

#[cfg(any(windows, target_os = "linux", test))]
const MAX_FILE: u64 = 16 * 1024 * 1024;
#[cfg(any(windows, target_os = "linux", test))]
const MAX_PIXELS: u64 = 16_000_000;
#[cfg(any(windows, target_os = "linux", test))]
const THUMBNAIL_SIDE: u32 = 96;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VisualSource {
    Task { id: String },
    History { id: i64 },
}
#[derive(Default, Serialize)]
pub struct DownloadVisual {
    size_bytes: Option<u64>,
    thumbnail_png: Option<String>,
}

pub async fn get(app: AppHandle, source: VisualSource) -> Result<DownloadVisual, String> {
    let target = match source {
        VisualSource::Task { id } => {
            if id.is_empty() || id.len() > 128 {
                return Err("downloads.not_found".into());
            }
            let record = app.state::<DownloadManager>().record(&id)?;
            if record.task.status != "Completed" || !record.published {
                return Ok(DownloadVisual::default());
            }
            record.task.save_path
        }
        VisualSource::History { id } => {
            if id <= 0 {
                return Err("downloads.not_found".into());
            }
            Some(
                crate::database::executor::run(app.clone(), move |conn| {
                    downloads_repo::find_path(conn, id)?.ok_or("downloads.not_found".into())
                })
                .await?,
            )
        }
    };
    let Some(target) = target else {
        return Ok(DownloadVisual::default());
    };
    let root = files::directory(&app)?;
    static WORK: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    let permit = WORK
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(2)))
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| "downloads.error")?;
    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        inspect(&root, Path::new(&target))
    })
    .await
    .map_err(|_| "downloads.error".into())
}

fn owned_file(root: &Path, path: &Path) -> Option<File> {
    let root = root.canonicalize().ok()?;
    // Published downloads are direct children, never partials or arbitrary descendants.
    if path.parent()?.canonicalize().ok()? != root {
        return None;
    }
    let metadata = std::fs::symlink_metadata(path).ok()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return None;
    }
    let file = File::open(path).ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    #[cfg(windows)]
    if windows::final_path(&file)?.parent()? != root {
        return None;
    }
    #[cfg(not(windows))]
    if path.canonicalize().ok()?.parent()? != root {
        return None;
    }
    Some(file)
}

fn inspect(root: &Path, path: &Path) -> DownloadVisual {
    let Some(file) = owned_file(root, path) else {
        return DownloadVisual::default();
    };
    let Some(size) = file.metadata().ok().map(|m| m.len()) else {
        return DownloadVisual::default();
    };
    #[cfg(any(windows, target_os = "linux"))]
    let thumbnail_png = read_thumbnail(file, size);
    #[cfg(not(any(windows, target_os = "linux")))]
    let thumbnail_png = None;
    DownloadVisual {
        size_bytes: Some(size),
        thumbnail_png,
    }
}

#[cfg(any(windows, target_os = "linux"))]
fn read_thumbnail(mut file: File, size: u64) -> Option<String> {
    use std::io::Read;
    if size == 0 || size > MAX_FILE {
        return None;
    }
    let mut bytes = vec![0; 12];
    file.read_exact(&mut bytes).ok()?;
    if !platform::supported_header(&bytes) {
        return None;
    }
    if file
        .take(MAX_FILE + 1 - 12)
        .read_to_end(&mut bytes)
        .is_err()
        || bytes.len() as u64 > MAX_FILE
    {
        return None;
    }
    let png = platform::thumbnail(&bytes)?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(png)))
}

#[cfg(any(windows, target_os = "linux", test))]
fn thumbnail_size(width: u32, height: u32) -> Option<(u32, u32)> {
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return None;
    }
    let longest = width.max(height).max(THUMBNAIL_SIDE);
    Some((
        (u64::from(width) * u64::from(THUMBNAIL_SIDE) / u64::from(longest)).max(1) as u32,
        (u64::from(height) * u64::from(THUMBNAIL_SIDE) / u64::from(longest)).max(1) as u32,
    ))
}
