#[cfg(windows)]
use crate::state::browser_state::BrowserState;
#[cfg(windows)]
use tauri::{AppHandle, Manager};

/// Replace our owned future-document script, then update the live document without navigation.
#[cfg(windows)]
pub async fn apply_content_policy(app: &AppHandle, enabled: bool) -> Result<(), String> {
    use webview2_com::{AddScriptToExecuteOnDocumentCreatedCompletedHandler, CoTaskMemPWSTR};
    let state = app.state::<BrowserState>();
    let tokens = state
        .keyboard_bridge_tokens
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let exceptions = state.shield_settings.lock().map_err(|e| e.to_string())?.exceptions.clone();
    for (label, token) in tokens {
        let Some(view) = app.get_webview(&label) else {
            continue;
        };
        let script = format!(
            "window.__veloceSetShieldPolicy?.({}, {enabled});",
            serde_json::to_string(&token).map_err(|e| e.to_string())?
        );
        let old = state
            .shield_script_ids
            .lock()
            .map_err(|e| e.to_string())?
            .get(&label)
            .cloned();
        let (send, receive) = tokio::sync::oneshot::channel();
        let send = std::sync::Arc::new(std::sync::Mutex::new(Some(send)));
        let initial_send = send.clone();
        let initial_script =
            crate::privacy::adblock_engine::build_replacement_shield_script_with_exceptions(enabled, &token, &exceptions);
        view.with_webview(move |platform| {
            let result = (|| {
                let core = unsafe { platform.controller().CoreWebView2()? };
                let callback = AddScriptToExecuteOnDocumentCreatedCompletedHandler::create(
                    Box::new(move |status, id| {
                        let result = status.map(|()| id).map_err(|e| e.to_string());
                        if let Ok(mut send) = send.lock() {
                            if let Some(send) = send.take() {
                                let _ = send.send(result);
                            }
                        }
                        Ok(())
                    }),
                );
                let source = CoTaskMemPWSTR::from(initial_script.as_str());
                unsafe {
                    core.AddScriptToExecuteOnDocumentCreated(
                        *source.as_ref().as_pcwstr(),
                        &callback,
                    )?;
                }
                Ok::<(), webview2_com::Error>(())
            })();
            if let Err(error) = result {
                if let Ok(mut send) = initial_send.lock() {
                    if let Some(send) = send.take() {
                        let _ = send.send(Err(error.to_string()));
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;
        let id = receive.await.map_err(|e| e.to_string())??;
        state
            .shield_script_ids
            .lock()
            .map_err(|e| e.to_string())?
            .insert(label, id);
        if let Some(old) = old {
            let (send, receive) = tokio::sync::oneshot::channel();
            view.with_webview(move |platform| {
                let old = CoTaskMemPWSTR::from(old.as_str());
                let result = unsafe {
                    platform.controller().CoreWebView2().and_then(|core| {
                        core.RemoveScriptToExecuteOnDocumentCreated(*old.as_ref().as_pcwstr())
                    })
                }
                .map_err(|e| e.to_string());
                let _ = send.send(result);
            })
            .map_err(|e| e.to_string())?;
            receive.await.map_err(|e| e.to_string())??;
        }
        view.eval(script).map_err(|e| e.to_string())?;
    }
    Ok(())
}
