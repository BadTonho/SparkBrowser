//! A single, document-bound floating player. Its external WebView has no IPC grants.
use super::{current, MediaService, TabMediaState};
use crate::state::tab_model::webview_label_for_tab;
use crate::window::webview_manager::create_internal_bridge_token;
use serde_json::json;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{
    webview::{NewWindowFeatures, NewWindowResponse},
    AppHandle, Manager, Url, WebviewWindowBuilder,
};

#[derive(Clone)]
struct Session {
    id: String,
    tab: String,
    token: String,
    context: String,
    document: String,
    document_key: String,
    created: Instant,
    opened: bool,
    sequence: u64,
    acknowledged: u64,
    normal_size: Option<tauri::PhysicalSize<u32>>,
    restore_size: Option<tauri::PhysicalSize<u32>>,
}
static SESSION: Mutex<Option<Session>> = Mutex::new(None);
fn label(id: &str) -> String {
    format!("floating-video-{id}")
}

pub fn prepare(
    tab: &str,
    token: &str,
    context: &str,
    document: &str,
    document_key: &str,
) -> String {
    let id = create_internal_bridge_token();
    *SESSION.lock().expect("PiP session") = Some(Session {
        id: id.clone(),
        tab: tab.into(),
        token: token.into(),
        context: context.into(),
        document: document.into(),
        document_key: document_key.into(),
        created: Instant::now(),
        opened: false,
        sequence: 0,
        acknowledged: 0,
        normal_size: None,
        restore_size: None,
    });
    id
}

pub fn new_window(
    app: &AppHandle,
    tab: &str,
    token: &str,
    url: &Url,
    features: NewWindowFeatures,
) -> NewWindowResponse<tauri::Wry> {
    let session = {
        let Ok(mut state) = SESSION.lock() else {
            return NewWindowResponse::Deny;
        };
        let Some(s) = state.as_mut() else {
            return NewWindowResponse::Deny;
        };
        if s.opened
            || s.tab != tab
            || s.token != token
            || s.created.elapsed() > Duration::from_secs(5)
            || !current(app, &webview_label_for_tab(tab), token)
            || url.as_str() != format!("about:blank#veloce-pip-{}", s.id)
        {
            return NewWindowResponse::Deny;
        }
        s.opened = true;
        s.clone()
    };
    let a = app.clone();
    let id = session.id.clone();
    #[allow(unused_mut)]
    let mut window_builder = WebviewWindowBuilder::new(
        app,
        label(&session.id),
        tauri::WebviewUrl::External(url.clone()),
    );
    #[cfg(windows)]
    {
        window_builder = window_builder.browser_extensions_enabled(true);
    }
    let window = window_builder
        .window_features(features)
        .title("Floating video")
        .decorations(false)
        .shadow(
            app.state::<MediaService>()
                .pip_window_shadow
                .load(std::sync::atomic::Ordering::Relaxed),
        )
        .maximizable(false)
        .inner_size(400.0, 225.0)
        .min_inner_size(280.0, 158.0)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .resizable(true)
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_navigation(move |url| {
            if url.scheme() == "veloce-internal" {
                navigation(&a, &id, url);
                return false;
            }
            url.as_str() == format!("about:blank#veloce-pip-{id}") || url.as_str() == "about:blank"
        })
        .build();
    match window {
        Ok(window) => {
            if let Some(view) = app.get_webview(window.label()) {
                if crate::security::harden(app, &view, true).is_err() {
                    let _ = window.destroy();
                    return NewWindowResponse::Deny;
                }
            }
            if let Ok(mut state) = SESSION.lock() {
                if let Some(s) = state.as_mut().filter(|s| s.id == session.id) {
                    s.normal_size = window.inner_size().ok();
                }
            }
            let a = app.clone();
            let id = session.id;
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Resized(size) = event {
                    if let Some(window) = a.get_webview_window(&label(&id)) {
                        remember_size(&window, &id, *size);
                    }
                }
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Some(window) = a.get_webview_window(&label(&id)) {
                        let _ = window.eval("window.__velocePipRequestClose?.();");
                    }
                }
            });
            NewWindowResponse::Create { window }
        }
        Err(_) => NewWindowResponse::Deny,
    }
}

/// Called only by the shell while holding the shared media operation lock.
pub async fn set_shadow(app: &AppHandle, enabled: bool) -> Result<bool, String> {
    let (send, receive) = tokio::sync::oneshot::channel();
    let a = app.clone();
    app.run_on_main_thread(move || {
        let _ = send.send(apply_shadow(&a, enabled));
    })
    .map_err(|_| "media.error")?;
    receive.await.map_err(|_| "media.error")?
}

