//! Document contexts and out-of-process iframe sessions, retained only on the UI thread.
use super::{current, protocol::call, signals};
use serde_json::{json, Value};
use std::{cell::RefCell, collections::HashMap};
use tauri::AppHandle;
use webview2_com::{
    CallDevToolsProtocolMethodCompletedHandler, CoTaskMemPWSTR,
    DevToolsProtocolEventReceivedEventHandler, Microsoft::Web::WebView2::Win32::*,
};
use windows_core::{Interface, PCWSTR};

#[derive(Clone)]
struct Context {
    unique: String,
    session: String,
}
#[derive(Default)]
struct ViewContexts {
    token: String,
    contexts: HashMap<(String, i64), Context>,
}
thread_local! {static VIEWS:RefCell<HashMap<String,ViewContexts>>=RefCell::new(HashMap::new());}

/// Counts only: no context identifiers or document metadata leave the UI thread.
pub async fn resource_counts(app: &AppHandle) -> Result<(usize, usize), String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let counts = VIEWS.with(|views| {
            let views = views.borrow();
            (views.len(), views.values().map(|v| v.contexts.len()).sum())
        });
        let _ = tx.send(counts);
    })
    .map_err(|e| e.to_string())?;
    tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
pub fn session(label: &str, token: &str, unique: &str) -> Option<String> {
    VIEWS.with(|m| {
        let m = m.borrow();
        let v = m.get(label)?;
        if v.token != token {
            return None;
        }
        v.contexts
            .values()
            .find(|c| c.unique == unique)
            .map(|c| c.session.clone())
    })
}
pub fn snapshot(label: &str) -> Option<(String, Vec<String>)> {
    VIEWS.with(|m| {
        m.borrow().get(label).map(|v| {
            (
                v.token.clone(),
                v.contexts.values().map(|c| c.unique.clone()).collect(),
            )
        })
    })
}
pub fn clear(app: &AppHandle, label: &str) {
    VIEWS.with(|m| m.borrow_mut().remove(label));
    if let Some(id) = label.strip_prefix("content-webview-") {
        signals::remove(app, id, None);
    }
}
fn remove_session(app: &AppHandle, label: &str, session: &str) {
    let removed = VIEWS.with(|m| {
        let mut m = m.borrow_mut();
        let Some(v) = m.get_mut(label) else {
            return Vec::new();
        };
        let removed: Vec<_> = v
            .contexts
            .values()
            .filter(|c| c.session == session)
            .map(|c| c.unique.clone())
            .collect();
        v.contexts.retain(|_, c| c.session != session);
        removed
    });
    if let Some(id) = label.strip_prefix("content-webview-") {
        for c in removed {
            signals::remove(app, id, Some(&c));
        }
    }
}
fn enable(core: &ICoreWebView2, session: &str) {
    for (method, params) in [
        ("Runtime.addBinding", json!({"name":"__veloceMediaSignal"})),
        ("Runtime.enable", json!({})),
        (
            "Target.setAutoAttach",
            json!({"autoAttach":true,"waitForDebuggerOnStart":false,"flatten":true,"filter":[{"type":"iframe","exclude":false},{"exclude":true}]}),
        ),
    ] {
        let _ = call(
            core,
            session,
            method,
            &params,
            &CallDevToolsProtocolMethodCompletedHandler::create(Box::new(|_, _| Ok(()))),
        );
    }
}
pub fn install(app: &AppHandle, view: &tauri::Webview, token: &str) -> Result<(), String> {
    let (app, label, token) = (app.clone(), view.label().to_string(), token.to_string());
    view.with_webview(move |p| unsafe {
        let Ok(core) = p.controller().CoreWebView2() else {
            return;
        };
        VIEWS.with(|m| {
            m.borrow_mut().insert(
                label.clone(),
                ViewContexts {
                    token: token.clone(),
                    ..Default::default()
                },
            )
        });
        for event in [
            "Runtime.executionContextCreated",
            "Runtime.executionContextDestroyed",
            "Runtime.executionContextsCleared",
            "Runtime.bindingCalled",
            "Target.attachedToTarget",
            "Target.detachedFromTarget",
        ] {
            let name: Vec<u16> = event.encode_utf16().chain(Some(0)).collect();
            let Ok(receiver) = core.GetDevToolsProtocolEventReceiver(PCWSTR(name.as_ptr())) else {
                continue;
            };
            let (a, l, t) = (app.clone(), label.clone(), token.clone());
            let _ = receiver.add_DevToolsProtocolEventReceived(
                &DevToolsProtocolEventReceivedEventHandler::create(Box::new(
                    move |sender, args| {
                        if !current(&a, &l, &t) {
                            return Ok(());
                        }
                        let Some(args) = args else {
                            return Ok(());
                        };
                        let mut raw = Default::default();
                        args.ParameterObjectAsJson(&mut raw)?;
                        let Ok(value) =
                            serde_json::from_str::<Value>(&CoTaskMemPWSTR::from(raw).to_string())
                        else {
                            return Ok(());
                        };
                        let session = if let Ok(args) =
                            args.cast::<ICoreWebView2DevToolsProtocolEventReceivedEventArgs2>()
                        {
                            let mut raw = Default::default();
                            args.SessionId(&mut raw)?;
                            CoTaskMemPWSTR::from(raw).to_string()
                        } else {
                            String::new()
                        };
                        match event {
                            "Target.attachedToTarget" => {
                                if value["targetInfo"]["type"] == "iframe" {
                                    if let (Some(core), Some(session)) =
                                        (sender, value["sessionId"].as_str())
                                    {
                                        enable(&core, session);
                                    }
                                }
                            }
                            "Target.detachedFromTarget" => {
                                if let Some(session) = value["sessionId"].as_str() {
                                    remove_session(&a, &l, session);
                                }
                            }
                            "Runtime.executionContextsCleared" => remove_session(&a, &l, &session),
                            "Runtime.executionContextCreated" => {
                                let c = &value["context"];
                                if c["auxData"]["isDefault"] == true {
                                    if let (Some(id), Some(unique)) =
                                        (c["id"].as_i64(), c["uniqueId"].as_str())
                                    {
                                        VIEWS.with(|m| {
                                            if let Some(v) = m.borrow_mut().get_mut(&l) {
                                                if v.contexts.len() < 64 {
                                                    v.contexts.insert(
                                                        (session.clone(), id),
                                                        Context {
                                                            unique: unique.into(),
                                                            session,
                                                        },
                                                    );
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                            "Runtime.executionContextDestroyed" => {
                                if let Some(id) = value["executionContextId"].as_i64() {
                                    let removed = VIEWS.with(|m| {
                                        m.borrow_mut()
                                            .get_mut(&l)
                                            .and_then(|v| v.contexts.remove(&(session, id)))
                                    });
                                    if let (Some(c), Some(id)) =
                                        (removed, l.strip_prefix("content-webview-"))
                                    {
                                        signals::remove(&a, id, Some(&c.unique));
                                    }
                                }
                            }
                            "Runtime.bindingCalled" => {
                                if value["name"] != "__veloceMediaSignal" {
                                    return Ok(());
                                }
                                let context = VIEWS.with(|m| {
                                    m.borrow().get(&l).and_then(|v| {
                                        value["executionContextId"]
                                            .as_i64()
                                            .and_then(|id| v.contexts.get(&(session, id)).cloned())
                                    })
                                });
                                if let (Some(c), Some(raw), Some(id)) = (
                                    context,
                                    value["payload"].as_str().filter(|s| s.len() <= 2048),
                                    l.strip_prefix("content-webview-"),
                                ) {
                                    if let Ok(payload) = serde_json::from_str::<Value>(raw) {
                                        if payload["token"] == t {
                                            signals::receive(&a, id, &c.unique, &payload);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        Ok(())
                    },
                )),
                &mut 0,
            );
        }
        enable(&core, "");
    })
    .map_err(|_| "media.unavailable".into())
}
