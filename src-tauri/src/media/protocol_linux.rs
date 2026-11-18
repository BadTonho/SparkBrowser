//! WebKitGTK transport for the media controller. The controller runs in the top frame only (as on
//! Windows). Commands are evaluated in that frame and answered through an authenticated script
//! message handler; the same handler carries the controller's state signals.
use super::{current, signal_message, signals};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tauri::{AppHandle, Manager};

/// The only execution context on this platform: the top-level document.
pub const MAIN_CONTEXT: &str = "main";
const HANDLER: &str = "sparkMedia";
const MAX_SIGNAL: usize = 2048;
const MAX_REPLY: usize = 64 * 1024;

#[derive(Default)]
struct View {
    token: String,
    document: Option<String>,
    sequence: u64,
}
type Reply = tokio::sync::oneshot::Sender<Option<Value>>;
fn views() -> &'static Mutex<HashMap<String, View>> {
    static VIEWS: OnceLock<Mutex<HashMap<String, View>>> = OnceLock::new();
    VIEWS.get_or_init(Default::default)
}
fn pending() -> &'static Mutex<HashMap<String, (String, Reply)>> {
    static PENDING: OnceLock<Mutex<HashMap<String, (String, Reply)>>> = OnceLock::new();
    PENDING.get_or_init(Default::default)
}
fn tab_of(label: &str) -> Option<&str> {
    label.strip_prefix("content-webview-")
}

/// Starts accepting reports from a new document and drops what the previous one reported.
fn reset_document(app: &AppHandle, label: &str) {
    if let Ok(mut views) = views().lock() {
        if let Some(view) = views.get_mut(label) {
            view.document = None;
            view.sequence = 0;
        }
    }
    if let Ok(mut pending) = pending().lock() {
        pending.retain(|_, (owner, _)| owner != label);
    }
    if let Some(id) = tab_of(label) {
        signals::remove(app, id, Some(MAIN_CONTEXT));
    }
}

fn receive(app: &AppHandle, label: &str, raw: &str) {
    if raw.len() > MAX_REPLY {
        return;
    }
    let Some(token) = views()
        .lock()
        .ok()
        .and_then(|v| v.get(label).map(|v| v.token.clone()))
    else {
        return;
    };
    if !current(app, label, &token) {
        return;
    }
    if raw.len() <= MAX_SIGNAL {
        if let Some(signal) = signal_message::parse(raw, &token) {
            let accepted = views().lock().is_ok_and(|mut views| {
                let Some(view) = views.get_mut(label) else {
                    return false;
                };
                if signal.sequence <= view.sequence
                    || view
                        .document
                        .as_deref()
                        .is_some_and(|document| document != signal.document)
                {
                    return false;
                }
                view.document = Some(signal.document.clone());
                view.sequence = signal.sequence;
                true
            });
            if let (true, Some(id)) = (accepted, tab_of(label)) {
                signals::receive(app, id, MAIN_CONTEXT, &signal);
            }
            return;
        }
    }
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return;
    };
    if value["token"] != json!(token) {
        return;
    }
    let Some(id) = value["reply"].as_str() else {
        return;
    };
    let sender = pending().lock().ok().and_then(|mut pending| {
        match pending.get(id) {
            Some((owner, _)) if owner == label => pending.remove(id),
            _ => None,
        }
        .map(|(_, sender)| sender)
    });
    if let Some(sender) = sender {
        crate::error_log::ignored(sender.send((value["ok"] == true).then(|| value["value"].clone())));
    }
}

