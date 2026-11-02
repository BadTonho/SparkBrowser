//! Fixed media operations in document execution contexts. COM never leaves the UI thread.
use super::current;
use serde_json::{json, Value};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{AppHandle, Manager};
use webview2_com::{
    CallDevToolsProtocolMethodCompletedHandler, CoTaskMemPWSTR,
    DevToolsProtocolEventReceivedEventHandler,
};
use windows_core::PCWSTR;

#[derive(Clone)]
struct Context {
    unique: String,
}
type ViewContexts = HashMap<String, (String, HashMap<i64, Context>)>;
thread_local! { static CONTEXTS: RefCell<ViewContexts> = RefCell::new(HashMap::new()); }
pub fn clear(label: &str) {
    CONTEXTS.with(|m| {
        m.borrow_mut().remove(label);
    });
}

pub fn install(app: &AppHandle, view: &tauri::Webview, token: &str) -> Result<(), String> {
    let (app, label, token) = (app.clone(), view.label().to_string(), token.to_string());
    view.with_webview(move |p| unsafe {
        let Ok(core) = p.controller().CoreWebView2() else {
            return;
        };
        CONTEXTS.with(|m| {
            m.borrow_mut()
                .insert(label.clone(), (token.clone(), HashMap::new()))
        });
        for event in [
            "Runtime.executionContextCreated",
            "Runtime.executionContextDestroyed",
            "Runtime.executionContextsCleared",
        ] {
            let event_name: Vec<u16> = event.encode_utf16().chain(Some(0)).collect();
            let Ok(receiver) = core.GetDevToolsProtocolEventReceiver(PCWSTR(event_name.as_ptr()))
            else {
                continue;
            };
            let (a, l, t) = (app.clone(), label.clone(), token.clone());
            let _ = receiver.add_DevToolsProtocolEventReceived(
                &DevToolsProtocolEventReceivedEventHandler::create(Box::new(move |_, args| {
                    if !current(&a, &l, &t) {
                        return Ok(());
                    }
                    if let Some(args) = args {
                        let mut raw = Default::default();
                        args.ParameterObjectAsJson(&mut raw)?;
                        if let Ok(v) =
                            serde_json::from_str::<Value>(&CoTaskMemPWSTR::from(raw).to_string())
                        {
                            CONTEXTS.with(|m| {
                                let mut m = m.borrow_mut();
                                let Some((_, contexts)) = m.get_mut(&l) else {
                                    return;
                                };
                                match event {
                                    "Runtime.executionContextsCleared" => contexts.clear(),
                                    "Runtime.executionContextDestroyed" => {
                                        if let Some(id) = v["executionContextId"].as_i64() {
                                            contexts.remove(&id);
                                        }
                                    }
                                    _ => {
                                        let c = &v["context"];
                                        if c["auxData"]["isDefault"] == true && contexts.len() < 64
                                        {
                                            if let (Some(id), Some(unique)) =
                                                (c["id"].as_i64(), c["uniqueId"].as_str())
                                            {
                                                contexts.insert(
                                                    id,
                                                    Context {
                                                        unique: unique.into(),
                                                    },
                                                );
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                    Ok(())
                })),
                &mut 0,
            );
        }
        let method: Vec<u16> = "Runtime.enable\0".encode_utf16().collect();
        let params: Vec<u16> = "{}\0".encode_utf16().collect();
        let _ = core.CallDevToolsProtocolMethod(
            PCWSTR(method.as_ptr()),
            PCWSTR(params.as_ptr()),
            &CallDevToolsProtocolMethodCompletedHandler::create(Box::new(|_, _| Ok(()))),
        );
    })
    .map_err(|_| "media.unavailable".into())
}

pub async fn contexts(app: &AppHandle, id: &str) -> Result<(String, Vec<String>), String> {
    let label = crate::state::tab_model::webview_label_for_tab(id);
    let view = app.get_webview(&label).ok_or("media.unavailable")?;
    let (tx, rx) = tokio::sync::oneshot::channel();
    view.with_webview(move |_| {
        let value = CONTEXTS.with(|m| {
            m.borrow()
                .get(&label)
                .map(|(t, c)| (t.clone(), c.values().map(|v| v.unique.clone()).collect()))
        });
        let _ = tx.send(value);
    })
    .map_err(|_| "media.unavailable")?;
    tokio::time::timeout(Duration::from_secs(5), rx)
        .await
        .map_err(|_| "media.error")?
        .map_err(|_| "media.error")?
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
        if !current(&a,&label,&t) {send(Err("media.stale".into()));return;}
        let expression=format!("window.__veloceMedia?.({}, {}) ?? null",serde_json::to_string(&t).unwrap_or_default(),operation);
        let params=json!({"expression":expression,"uniqueContextId":c,"returnByValue":true,"awaitPromise":true,"userGesture":user_gesture,"allowUnsafeEvalBlockedByCSP":false});
        let raw:Vec<u16>=params.to_string().encode_utf16().chain(Some(0)).collect();
        let method:Vec<u16>="Runtime.evaluate\0".encode_utf16().collect();
        let (cb,a,l,t)=(tx.clone(),a.clone(),label.clone(),t.clone());
        let handler=CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |result,raw| {
            let value=if !current(&a,&l,&t) || result.is_err() {Err("media.stale".into())} else {
                let v:Value=serde_json::from_str(&raw.to_string()).unwrap_or(Value::Null);
                if !v["exceptionDetails"].is_null() {Err("media.error".into())} else if v["result"]["value"].is_null(){Err("media.unavailable".into())} else {Ok(v["result"]["value"].clone())}
            };
            if let Ok(mut slot)=cb.lock(){if let Some(tx)=slot.take(){let _=tx.send(value);}}
            Ok(())
        }));
        let result=p.controller().CoreWebView2().and_then(|core|core.CallDevToolsProtocolMethod(PCWSTR(method.as_ptr()),PCWSTR(raw.as_ptr()),&handler));
        if result.is_err(){send(Err("media.unavailable".into()));}
    }).map_err(|_|"media.unavailable")?;
    tokio::time::timeout(Duration::from_secs(5), rx)
        .await
        .map_err(|_| "media.error")?
        .map_err(|_| "media.error")?
}
