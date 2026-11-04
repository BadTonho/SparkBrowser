use tauri::AppHandle;

#[cfg(windows)]
use std::sync::{Arc, Mutex};
#[cfg(windows)]
use tauri::Manager;
#[cfg(windows)]
use webview2_com::{
    CoTaskMemPWSTR, GetCookiesCompletedHandler, Microsoft::Web::WebView2::Win32::*,
};
#[cfg(windows)]
use windows_core::{Interface, PCWSTR};

#[cfg(windows)]
pub async fn get_cookies_for_url(app: &AppHandle, url: &str) -> Result<String, String> {
    let view = crate::window::webview_manager::active_content_webview(app)
        .or_else(|| {
            app.state::<crate::state::browser_state::BrowserState>()
                .snapshot_tabs()
                .first()
                .and_then(|t| {
                    app.get_webview(&crate::state::tab_model::webview_label_for_tab(&t.id))
                })
        })
        .or_else(|| app.get_webview("main"))
        .ok_or_else(|| "no_webview_available".to_string())?;

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let sender = Arc::new(Mutex::new(Some(sender)));
    let uri: Vec<u16> = url.encode_utf16().chain(Some(0)).collect();

    let sender_for_webview = sender.clone();
    let result = view.with_webview(move |platform| {
        let op = (|| -> windows_core::Result<()> {
            let core = unsafe { platform.controller().CoreWebView2()? };
            let core2 = core.cast::<ICoreWebView2_2>()?;
            let cm = unsafe { core2.CookieManager()? };
            let callback_sender = sender_for_webview.clone();

            let handler = GetCookiesCompletedHandler::create(Box::new(move |hr, list| {
                let parse_result = (|| -> windows_core::Result<String> {
                    hr?;
                    let mut parts = Vec::new();
                    if let Some(list) = list {
                        let mut count = 0;
                        unsafe {
                            list.Count(&mut count)?;
                            for i in 0..count {
                                let c = list.GetValueAtIndex(i)?;
                                let mut name = Default::default();
                                let mut value = Default::default();
                                c.Name(&mut name)?;
                                c.Value(&mut value)?;
                                let n = CoTaskMemPWSTR::from(name).to_string();
                                let v = CoTaskMemPWSTR::from(value).to_string();
                                if !n.is_empty() {
                                    parts.push(format!("{n}={v}"));
                                }
                            }
                        }
                    }
                    Ok(parts.join("; "))
                })();

                if let Ok(mut lock) = callback_sender.lock() {
                    if let Some(tx) = lock.take() {
                        let _ = tx.send(parse_result.map_err(|e| e.to_string()));
                    }
                }
                Ok(())
            }));

            unsafe {
                cm.GetCookies(PCWSTR(uri.as_ptr()), &handler)?;
            }
            Ok(())
        })();

        if let Err(e) = op {
            if let Ok(mut lock) = sender_for_webview.lock() {
                if let Some(tx) = lock.take() {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        }
    });

    if let Err(e) = result {
        return Err(e.to_string());
    }

    match tokio::time::timeout(std::time::Duration::from_secs(5), receiver).await {
        Ok(Ok(Ok(cookie_header))) => Ok(cookie_header),
        Ok(Ok(Err(e))) => Err(e),
        Ok(Err(_)) => Err("channel_closed".into()),
        Err(_) => Err("cookie_timeout".into()),
    }
}

#[cfg(not(windows))]
pub async fn get_cookies_for_url(_app: &AppHandle, _url: &str) -> Result<String, String> {
    Ok(String::new())
}
