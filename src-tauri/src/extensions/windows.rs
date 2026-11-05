use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Webview};
use webview2_com::{
    BrowserExtensionEnableCompletedHandler, BrowserExtensionRemoveCompletedHandler, CoTaskMemPWSTR,
    Microsoft::Web::WebView2::Win32::*, ProfileAddBrowserExtensionCompletedHandler,
    ProfileGetBrowserExtensionsCompletedHandler,
};
use windows_core::{Interface, PCWSTR};

type Reply<T> = Arc<Mutex<Option<tokio::sync::oneshot::Sender<Result<T, String>>>>>;

fn reply<T>(sender: &Reply<T>, result: Result<T, String>) {
    if let Ok(mut sender) = sender.lock() {
        if let Some(sender) = sender.take() {
            let _ = sender.send(result);
        }
    }
}

pub fn get_profile7(core: &ICoreWebView2) -> Result<ICoreWebView2Profile7, String> {
    unsafe {
        core.cast::<ICoreWebView2_13>()
            .and_then(|c| c.Profile())
            .and_then(|p| p.cast::<ICoreWebView2Profile7>())
            .map_err(|e| format!("ICoreWebView2Profile7 unavailable: {}", e))
    }
}

/// Executes an operation against the active content webview's Profile7 or main shell profile.
async fn with_profile7<T: Send + 'static>(
    app: &AppHandle,
    operation: impl FnOnce(ICoreWebView2Profile7, Reply<T>) -> Result<(), String> + Send + 'static,
) -> Result<T, String> {
    let view = crate::window::webview_manager::active_content_webview(app)
        .or_else(|| app.get_webview("main"))
        .ok_or_else(|| "No webview available for extension profile".to_string())?;

    let (send, receive) = tokio::sync::oneshot::channel();
    let send = Arc::new(Mutex::new(Some(send)));

    view.with_webview(move |platform| {
        let result = unsafe { platform.controller().CoreWebView2() }
            .map_err(|e| e.to_string())
            .and_then(|core| get_profile7(&core))
            .and_then(|profile| operation(profile, send.clone()));

        if let Err(error) = result {
            reply(&send, Err(error));
        }
    })
    .map_err(|e| e.to_string())?;

    receive.await.map_err(|e| e.to_string())?
}

/// Adds an unpacked browser extension to WebView2.
/// Returns the WebView2 Extension ID.
pub async fn add_browser_extension(
    app: &AppHandle,
    extension_folder_path: &str,
) -> Result<String, String> {
    let wide_path: Vec<u16> = extension_folder_path
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    with_profile7(app, move |profile, send| {
        let pcwstr = PCWSTR(wide_path.as_ptr());
        let send_clone = send.clone();

        let handler = ProfileAddBrowserExtensionCompletedHandler::create(Box::new(
            move |status, extension_opt| {
                if let Err(e) = status {
                    reply(
                        &send_clone,
                        Err(format!("AddBrowserExtension failed: {}", e)),
                    );
                    return Ok(());
                }

                let ext = match extension_opt {
                    Some(e) => e,
                    None => {
                        reply(
                            &send_clone,
                            Err("Extension added but handle is null".into()),
                        );
                        return Ok(());
                    }
                };

                let mut raw_id = Default::default();
                unsafe {
                    if let Err(e) = ext.Id(&mut raw_id) {
                        reply(
                            &send_clone,
                            Err(format!("Failed to retrieve extension ID: {}", e)),
                        );
                        return Ok(());
                    }
                }
                let ext_id = CoTaskMemPWSTR::from(raw_id).to_string();
                reply(&send_clone, Ok(ext_id));
                Ok(())
            },
        ));

        unsafe {
            profile
                .AddBrowserExtension(pcwstr, &handler)
                .map_err(|e| format!("Failed to call AddBrowserExtension: {}", e))?;
        }
        Ok(())
    })
    .await
}