pub fn install(app: &AppHandle, view: &tauri::Webview, token: &str) -> Result<(), String> {
    let (app, label, token) = (app.clone(), view.label().to_owned(), token.to_owned());
    view.with_webview(move |platform| {
        use javascriptcore::ValueExt as _;
        use webkit2gtk::{LoadEvent, SettingsExt, UserContentManagerExt, WebViewExt};
        let webview = platform.inner();
        if let Ok(mut views) = views().lock() {
            views.insert(
                label.clone(),
                View {
                    token,
                    ..Default::default()
                },
            );
        }
        if let Some(settings) = webview.settings() {
            // Needed by captureStream() for the floating player and volume amplification.
            settings.set_enable_media_stream(true);
            settings.set_enable_webaudio(true);
        }
        if let Some(manager) = webview.user_content_manager() {
            manager.register_script_message_handler(HANDLER);
            let (a, l) = (app.clone(), label.clone());
            manager.connect_script_message_received(Some(HANDLER), move |_, result| {
                if let Some(raw) = result.js_value().filter(|v| v.is_string()) {
                    receive(&a, &l, raw.to_str().as_str());
                }
            });
        }
        let l = label.clone();
        webview.connect_load_changed(move |_, event| {
            if event == LoadEvent::Started {
                reset_document(&app, &l);
            }
        });
    })
    .map_err(|_| "media.unavailable".into())
}

pub fn clear(app: &AppHandle, label: &str) {
    if let Ok(mut views) = views().lock() {
        views.remove(label);
    }
    if let Ok(mut pending) = pending().lock() {
        pending.retain(|_, (owner, _)| owner != label);
    }
    if let Some(id) = tab_of(label) {
        signals::remove(app, id, None);
    }
}

pub async fn contexts(app: &AppHandle, id: &str) -> Result<(String, Vec<String>), String> {
    let label = crate::state::tab_model::webview_label_for_tab(id);
    app.get_webview(&label).ok_or("media.unavailable")?;
    let token = views()
        .lock()
        .ok()
        .and_then(|v| v.get(&label).map(|v| v.token.clone()))
        .ok_or("media.unavailable")?;
    Ok((token, vec![MAIN_CONTEXT.to_owned()]))
}

pub async fn evaluate(
    app: &AppHandle,
    id: &str,
    token: &str,
    context: &str,
    operation: Value,
    _user_gesture: bool,
) -> Result<Value, String> {
    let label = crate::state::tab_model::webview_label_for_tab(id);
    let view = app.get_webview(&label).ok_or("media.unavailable")?;
    let known = views()
        .lock()
        .is_ok_and(|v| v.get(&label).is_some_and(|v| v.token == token));
    if context != MAIN_CONTEXT || !known || !current(app, &label, token) {
        return Err("media.stale".into());
    }
    let request = crate::window::webview_manager::create_internal_bridge_token();
    let (tx, rx) = tokio::sync::oneshot::channel();
    pending()
        .lock()
        .map_err(|_| "media.error")?
        .insert(request.clone(), (label.clone(), tx));
    let (token_json, request_json) = (json!(token).to_string(), json!(request).to_string());
    let script = format!(
        "(async()=>{{let ok=true,value=null;try{{value=(await window.__sparkMedia?.({token_json},{operation}))??null}}catch(e){{ok=false}}\
         window.webkit.messageHandlers.{HANDLER}.postMessage(JSON.stringify({{reply:{request_json},token:{token_json},ok,value}}))}})()"
    );
    let sent = view.with_webview(move |platform| {
        use webkit2gtk::WebViewExt;
        // `evaluate_javascript` needs WebKitGTK 2.40; `run_javascript` works from 2.22.
        #[allow(deprecated)]
        platform
            .inner()
            .run_javascript(&script, None::<&webkit2gtk::gio::Cancellable>, |_| {});
    });
    let outcome = if sent.is_err() {
        Err("media.unavailable")
    } else {
        match tokio::time::timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(Some(value))) => Ok(value),
            Ok(Ok(None)) | Err(_) => Err("media.error"),
            Ok(Err(_)) => Err("media.stale"),
        }
    };
    if let Ok(mut pending) = pending().lock() {
        pending.remove(&request);
    }
    let value = outcome?;
    if !current(app, &label, token) {
        return Err("media.stale".into());
    }
    if !value["exceptionDetails"].is_null() {
        Err("media.error".into())
    } else if value.is_null() {
        Err("media.unavailable".into())
    } else {
        Ok(value)
    }
}
