use super::manager::DownloadManager;
use crate::{database::downloads_repo::record_download, state::browser_state::BrowserState};
use std::{path::PathBuf, sync::Mutex};
use tauri::{webview::DownloadEvent, AppHandle, Emitter, Manager};

struct Pending {
    id: String,
    url: String,
    path: PathBuf,
    _permit: tokio::sync::OwnedSemaphorePermit,
    guard: CompletionGuard,
}

struct CompletionGuard {
    app: AppHandle,
    tab_id: String,
    id: String,
    completed: bool,
}
impl Drop for CompletionGuard {
    fn drop(&mut self) {
        if !self.completed {
            let Some(manager) = self.app.try_state::<DownloadManager>() else {
                return;
            };
            manager.update(&self.app, &self.id, |task| {
                task.status = "Failed".into();
                task.error = Some("The download WebView was closed".into());
            });
            manager.refresh_protection(&self.app, &self.tab_id);
        }
    }
}

/// Preserve engine authentication/cookies and blob handling; share admission limits with manual downloads.
pub fn handler(
    app: AppHandle,
    tab_id: String,
) -> impl Fn(tauri::Webview, DownloadEvent<'_>) -> bool + Send + Sync + 'static {
    let pending: Mutex<Vec<Pending>> = Mutex::new(Vec::new());
    move |_, event| {
        let manager = app.state::<DownloadManager>();
        match event {
            DownloadEvent::Requested { url, destination } => {
                let admit = (|| {
                    let shield = *app
                        .state::<BrowserState>()
                        .shield_enabled
                        .lock()
                        .map_err(|e| e.to_string())?;
                    if !url.username().is_empty()
                        || url.password().is_some()
                        || !matches!(url.scheme(), "http" | "https" | "blob")
                        || crate::privacy::adblock_engine::should_block_url(url.as_str(), shield)
                    {
                        return Err("Page download blocked by the URL policy".to_string());
                    }
                    let permit = manager.slots.clone().try_acquire_owned().map_err(|_| "Two downloads are already running. Retry the page download when a slot is free.".to_string())?;
                    let directory = app.path().download_dir().map_err(|e| e.to_string())?;
                    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
                    let name = super::transfer::file_name(&url);
                    let reserved = tempfile::Builder::new()
                        .prefix("veloce-")
                        .suffix(&format!("-{name}"))
                        .tempfile_in(directory)
                        .map_err(|e| e.to_string())?;
                    let task = manager.register(tab_id.clone(), url.to_string(), name, true)?;
                    let guard = CompletionGuard {
                        app: app.clone(),
                        tab_id: tab_id.clone(),
                        id: task.id.clone(),
                        completed: false,
                    };
                    let (file, path) = reserved.keep().map_err(|e| e.to_string())?;
                    drop(file);
                    *destination = path.clone();
                    manager.update(&app, &task.id, |task| {
                        task.save_path = Some(path.to_string_lossy().to_string())
                    });
                    pending.lock().map_err(|e| e.to_string())?.push(Pending {
                        id: task.id,
                        url: url.to_string(),
                        path,
                        _permit: permit,
                        guard,
                    });
                    manager.refresh_protection(&app, &tab_id);
                    Ok::<(), String>(())
                })();
                if let Err(error) = admit {
                    let _ = app.emit("browser-error", error);
                    return false;
                }
            }
            DownloadEvent::Finished { url, path, success } => {
                let item = pending.lock().ok().and_then(|mut entries| {
                    let index = entries
                        .iter()
                        .position(|entry| {
                            path.as_ref()
                                .map_or(entry.url == url.as_str(), |path| entry.path == *path)
                        })
                        .or_else(|| entries.iter().position(|entry| entry.url == url.as_str()))?;
                    Some(entries.remove(index))
                });
                if let Some(mut item) = item {
                    item.guard.completed = true;
                    if let Some(actual) = path.filter(|actual| *actual != item.path) {
                        if std::fs::metadata(&item.path).is_ok_and(|metadata| metadata.len() == 0) {
                            let _ = std::fs::remove_file(&item.path);
                        }
                        item.path = actual;
                    }
                    let size = std::fs::metadata(&item.path)
                        .ok()
                        .map(|metadata| metadata.len());
                    manager.update(&app, &item.id, |task| {
                        task.status = if success { "Completed" } else { "Failed" }.into();
                        task.received = size.unwrap_or(0);
                        task.total = size;
                        task.save_path = Some(item.path.to_string_lossy().to_string());
                        task.file_name = item
                            .path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string())
                            .unwrap_or_default();
                        task.error =
                            (!success).then(|| "The page download did not complete".into());
                    });
                    manager.refresh_protection(&app, &tab_id);
                    if success {
                        let app = app.clone();
                        tauri::async_runtime::spawn_blocking(move || {
                            let state = app.state::<BrowserState>();
                            let name = item
                                .path
                                .file_name()
                                .map(|name| name.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let result =
                                state.db.lock().map_err(|e| e.to_string()).and_then(|db| {
                                    record_download(
                                        db.as_ref().ok_or("Download database is not ready")?,
                                        &name,
                                        &item.url,
                                        &item.path.to_string_lossy(),
                                        "Completed",
                                    )
                                });
                            match result {
                                Ok(items) => {
                                    let _ = app.emit("downloads-updated", items);
                                }
                                Err(_) => {
                                    let _ = app.emit(
                                        "browser-error",
                                        "Could not persist the page download record",
                                    );
                                }
                            }
                        });
                    } else if size == Some(0) {
                        // Only remove the empty reservation created by this handler.
                        let _ = std::fs::remove_file(item.path);
                    }
                }
            }
            _ => {}
        }
        true
    }
}