/// Toggles an extension enabled/disabled state in WebView2.
pub async fn set_browser_extension_enabled(
    app: &AppHandle,
    extension_id: String,
    enabled: bool,
) -> Result<(), String> {
    with_profile7(app, move |profile, send| {
        let send_clone = send.clone();
        let target_id = extension_id.clone();

        let get_handler = ProfileGetBrowserExtensionsCompletedHandler::create(Box::new(
            move |status, list_opt| {
                if let Err(e) = status {
                    reply(
                        &send_clone,
                        Err(format!("GetBrowserExtensions failed: {}", e)),
                    );
                    return Ok(());
                }

                let list = match list_opt {
                    Some(l) => l,
                    None => {
                        reply(&send_clone, Err("Extension list is null".into()));
                        return Ok(());
                    }
                };

                let mut count = 0;
                unsafe {
                    if let Err(e) = list.Count(&mut count) {
                        reply(
                            &send_clone,
                            Err(format!("Failed to get extension count: {}", e)),
                        );
                        return Ok(());
                    }
                }

                let mut found_ext = None;
                for i in 0..count {
                    unsafe {
                        if let Ok(item) = list.GetValueAtIndex(i) {
                            let mut raw_id = Default::default();
                            if item.Id(&mut raw_id).is_ok() {
                                let item_id = CoTaskMemPWSTR::from(raw_id).to_string();
                                if item_id == target_id {
                                    found_ext = Some(item);
                                    break;
                                }
                            }
                        }
                    }
                }

                let ext = match found_ext {
                    Some(e) => e,
                    None => {
                        // Extension might not be loaded in WebView2 memory yet; consider success
                        reply(&send_clone, Ok(()));
                        return Ok(());
                    }
                };

                let send_inner = send_clone.clone();
                let enable_handler =
                    BrowserExtensionEnableCompletedHandler::create(Box::new(move |status| {
                        if let Err(e) = status {
                            reply(&send_inner, Err(format!("Enable/disable failed: {}", e)));
                        } else {
                            reply(&send_inner, Ok(()));
                        }
                        Ok(())
                    }));

                unsafe {
                    if let Err(e) = ext.Enable(enabled, &enable_handler) {
                        reply(&send_clone, Err(format!("Failed to call Enable: {}", e)));
                    }
                }
                Ok(())
            },
        ));

        unsafe {
            profile
                .GetBrowserExtensions(&get_handler)
                .map_err(|e| format!("Failed to call GetBrowserExtensions: {}", e))?;
        }
        Ok(())
    })
    .await
}

/// Removes an extension from WebView2.
pub async fn remove_browser_extension(app: &AppHandle, extension_id: String) -> Result<(), String> {
    with_profile7(app, move |profile, send| {
        let send_clone = send.clone();
        let target_id = extension_id.clone();

        let get_handler = ProfileGetBrowserExtensionsCompletedHandler::create(Box::new(
            move |status, list_opt| {
                if let Err(e) = status {
                    reply(
                        &send_clone,
                        Err(format!("GetBrowserExtensions failed: {}", e)),
                    );
                    return Ok(());
                }

                let list = match list_opt {
                    Some(l) => l,
                    None => {
                        reply(&send_clone, Ok(()));
                        return Ok(());
                    }
                };

                let mut count = 0;
                unsafe {
                    if let Err(e) = list.Count(&mut count) {
                        reply(
                            &send_clone,
                            Err(format!("Failed to count extensions: {}", e)),
                        );
                        return Ok(());
                    }
                }

                let mut found_ext = None;
                for i in 0..count {
                    unsafe {
                        if let Ok(item) = list.GetValueAtIndex(i) {
                            let mut raw_id = Default::default();
                            if item.Id(&mut raw_id).is_ok() {
                                let item_id = CoTaskMemPWSTR::from(raw_id).to_string();
                                if item_id == target_id {
                                    found_ext = Some(item);
                                    break;
                                }
                            }
                        }
                    }
                }

                let ext = match found_ext {
                    Some(e) => e,
                    None => {
                        reply(&send_clone, Ok(()));
                        return Ok(());
                    }
                };

                let send_inner = send_clone.clone();
                let remove_handler =
                    BrowserExtensionRemoveCompletedHandler::create(Box::new(move |status| {
                        if let Err(e) = status {
                            reply(&send_inner, Err(format!("Remove extension failed: {}", e)));
                        } else {
                            reply(&send_inner, Ok(()));
                        }
                        Ok(())
                    }));

                unsafe {
                    if let Err(e) = ext.Remove(&remove_handler) {
                        reply(&send_clone, Err(format!("Failed to call Remove: {}", e)));
                    }
                }
                Ok(())
            },
        ));

        unsafe {
            profile
                .GetBrowserExtensions(&get_handler)
                .map_err(|e| format!("Failed to call GetBrowserExtensions: {}", e))?;
        }
        Ok(())
    })
    .await
}

/// Synchronizes all enabled extensions onto a newly spawned Webview profile.
pub fn sync_new_webview(app: &AppHandle, view: &Webview) -> Result<(), String> {
    let conn = crate::database::connection::init_database(app)?;
    let extensions = crate::database::extensions_repo::get_all(&conn)?;
    let enabled_paths: Vec<String> = extensions
        .into_iter()
        .filter(|e| e.enabled)
        .map(|e| e.path)
        .collect();

    if enabled_paths.is_empty() {
        return Ok(());
    }

    view.with_webview(move |platform| {
        let result = (|| -> windows_core::Result<()> {
            let core = unsafe { platform.controller().CoreWebView2()? };
            let profile7 = get_profile7(&core)
                .map_err(|_| windows_core::Error::from_hresult(windows_core::HRESULT(-1)))?;

            for path in enabled_paths {
                let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
                let handler =
                    ProfileAddBrowserExtensionCompletedHandler::create(Box::new(|_, _| Ok(())));
                unsafe {
                    let _ = profile7.AddBrowserExtension(PCWSTR(wide.as_ptr()), &handler);
                }
            }
            Ok(())
        })();
        let _ = result;
    })
    .map_err(|e| e.to_string())?;

    Ok(())
}
