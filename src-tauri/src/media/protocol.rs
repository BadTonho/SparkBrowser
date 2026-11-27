//! Fixed media operations in document execution contexts. COM never leaves the UI thread.
use super::current;
use crate::sync::Mutex;
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tauri::{AppHandle, Manager};
use webview2_com::{
    CallDevToolsProtocolMethodCompletedHandler, Microsoft::Web::WebView2::Win32::*,
};
use windows_core::{Interface, PCWSTR};

pub use super::contexts::{clear, install};
fn valid_context(label: &str, token: &str, context: &str) -> bool {
    super::contexts::session(label, token, context).is_some()
}
pub fn call(
    core: &ICoreWebView2,
    session: &str,
    method: &str,
    params: &Value,
    handler: &ICoreWebView2CallDevToolsProtocolMethodCompletedHandler,
) -> windows_core::Result<()> {
    let method: Vec<u16> = method.encode_utf16().chain(Some(0)).collect();
    let params: Vec<u16> = params.to_string().encode_utf16().chain(Some(0)).collect();
    unsafe {
        if session.is_empty() {
            core.CallDevToolsProtocolMethod(
                PCWSTR(method.as_ptr()),
                PCWSTR(params.as_ptr()),
                handler,
            )
        } else {
            let session: Vec<u16> = session.encode_utf16().chain(Some(0)).collect();
            core.cast::<ICoreWebView2_11>()?
                .CallDevToolsProtocolMethodForSession(
                    PCWSTR(session.as_ptr()),
                    PCWSTR(method.as_ptr()),
                    PCWSTR(params.as_ptr()),
                    handler,
                )
        }
    }
}

pub async fn contexts(app: &AppHandle, id: &str) -> Result<(String, Vec<String>), String> {
    let label = crate::state::tab_model::webview_label_for_tab(id);
    let view = app.get_webview(&label).ok_or("media.unavailable")?;
    let (tx, rx) = tokio::sync::oneshot::channel();
    view.with_webview(move |_| {
        let value = super::contexts::snapshot(&label);
        crate::error_log::ignored(tx.send(value));
    })
    .map_err(|_| "media.unavailable")?;
    tokio::time::timeout(Duration::from_secs(5), rx)
        .await
        .map_err(|_| "media.error: timed out reading WebView2 media contexts")?
        .map_err(|_| "media.error: WebView2 media context response was lost")?
        .ok_or_else(|| "media.unavailable".into())
}

pub async fn evaluate(
    app: &AppHandle,
    id: &str,
    token: &str,
    context: &str,
    operation: Value,
    user_gesture: bool,
) -> Result<Value, String> {
    let label = crate::state::tab_model::webview_label_for_tab(id);
    let view = app.get_webview(&label).ok_or("media.unavailable")?;
    let (a, t, c) = (app.clone(), token.to_string(), context.to_string());
    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));
    view.with_webview(move |p| unsafe {
        let send=|result| {if let Ok(mut slot)=tx.lock(){if let Some(tx)=slot.take(){let _=tx.send(result);}}};
        if !current(&a,&label,&t) || !valid_context(&label,&t,&c) {send(Err("media.stale".into()));return;}
        let expression=format!("window.__sparkMedia?.({}, {}) ?? null",serde_json::to_string(&t).unwrap_or_default(),operation);
        let params=json!({"expression":expression,"uniqueContextId":c,"returnByValue":true,"awaitPromise":true,"userGesture":user_gesture,"allowUnsafeEvalBlockedByCSP":false});
        let session=super::contexts::session(&label,&t,&c).unwrap_or_default();
        let (cb,a,l,t,c)=(tx.clone(),a.clone(),label.clone(),t.clone(),c.clone());
        let handler=CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |result,raw| {
            let value=if !current(&a,&l,&t) || !valid_context(&l,&t,&c) || result.is_err() {Err("media.stale".into())} else {
                let v:Value=serde_json::from_str(&raw.to_string()).unwrap_or(Value::Null);
                if !v["exceptionDetails"].is_null() {Err("media.error: WebView2 media controller threw an exception".into())} else if v["result"]["value"].is_null(){Err("media.unavailable".into())} else {Ok(v["result"]["value"].clone())}
            };
            if let Ok(mut slot)=cb.lock(){if let Some(tx)=slot.take(){let _=tx.send(value);}}
            Ok(())
        }));
        let result=p.controller().CoreWebView2().and_then(|core|call(&core,&session,"Runtime.evaluate",&params,&handler));
        if result.is_err(){send(Err("media.unavailable".into()));}
    }).map_err(|_|"media.unavailable")?;
    tokio::time::timeout(Duration::from_secs(5), rx)
        .await
        .map_err(|_| "media.error: timed out waiting for the WebView2 media controller")?
        .map_err(|_| "media.error: WebView2 media controller response was lost")?
}
