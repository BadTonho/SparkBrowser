use super::*;

#[cfg(windows)]
async fn get_webview_cookies_for_url(
    app: &AppHandle,
    preferred_tab_id: Option<&str>,
    url: &str,
) -> Result<Vec<(String, String)>, String> {
    use std::sync::Arc;
    use webview2_com::{
        CoTaskMemPWSTR, GetCookiesCompletedHandler, Microsoft::Web::WebView2::Win32::*,
    };
    use windows_core::{Interface, PCWSTR};

    let id = preferred_tab_id.ok_or("backup.sync_gdrive_login_required")?;
    let view = app
        .get_webview(&crate::state::tab_model::webview_label_for_tab(id))
        .ok_or("backup.sync_gdrive_login_required")?;

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let sender = Arc::new(Mutex::new(Some(sender)));
    let uri: Vec<u16> = url.encode_utf16().chain(Some(0)).collect();

    let sender_for_webview = sender.clone();
    view.with_webview(move |platform| {
        let op = (|| -> windows_core::Result<()> {
            let core = unsafe { platform.controller().CoreWebView2()? };
            let core2 = core.cast::<ICoreWebView2_2>()?;
            let cm = unsafe { core2.CookieManager()? };
            let callback_sender = sender_for_webview.clone();

            let handler = GetCookiesCompletedHandler::create(Box::new(move |hr, list| {
                let parse_result = (|| -> windows_core::Result<Vec<(String, String)>> {
                    hr?;
                    let mut pairs = Vec::new();
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
                                    pairs.push((n, v));
                                }
                            }
                        }
                    }
                    Ok(pairs)
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
    })
    .map_err(|e| e.to_string())?;

    match tokio::time::timeout(Duration::from_secs(5), receiver).await {
        Ok(Ok(Ok(pairs))) => Ok(pairs),
        Ok(Ok(Err(e))) => Err(format!("backup.sync_network_failed: {e}")),
        _ => Err("backup.sync_network_failed: Cookie manager timeout".into()),
    }
}

#[cfg(windows)]
pub(super) async fn collect_google_cookies(
    app: &AppHandle,
    preferred_tab_id: Option<&str>,
) -> Result<(String, HashMap<String, String>), String> {
    let mut map = HashMap::new();
    for domain_url in [
        "https://drive.google.com",
        "https://clients6.google.com",
        "https://mail.google.com",
    ] {
        if let Ok(pairs) = get_webview_cookies_for_url(app, preferred_tab_id, domain_url).await {
            for (k, v) in pairs {
                map.insert(k, v);
            }
        }
    }
    let has_sapisid = map.contains_key("SAPISID")
        || map.contains_key("__Secure-3PAPISID")
        || map.contains_key("__Secure-1PAPISID");
    let has_sid = map.contains_key("SID")
        || map.contains_key("__Secure-1PSID")
        || map.contains_key("__Secure-3PSID");
    if !has_sapisid || !has_sid {
        return Err("backup.sync_gdrive_login_required".into());
    }
    let mut pairs: Vec<_> = map.iter().map(|(k, v)| format!("{k}={v}")).collect();
    pairs.sort();
    Ok((pairs.join("; "), map))
}
