use super::{
    files,
    manager::{DownloadManager, Registration},
};
use crate::state::browser_state::BrowserState;
use std::{
    cell::RefCell,
    collections::HashMap,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use webview2_com::{
    BytesReceivedChangedEventHandler, CoTaskMemPWSTR, DownloadStartingEventHandler,
    Microsoft::Web::WebView2::Win32::*, StateChangedEventHandler,
};
use windows_core::Interface;
struct Operation {
    core: ICoreWebView2DownloadOperation,
    label: String,
    token: String,
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
    last: Instant,
}
thread_local! {static OPERATIONS:RefCell<HashMap<String,Operation>>=RefCell::new(HashMap::new());}
fn valid(app: &AppHandle, id: &str) -> bool {
    OPERATIONS.with(|map| {
        map.borrow().get(id).is_some_and(|v| {
            app.get_webview(&v.label).is_some()
                && app
                    .state::<BrowserState>()
                    .security()
                    .keyboard_bridge_tokens()
                    .lock()
                    .is_ok_and(|m| m.get(&v.label) == Some(&v.token))
        })
    })
}
fn state_changed(
    app: &AppHandle,
    id: &str,
    core: &ICoreWebView2DownloadOperation,
) -> windows_core::Result<()> {
    if !valid(app, id) {
        return Ok(());
    }
    let (mut state, mut reason, mut resumable) =
        (Default::default(), Default::default(), Default::default());
    unsafe {
        core.State(&mut state)?;
        core.InterruptReason(&mut reason)?;
        core.CanResume(&mut resumable)?;
    }
    let manager = app.state::<DownloadManager>();
    if state == COREWEBVIEW2_DOWNLOAD_STATE_COMPLETED {
        OPERATIONS.with(|map| {
            map.borrow_mut().remove(id);
        });
        manager.native_capability(app, id, false, false);
        if let Err(e) = manager.body_completed(app, id) {
            crate::error_log::ignored(manager.update_saved(app, id, |t| {
                t.status = "Interrupted".into();
                t.error = Some(e);
            }));
        }
    } else if state == COREWEBVIEW2_DOWNLOAD_STATE_INTERRUPTED {
        let paused = reason == COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_USER_PAUSED;
        if paused {
            OPERATIONS.with(|map| {
                if let Some(v) = map.borrow_mut().get_mut(id) {
                    v.permit.take();
                }
            });
        }
        manager.native_capability(app, id, true, resumable.as_bool());
        crate::error_log::ignored(manager.update_saved(app, id, |t| {
            t.status = if paused { "Paused" } else { "Interrupted" }.into();
            t.error = if paused {
                None
            } else {
                Some(format!("downloads.native_interrupted: {}", reason.0))
            };
        }));
    } else {
        manager.update(app, id, |t| {
            t.status = "Downloading".into();
            t.error = None;
        });
    }
    Ok(())
}

struct NativeDownloadStart<'a> {
    label: &'a str,
    tab_id: &'a str,
    token: &'a str,
    args: &'a ICoreWebView2DownloadStartingEventArgs,
    operation: ICoreWebView2DownloadOperation,
    url: String,
    parsed: tauri::Url,
    target: Option<std::path::PathBuf>,
}
fn start_native_download(app: &AppHandle, start: NativeDownloadStart<'_>) -> Result<(), String> {
    let NativeDownloadStart {
        label,
        tab_id,
        token,
        args,
        operation,
        url,
        parsed,
        target,
    } = start;
    if !matches!(parsed.scheme(), "http" | "https" | "blob")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || crate::privacy::adblock_engine::should_block_url(
            &url,
            crate::privacy::adblock_engine::download_shield_enabled(
                &app.state::<BrowserState>(),
                Some(tab_id),
                &url,
            ),
        )
    {
        return Err("downloads.blocked".into());
    }
    let manager = app.state::<DownloadManager>();
    let permit = manager
        .slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| "downloads.slots_busy")?;
    let task = manager.register(
        app,
        Registration {
            tab_id: tab_id.to_owned(),
            url,
            name: super::transfer::file_name(&parsed),
            native: true,
            selected_target: target,
            expected: None,
        },
    )?;
    let path = files::owned_partial(&manager.record(&task.id)?)?;
    let raw = path.to_string_lossy();
    let native_path = if let Some(unc) = raw.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        raw.strip_prefix(r"\\?\").unwrap_or(&raw).to_string()
    };
    unsafe {
        args.SetResultFilePath(&windows_core::HSTRING::from(native_path.as_str()))
            .map_err(|e| e.to_string())?;
        args.SetHandled(true).map_err(|e| e.to_string())?;
    }
    OPERATIONS.with(|map| {
        map.borrow_mut().insert(
            task.id.clone(),
            Operation {
                core: operation.clone(),
                label: label.to_owned(),
                token: token.to_owned(),
                permit: Some(permit),
                last: Instant::now() - Duration::from_secs(1),
            },
        );
    });
    let (a1, id1) = (app.clone(), task.id.clone());
    let progress = BytesReceivedChangedEventHandler::create(Box::new(move |sender, _| {
        if !valid(&a1, &id1) {
            return Ok(());
        }
        let emit = OPERATIONS.with(|map| {
            map.borrow_mut().get_mut(&id1).is_some_and(|v| {
                if v.last.elapsed() < Duration::from_millis(100) {
                    false
                } else {
                    v.last = Instant::now();
                    true
                }
            })
        });
        if emit {
            if let Some(core) = sender {
                let (mut received, mut total) = (0, 0);
                unsafe {
                    core.BytesReceived(&mut received)?;
                    core.TotalBytesToReceive(&mut total)?;
                }
                a1.state::<DownloadManager>().update(&a1, &id1, |t| {
                    t.received = received.max(0) as u64;
                    t.total = (total >= 0).then_some(total as u64);
                });
            }
        }
        Ok(())
    }));
    let (a2, id2) = (app.clone(), task.id.clone());
    let changed = StateChangedEventHandler::create(Box::new(move |sender, _| {
        if let Some(core) = sender {
            state_changed(&a2, &id2, &core)?;
        }
        Ok(())
    }));
    unsafe {
        operation
            .add_BytesReceivedChanged(&progress, &mut 0)
            .map_err(|e| e.to_string())?;
        operation
            .add_StateChanged(&changed, &mut 0)
            .map_err(|e| e.to_string())?;
    }
    manager.native_capability(app, &task.id, true, false);
    manager.refresh_protection(app, tab_id);
    Ok(())
}
pub fn install(
    app: &AppHandle,
    view: &tauri::Webview,
    tab_id: &str,
    token: &str,
) -> Result<(), String> {
    let (app, label, tab_id, token) = (
        app.clone(),
        view.label().to_owned(),
        tab_id.to_owned(),
        token.to_owned(),
    );
    view.with_webview(move |platform| {
        let result = (|| -> webview2_com::Result<()> {
            let core: ICoreWebView2_4 = unsafe { platform.controller().CoreWebView2()? }.cast()?;
            let a = app.clone();
            let mut registration = 0;
            let handler = DownloadStartingEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else {
                    return Ok(());
                };
                let mut registered = None;
                let admit = (|| -> Result<(), String> {
                    let operation = unsafe { args.DownloadOperation().map_err(|e| e.to_string())? };
                    let mut uri = Default::default();
                    unsafe {
                        operation.Uri(&mut uri).map_err(|e| e.to_string())?;
                    }
                    let url = CoTaskMemPWSTR::from(uri).to_string();
                    let parsed = tauri::Url::parse(&url).map_err(|_| "downloads.invalid_url")?;

                    // If it is a Chrome Web Store extension or CRX package, intercept and install natively into Spark
                    let is_crx = url.contains("clients2.google.com/service/update2/crx")
                        || parsed.path().ends_with(".crx");
                    if is_crx {
                        if let Some(ext_id) = crate::extensions::crx::parse_webstore_id(&url) {
                            let install_app = a.clone();
                            tauri::async_runtime::spawn(async move {
                                match crate::extensions::install_from_webstore(
                                    &install_app,
                                    &ext_id,
                                )
                                .await
                                {
                                    Ok(details) => {
                                        crate::error_log::ignored(crate::security::emit_shell(
                                            &install_app,
                                            "extension-installed",
                                            details,
                                        ));
                                    }
                                    Err(err) => {
                                        crate::error_log::warn(&format!(
                                            "Failed to install extension {}: {}",
                                            ext_id, err
                                        ));
                                        crate::error_log::ignored(crate::security::emit_shell(
                                            &install_app,
                                            "extension-install-failed",
                                            serde_json::json!({ "id": ext_id, "error": err }),
                                        ));
                                    }
                                }
                            });
                            unsafe {
                                crate::error_log::ignored(args.SetCancel(true));
                                crate::error_log::ignored(args.SetHandled(true));
                            }
                            return Ok(());
                        }
                    }
                    if !matches!(parsed.scheme(), "http" | "https" | "blob")
                        || !parsed.username().is_empty()
                        || parsed.password().is_some()
                        || crate::privacy::adblock_engine::should_block_url(
                            &url,
                            crate::privacy::adblock_engine::download_shield_enabled(
                                &a.state::<BrowserState>(),
                                Some(&tab_id),
                                &url,
                            ),
                        )
                    {
                        return Err("downloads.blocked".into());
                    }
                    if a.state::<BrowserState>()
                        .policies()
                        .ask_download_destination()
                    {
                        let mut dialog = a
                            .dialog()
                            .file()
                            .set_file_name(super::transfer::file_name(&parsed));
                        if let Ok(start) = super::files::directory(&a) {
                            dialog = dialog.set_directory(start);
                        }
                        let selected = dialog.blocking_save_file();
                        let Some(path) = selected else {
                            unsafe {
                                crate::error_log::ignored(args.SetCancel(true));
                            }
                            return Ok(());
                        };
                        let target = path.into_path().map_err(|_| "downloads.invalid_target")?;
                        start_native_download(
                            &a,
                            NativeDownloadStart {
                                label: &label,
                                tab_id: &tab_id,
                                token: &token,
                                args: &args,
                                operation,
                                url,
                                parsed,
                                target: Some(target),
                            },
                        )?;
                        return Ok(());
                    }
                    let manager = a.state::<DownloadManager>();
                    let permit = manager
                        .slots
                        .clone()
                        .try_acquire_owned()
                        .map_err(|_| "downloads.slots_busy")?;
                    let task = manager.register(
                        &a,
                        Registration {
                            tab_id: tab_id.clone(),
                            url,
                            name: super::transfer::file_name(&parsed),
                            native: true,
                            selected_target: None,
                            expected: None,
                        },
                    )?;
                    registered = Some(task.id.clone());
                    let path = files::owned_partial(&manager.record(&task.id)?)?;
                    // Chromium requires a regular Win32 path here, not Rust's verbatim prefix.
                    let raw = path.to_string_lossy();
                    let native_path = if let Some(unc) = raw.strip_prefix(r"\\?\UNC\") {
                        format!(r"\\{unc}")
                    } else {
                        raw.strip_prefix(r"\\?\").unwrap_or(&raw).to_string()
                    };
                    unsafe {
                        args.SetResultFilePath(&windows_core::HSTRING::from(native_path.as_str()))
                            .map_err(|e| e.to_string())?;
                        args.SetHandled(true).map_err(|e| e.to_string())?;
                    }
                    OPERATIONS.with(|map| {
                        map.borrow_mut().insert(
                            task.id.clone(),
                            Operation {
                                core: operation.clone(),
                                label: label.clone(),
                                token: token.clone(),
                                permit: Some(permit),
                                last: Instant::now() - Duration::from_secs(1),
                            },
                        );
                    });
                    let (a1, id1) = (a.clone(), task.id.clone());
                    let progress =
                        BytesReceivedChangedEventHandler::create(Box::new(move |sender, _| {
                            if !valid(&a1, &id1) {
                                return Ok(());
                            }
                            let emit = OPERATIONS.with(|map| {
                                map.borrow_mut().get_mut(&id1).is_some_and(|v| {
                                    if v.last.elapsed() < Duration::from_millis(100) {
                                        false
                                    } else {
                                        v.last = Instant::now();
                                        true
                                    }
                                })
                            });
                            if emit {
                                if let Some(core) = sender {
                                    let (mut received, mut total) = (0, 0);
                                    unsafe {
                                        core.BytesReceived(&mut received)?;
                                        core.TotalBytesToReceive(&mut total)?;
                                    }
                                    a1.state::<DownloadManager>().update(&a1, &id1, |t| {
                                        t.received = received.max(0) as u64;
                                        t.total = (total >= 0).then_some(total as u64);
                                    });
                                }
                            }
                            Ok(())
                        }));
                    let (a2, id2) = (a.clone(), task.id.clone());
                    let changed = StateChangedEventHandler::create(Box::new(move |sender, _| {
                        if let Some(core) = sender {
                            state_changed(&a2, &id2, &core)?;
                        }
                        Ok(())
                    }));
                    unsafe {
                        operation
                            .add_BytesReceivedChanged(&progress, &mut 0)
                            .map_err(|e| e.to_string())?;
                        operation
                            .add_StateChanged(&changed, &mut 0)
                            .map_err(|e| e.to_string())?;
                    }
                    manager.native_capability(&a, &task.id, true, false);
                    manager.refresh_protection(&a, &tab_id);
                    Ok(())
                })();
                if let Err(error) = admit {
                    if let Some(id) = registered {
                        let manager = a.state::<DownloadManager>();
                        manager.native_capability(&a, &id, false, false);
                        crate::error_log::ignored(manager.update_saved(&a, &id, |t| {
                            t.status = "Interrupted".into();
                            t.error = Some(error.clone());
                        }));
                    }
                    unsafe {
                        crate::error_log::ignored(args.SetCancel(true));
                    }
                    crate::error_log::ignored(crate::security::emit_shell(
                        &a,
                        "browser-error",
                        error,
                    ));
                }
                Ok(())
            }));
            unsafe {
                core.add_DownloadStarting(&handler, &mut registration)?;
            }
            Ok(())
        })();
        if let Err(e) = result {
            crate::error_log::ignored(crate::security::emit_shell(
                &app,
                "browser-error",
                format!("downloads.native_unavailable: {e}"),
            ));
        }
    })
    .map_err(|e| e.to_string())
}
pub async fn control(app: &AppHandle, id: &str, action: &str) -> Result<(), String> {
    let (send, receive) = tokio::sync::oneshot::channel();
    let a = app.clone();
    let id = id.to_owned();
    let action = action.to_owned();
    app.run_on_main_thread(move || {
        let result = (|| {
            if !valid(&a, &id) {
                if action == "cancel" {
                    let record = a.state::<DownloadManager>().record(&id)?;
                    if let Ok(path) = files::owned_partial(&record) {
                        if path.exists() {
                            std::fs::remove_file(path).map_err(|e| e.to_string())?;
                        }
                    }
                    a.state::<DownloadManager>()
                        .update_saved(&a, &id, |t| t.status = "Cancelled".into())?;
                    return Ok(());
                }
                return Err("downloads.native_unavailable".into());
            }
            let core = OPERATIONS
                .with(|map| map.borrow().get(&id).map(|v| v.core.clone()))
                .ok_or("downloads.native_unavailable")?;
            match action.as_str() {
                "pause" => unsafe {
                    core.Pause().map_err(|e| e.to_string())?;
                },
                "resume" => {
                    let mut can = Default::default();
                    unsafe {
                        core.CanResume(&mut can).map_err(|e| e.to_string())?;
                    }
                    if !can.as_bool() {
                        return Err("downloads.native_unavailable".into());
                    }
                    let already_held = OPERATIONS
                        .with(|m| m.borrow().get(&id).is_some_and(|v| v.permit.is_some()));
                    if !already_held {
                        let permit = a
                            .state::<DownloadManager>()
                            .slots
                            .clone()
                            .try_acquire_owned()
                            .map_err(|_| "downloads.slots_busy")?;
                        OPERATIONS.with(|m| {
                            if let Some(v) = m.borrow_mut().get_mut(&id) {
                                v.permit = Some(permit);
                            }
                        });
                    }
                    if let Err(error) = unsafe { core.Resume() } {
                        if !already_held {
                            OPERATIONS.with(|m| {
                                if let Some(v) = m.borrow_mut().get_mut(&id) {
                                    v.permit.take();
                                }
                            });
                        }
                        return Err(error.to_string());
                    }
                }
                "cancel" => {
                    OPERATIONS.with(|map| {
                        map.borrow_mut().remove(&id);
                    });
                    unsafe {
                        core.Cancel().map_err(|e| e.to_string())?;
                    }
                    let manager = a.state::<DownloadManager>();
                    manager.native_capability(&a, &id, false, false);
                    manager.update_saved(&a, &id, |t| {
                        t.status = "Cancelled".into();
                        t.error = None;
                    })?;
                    let record = manager.record(&id)?;
                    if let Ok(path) = files::owned_partial(&record) {
                        crate::error_log::ignored(std::fs::remove_file(path));
                    }
                }
                _ => return Err("downloads.not_available".into()),
            }
            Ok(())
        })();
        crate::error_log::ignored(send.send(result));
    })
    .map_err(|e| e.to_string())?;
    receive.await.map_err(|e| e.to_string())?
}
pub fn cancel_view(app: &AppHandle, label: &str) {
    let a = app.clone();
    let label = label.to_owned();
    crate::error_log::ignored(app.run_on_main_thread(move || {
        let ids = OPERATIONS.with(|map| {
            map.borrow()
                .iter()
                .filter(|(_, v)| v.label == label)
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>()
        });
        for id in ids {
            if let Some(v) = OPERATIONS.with(|map| map.borrow_mut().remove(&id)) {
                unsafe {
                    crate::error_log::ignored(v.core.Cancel());
                }
                let manager = a.state::<DownloadManager>();
                manager.native_capability(&a, &id, false, false);
                crate::error_log::ignored(manager.update_saved(&a, &id, |t| {
                    t.status = "Interrupted".into();
                    t.error = Some("downloads.native_unavailable".into());
                }));
            }
        }
    }));
}

/// Called off the UI thread, only after explicit shutdown approval.
pub fn stop_for_exit(app: &AppHandle) -> Result<(), String> {
    let (send, receive) = std::sync::mpsc::channel();
    let a = app.clone();
    app.run_on_main_thread(move || {
        let entries = OPERATIONS.with(|m| m.borrow_mut().drain().collect::<Vec<_>>());
        let result = (|| {
            for (id, operation) in entries {
                unsafe {
                    operation.core.Cancel().map_err(|e| e.to_string())?;
                }
                let manager = a.state::<DownloadManager>();
                manager.native_capability(&a, &id, false, false);
                manager.update_saved(&a, &id, |t| {
                    t.status = "Interrupted".into();
                    t.error = Some("downloads.native_unavailable".into());
                })?;
            }
            Ok(())
        })();
        crate::error_log::ignored(send.send(result));
    })
    .map_err(|e| e.to_string())?;
    receive
        .recv_timeout(Duration::from_secs(20))
        .map_err(|e| e.to_string())?
}
