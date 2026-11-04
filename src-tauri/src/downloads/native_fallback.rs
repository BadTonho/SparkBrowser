use super::{files, manager::DownloadManager};
use crate::state::browser_state::BrowserState;
use std::{path::PathBuf, sync::Mutex};
use tauri::{webview::DownloadEvent, AppHandle, Manager};
struct Pending {
    id: String,
    url: String,
    path: PathBuf,
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
    app: AppHandle,
    settled: bool,
}
impl Drop for Pending {
    fn drop(&mut self) {
        if !self.settled {
            let manager = self.app.state::<DownloadManager>();
            let _ = manager.update_saved(&self.app, &self.id, |t| {
                t.status = "Interrupted".into();
                t.error = Some("downloads.native_unavailable".into());
            });
        }
    }
}
pub fn handler(
    app: AppHandle,
    tab_id: String,
) -> impl Fn(tauri::Webview, DownloadEvent<'_>) -> bool + Send + Sync + 'static {
    let pending: Mutex<Vec<Pending>> = Mutex::new(Vec::new());
    move |_, event| {
        let manager = app.state::<DownloadManager>();
        match event {
            DownloadEvent::Requested { url, destination } => {
                let result = (|| -> Result<(), String> {
                    if !matches!(url.scheme(), "http" | "https" | "blob")
                        || !url.username().is_empty()
                        || url.password().is_some()
                        || crate::privacy::adblock_engine::should_block_url(
                            url.as_str(),
                            *app.state::<BrowserState>()
                                .shield_enabled
                                .lock()
                                .map_err(|e| e.to_string())?,
                        )
                    {
                        return Err("downloads.blocked".into());
                    }
                    let permit = manager
                        .slots
                        .clone()
                        .try_acquire_owned()
                        .map_err(|_| "downloads.slots_busy")?;
                    let task = manager.register(
                        &app,
                        tab_id.clone(),
                        url.to_string(),
                        super::transfer::file_name(&url),
                        true,
                        None,
                    )?;
                    let path = files::owned_partial(&manager.record(&task.id)?)?;
                    *destination = path.clone();
                    manager.native_capability(&app, &task.id, false, false);
                    pending.lock().map_err(|e| e.to_string())?.push(Pending {
                        id: task.id,
                        url: url.to_string(),
                        path,
                        permit: Some(permit),
                        app: app.clone(),
                        settled: false,
                    });
                    manager.refresh_protection(&app, &tab_id);
                    Ok(())
                })();
                if let Err(error) = result {
                    let _ = crate::security::emit_shell(&app, "browser-error", error);
                    return false;
                }
            }
            DownloadEvent::Finished { url, path, success } => {
                let item = pending.lock().ok().and_then(|mut m| {
                    let index = m.iter().position(|v| {
                        path.as_ref()
                            .map_or(v.url == url.as_str(), |p| p == &v.path)
                    })?;
                    Some(m.remove(index))
                });
                if let Some(mut item) = item {
                    item.settled = true;
                    if success && path.as_ref() == Some(&item.path) {
                        item.permit.take();
                        if let Err(error) = manager.body_completed(&app, &item.id) {
                            let _ = manager.update_saved(&app, &item.id, |t| {
                                t.status = "Interrupted".into();
                                t.error = Some(error);
                            });
                        }
                    } else {
                        let _ = manager.update_saved(&app, &item.id, |t| {
                            t.status = "Interrupted".into();
                            t.error = Some("downloads.native_unavailable".into());
                        });
                    }
                }
            }
            _ => {}
        }
        true
    }
}
