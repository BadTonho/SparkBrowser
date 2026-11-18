use super::{
    files,
    manager::{DownloadManager, Registration},
};
use crate::state::browser_state::BrowserState;
use std::{path::PathBuf, sync::Mutex};
use tauri::{webview::DownloadEvent, AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
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
            crate::error_log::ignored(manager.update_saved(&self.app, &self.id, |t| {
                t.status = "Interrupted".into();
                t.error = Some("downloads.native_unavailable".into());
            }));
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
                            crate::privacy::adblock_engine::download_shield_enabled(
                                &app.state::<BrowserState>(),
                                Some(&tab_id),
                                url.as_str(),
                            ),
                        )
                    {
                        return Err("downloads.blocked".into());
                    }
                    let selected_target = if app
                        .state::<BrowserState>()
                        .policies()
                        .ask_download_destination()
                    {
                        let selected = app
                            .dialog()
                            .file()
                            .set_file_name(super::transfer::file_name(&url))
                            .blocking_save_file();
                        let Some(path) = selected else {
                            return Err("downloads.cancelled".into());
                        };
                        Some(path.into_path().map_err(|_| "downloads.invalid_target")?)
                    } else {
                        None
                    };
                    let permit = manager
                        .slots
                        .clone()
                        .try_acquire_owned()
                        .map_err(|_| "downloads.slots_busy")?;
                    let task = manager.register(
                        &app,
                        Registration {
                            tab_id: tab_id.clone(),
                            url: url.to_string(),
                            name: super::transfer::file_name(&url),
                            native: true,
                            selected_target,
                            expected: None,
                        },
                    )?;
                    let path = files::owned_partial(&manager.record(&task.id)?)?;
                    *destination = path.clone();
                    manager.native_capability(&app, &task.id, false, false);
                    #[cfg(target_os = "linux")]
                    super::native_linux::expect(url.as_str(), &task.id);
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
                    if error == "downloads.cancelled" {
                        return false;
                    }
                    crate::error_log::ignored(crate::security::emit_shell(
                        &app,
                        "browser-error",
                        error,
                    ));
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
                            crate::error_log::ignored(manager.update_saved(&app, &item.id, |t| {
                                t.status = "Interrupted".into();
                                t.error = Some(error);
                            }));
                        }
                    } else {
                        #[cfg(target_os = "linux")]
                        let cancelled = super::native_linux::take_cancelled(&item.id);
                        #[cfg(not(target_os = "linux"))]
                        let cancelled = false;
                        if cancelled {
                            crate::error_log::ignored(std::fs::remove_file(&item.path));
                        }
                        crate::error_log::ignored(manager.update_saved(&app, &item.id, |t| {
                            if cancelled {
                                t.status = "Cancelled".into();
                                t.error = None;
                            } else {
                                t.status = "Interrupted".into();
                                t.error = Some("downloads.native_unavailable".into());
                            }
                        }));
                    }
                }
            }
            _ => {}
        }
        true
    }
}