fn apply_shadow(app: &AppHandle, enabled: bool) -> Result<bool, String> {
    let session = SESSION.lock().map_err(|_| "media.error")?.clone();
    if let Some(session) = session {
        if current(app, &webview_label_for_tab(&session.tab), &session.token) {
            if let Some(window) = app.get_webview_window(&label(&session.id)) {
                let minimized = window.is_minimized().map_err(|_| "media.error")?;
                let size = if minimized {
                    session.normal_size
                } else {
                    Some(window.inner_size().map_err(|_| "media.error")?)
                };
                window.set_shadow(enabled).map_err(|_| "media.error")?;
                if let Some(size) = size {
                    if minimized {
                        if let Some(s) = SESSION
                            .lock()
                            .map_err(|_| "media.error")?
                            .as_mut()
                            .filter(|s| s.id == session.id)
                        {
                            s.restore_size = Some(size);
                        }
                    } else if window.set_size(size).is_err() {
                        let previous = app
                            .state::<MediaService>()
                            .pip_window_shadow
                            .load(std::sync::atomic::Ordering::Relaxed);
                        let _ = window.set_shadow(previous);
                        let _ = window.set_size(size);
                        return Err("media.error".into());
                    }
                }
            }
        }
    }
    app.state::<MediaService>()
        .pip_window_shadow
        .store(enabled, std::sync::atomic::Ordering::Relaxed);
    Ok(enabled)
}

// Windows changes the hidden frame insets with the shadow. Restore the saved
// client area after unminimizing, without activating or recreating the player.
fn remember_size(window: &tauri::WebviewWindow, id: &str, size: tauri::PhysicalSize<u32>) {
    if size.width == 0 || size.height == 0 || window.is_minimized().unwrap_or(true) {
        return;
    }
    let restore = {
        let Ok(mut state) = SESSION.lock() else {
            return;
        };
        let Some(s) = state.as_mut().filter(|s| s.id == id) else {
            return;
        };
        let restore = s.restore_size.take();
        s.normal_size = Some(restore.unwrap_or(size));
        restore
    };
    if let Some(size) = restore {
        let _ = window.set_size(size);
    }
}

fn emit(app: &AppHandle, id: &str, action: &str, value: Option<u16>) {
    let session = SESSION
        .lock()
        .ok()
        .and_then(|s| s.clone())
        .filter(|s| s.id == id);
    let Some(s) = session else {
        return;
    };
    if !current(app, &webview_label_for_tab(&s.tab), &s.token) {
        return;
    }
    let _ = crate::security::emit_shell(
        app,
        "media-pip-action",
        json!({"session":s.id,"sequence":s.sequence,"tabId":s.tab,"documentKey":s.document_key,"action":action,"value":value}),
    );
}

fn navigation(app: &AppHandle, id: &str, url: &Url) {
    if url.host_str() != Some("pip-player") {
        return;
    }
    let Some(parameters) = crate::security::bridge::query(url, &["session", "sequence", "value"])
    else {
        return;
    };
    let query = |key: &str| parameters.get(key).cloned();
    let Some(sequence) = query("sequence").and_then(|s| s.parse::<u64>().ok()) else {
        return;
    };
    let action = url.path().trim_matches('/');
    let value = query("value")
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|v| *v <= 200);
    if !matches!(
        action,
        "ready" | "volume" | "flush" | "return" | "close" | "capture-failed" | "minimize" | "drag"
    ) || ((action == "volume" || parameters.contains_key("value")) && value.is_none())
    {
        return;
    }
    {
        let Ok(mut state) = SESSION.lock() else {
            return;
        };
        let Some(s) = state.as_mut() else {
            return;
        };
        if s.id != id
            || query("session").as_deref() != Some(id)
            || sequence <= s.sequence
            || !current(app, &webview_label_for_tab(&s.tab), &s.token)
            || !app
                .state::<MediaService>()
                .tab(&s.tab)
                .frames
                .iter()
                .any(|f| {
                    f.context == s.context
                        && f.document == s.document
                        && (action == "ready" || f.pip_session.as_deref() == Some(id))
                })
        {
            return;
        }
        s.sequence = sequence;
    }
    if action == "ready" {
        if let Some(window) = app.get_webview_window(&label(id)) {
            // WebView2 attaches the supplied popup after window construction.
            // Reattach Tauri's frameless resize region above that content view.
            let _ = window.set_resizable(false);
            let _ = window.set_resizable(true);
            let _ = window.show();
        }
        let session = SESSION.lock().ok().and_then(|s| s.clone());
        if let Some(s) = session {
            sync(app, &app.state::<MediaService>().tab(&s.tab));
        }
    } else if matches!(action, "minimize" | "drag") {
        if let Some(window) = app.get_webview_window(&label(id)) {
            if action == "minimize" {
                let _ = window.minimize();
            } else {
                let _ = window.start_dragging();
                let _ = window.eval("window.__velocePipDragFinished?.();");
            }
        }
    } else {
        emit(app, id, action, value);
    }
}

pub fn sync(app: &AppHandle, tab: &TabMediaState) {
    let session = SESSION
        .lock()
        .ok()
        .and_then(|s| s.clone())
        .filter(|s| s.tab == tab.tab_id);
    if let Some(s) = session {
        if let Some(window) = app.get_webview_window(&label(&s.id)) {
            let payload = json!({"session":s.id,"sequence":s.acknowledged,"volume":tab.applied_volume,"disabled":tab.volume_status=="unavailable","muted":tab.muted});
            let _ = window.eval(format!("window.__velocePipState?.({payload});"));
        }
    }
}

/// Unrelated iframe churn may change the aggregate volume key. The selected
/// source frame, document, generation and video session must still be identical.
pub fn matches(id: &str, tab: &str, token: &str, frames: &[super::volume::FrameMedia]) -> bool {
    SESSION.lock().is_ok_and(|state| {
        state.as_ref().is_some_and(|s| {
            s.id == id
                && s.tab == tab
                && s.token == token
                && frames.iter().any(|f| {
                    f.context == s.context
                        && f.document == s.document
                        && f.pip_session.as_deref() == Some(id)
                })
        })
    })
}

pub fn acknowledge(app: &AppHandle, id: &str, sequence: u64) {
    let tab = {
        let Ok(mut state) = SESSION.lock() else {
            return;
        };
        let Some(s) = state
            .as_mut()
            .filter(|s| s.id == id && sequence <= s.sequence)
        else {
            return;
        };
        s.acknowledged = s.acknowledged.max(sequence);
        s.tab.clone()
    };
    sync(app, &app.state::<MediaService>().tab(&tab));
}

pub fn discard(app: &AppHandle, id: &str) {
    let s = {
        let Ok(mut state) = SESSION.lock() else {
            return;
        };
        if state.as_ref().is_none_or(|s| s.id != id) {
            return;
        }
        state.take()
    };
    if let Some(s) = s {
        if let Some(window) = app.get_webview_window(&label(&s.id)) {
            let _ = window.destroy();
        }
    }
}

pub fn source_removed(app: &AppHandle, tab: &str, context: Option<&str>) {
    let s = SESSION
        .lock()
        .ok()
        .and_then(|s| s.clone())
        .filter(|s| s.tab == tab && context.is_none_or(|c| c == s.context));
    if let Some(s) = s {
        discard(app, &s.id);
    }
}

pub fn owner() -> Option<(String, String, String, String, String)> {
    SESSION.lock().ok().and_then(|s| {
        s.as_ref().map(|s| {
            (
                s.id.clone(),
                s.tab.clone(),
                s.token.clone(),
                s.context.clone(),
                s.document.clone(),
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unrelated_frames_do_not_authorize_an_old_source_document() {
        let id = prepare(
            "tab",
            "generation",
            "source",
            "document",
            "aggregate-before",
        );
        let source = super::super::volume::FrameMedia {
            context: "source".into(),
            document: "document".into(),
            pip_session: Some(id.clone()),
            ..Default::default()
        };
        let other = super::super::volume::FrameMedia {
            context: "ad-frame".into(),
            document: "new-frame".into(),
            ..Default::default()
        };
        assert!(matches(&id, "tab", "generation", &[source.clone(), other]));
        assert!(!matches(
            &id,
            "other-tab",
            "generation",
            std::slice::from_ref(&source)
        ));
        assert!(!matches(
            &id,
            "tab",
            "old-generation",
            std::slice::from_ref(&source)
        ));
        assert!(!matches(
            "old-session",
            "tab",
            "generation",
            std::slice::from_ref(&source)
        ));
        assert!(!matches(
            &id,
            "tab",
            "generation",
            &[super::super::volume::FrameMedia {
                document: "new-document".into(),
                ..source.clone()
            }]
        ));
        assert!(!matches(
            &id,
            "tab",
            "generation",
            &[super::super::volume::FrameMedia {
                context: "new-context".into(),
                ..source
            }]
        ));
        *SESSION.lock().unwrap() = None;
    }
}
