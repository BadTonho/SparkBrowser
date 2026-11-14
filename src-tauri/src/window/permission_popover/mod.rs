mod bridge;

use super::webview_manager::create_internal_bridge_token;
use crate::state::browser_state::BrowserState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
#[cfg_attr(target_os = "linux", allow(unused_imports))]
use tauri::{AppHandle, LogicalPosition, Manager, Rect, WebviewUrl};

pub const LABEL: &str = "permission-popover-webview";

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestContext {
    id: String,
    tab_id: String,
    origin: String,
    top_origin: String,
    frame_origin: Option<String>,
    kind: String,
    display_origin: String,
    details: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    mode: String,
    tab_id: String,
    url: String,
    origin: String,
    anchor: Anchor,
    request: Option<RequestContext>,
    pending_count: usize,
    settings: serde_json::Value,
    labels: serde_json::Value,
    theme: serde_json::Value,
    locale: String,
    error: String,
    busy: bool,
    confirming_reload: bool,
}

struct Session {
    id: u64,
    token: String,
    workspace: String,
    presentation: Presentation,
    sequence: u64,
    closing: bool,
}
#[derive(Default)]
struct Inner {
    session: Option<Session>,
    closed_session: u64,
}
#[derive(Default)]
pub struct PermissionPopover(Mutex<Inner>);

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ActionPayload {
    action: String,
    #[serde(default)]
    origin: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    value: String,
    #[serde(default)]
    decision: String,
    #[serde(default)]
    reason: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionEvent {
    session_id: u64,
    tab_id: String,
    url: String,
    origin: String,
    request_id: Option<String>,
    action: String,
    kind: String,
    value: String,
    decision: String,
    reason: String,
}

fn valid_context(app: &AppHandle, state: &BrowserState, session: &Session) -> bool {
    if state
        .surfaces()
        .is_overlay_open()
        .lock()
        .map_or(true, |value| *value)
        || state
            .surfaces()
            .workspace_switcher_open()
            .lock()
            .map_or(true, |value| *value)
        || state
            .surfaces()
            .bookmark_context_menu_open()
            .lock()
            .map_or(true, |value| *value)
    {
        return false;
    }
    let active = state.is_active_tab(&session.presentation.tab_id);
    let workspace = state
        .active_workspace_of_tab(&session.presentation.tab_id)
        .is_some_and(|id| id == session.workspace);
    let page = state.session().tabs().lock().is_ok_and(|tabs| {
        tabs.iter()
            .any(|tab| tab.id == session.presentation.tab_id && tab.url == session.presentation.url)
    });
    let expected_origin =
        crate::site_permissions::origin(&session.presentation.url).unwrap_or_default();
    if !(active && workspace && page) || expected_origin != session.presentation.origin {
        return false;
    }
    if let Some(request) = &session.presentation.request {
        app.try_state::<crate::site_permissions::PermissionRequests>()
            .is_some_and(|requests| {
                requests.pending.lock().is_ok_and(|items| {
                    items.iter().any(|item| {
                        item.id == request.id
                            && item.tab_id == request.tab_id
                            && item.origin == request.origin
                            && item.top_origin == request.top_origin
                            && item.frame_origin == request.frame_origin
                            && format!("{:?}", item.kind).to_lowercase() == request.kind
                    })
                })
            })
    } else {
        true
    }
}

fn ensure_webview(app: &AppHandle, window_id: &str) -> Result<(), String> {
    if super::overlay_host::reuse(app, LABEL, window_id) {
        return Ok(());
    }
    let main = app.get_window(window_id).ok_or("Main window not found")?;
    let document = app
        .get_webview(window_id)
        .ok_or("Browser shell not found")?
        .url()
        .map_err(|e| e.to_string())?
        .join("/permission-popover.html")
        .map_err(|e| e.to_string())?;
    let callback_app = app.clone();
    let view = main
        .add_child(
            crate::window::create_webview_builder(
                LABEL,
                WebviewUrl::App("permission-popover.html".into()),
            )
            .transparent(true)
            .focused(false)
            .on_navigation(move |url| {
                if url.scheme() == "spark-internal" {
                    bridge::navigation(&callback_app, url);
                    return false;
                }
                crate::security::documents::same_document(url, &document)
            }),
            LogicalPosition::new(0.0, 0.0),
            main.inner_size()
                .map_err(|e| e.to_string())?
                .to_logical::<f64>(main.scale_factor().map_err(|e| e.to_string())?),
        )
        .map_err(|e| e.to_string())?;
    crate::security::harden(app, &view, true)?;
    view.hide().map_err(|e| e.to_string())
}

pub fn resize(app: &AppHandle) -> Result<(), String> {
    if let (Some(_view), Some(window)) = (
        app.get_webview(LABEL),
        super::overlay_host::window_of(app, LABEL),
    ) {
        let size = window
            .inner_size()
            .map_err(|e| e.to_string())?
            .to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);
        #[cfg(target_os = "linux")]
        super::linux_layout::update_bounds(
            app,
            LABEL,
            0,
            0,
            size.width as i32,
            size.height as i32,
            is_open(app),
        )?;
        #[cfg(not(target_os = "linux"))]
        _view
            .set_bounds(Rect {
                position: LogicalPosition::new(0.0, 0.0).into(),
                size: size.into(),
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn is_open(app: &AppHandle) -> bool {
    app.state::<PermissionPopover>()
        .0
        .lock()
        .map_or(true, |inner| inner.session.is_some())
}
pub fn dismiss(app: &AppHandle) {
    let service = app.state::<PermissionPopover>();
    let Ok(mut inner) = service.0.lock() else {
        return;
    };
    if let Some(session) = inner.session.take() {
        inner.closed_session = inner.closed_session.max(session.id);
        if let Some(view) = app.get_webview(LABEL) {
            let _ = view.hide();
        }
    }
}
pub fn sync(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let service = app.state::<PermissionPopover>();
    let inner = service.0.lock().map_err(|e| e.to_string())?;
    let Some(session) = inner.session.as_ref() else {
        return Ok(());
    };
    let valid = valid_context(app, state, session);
    drop(inner);
    if !valid {
        dismiss(app);
    } else if let Some(view) = app.get_webview(LABEL) {
        crate::window::show_overlay_webview(&view)?;
    }
    Ok(())
}
pub fn set(
    app: &AppHandle,
    window_id: &str,
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
) -> Result<(), String> {
    let service = app.state::<PermissionPopover>();
    let mut inner = service.0.lock().map_err(|e| e.to_string())?;
    if !visible {
        inner.closed_session = inner.closed_session.max(session_id);
        if inner
            .session
            .as_ref()
            .is_some_and(|session| session.id > session_id)
        {
            return Ok(());
        }
        inner.session = None;
        drop(inner);
        if let Some(view) = app.get_webview(LABEL) {
            let _ = view.eval(format!(
                "window.__sparkSetPermissionPopover?.({{visible:false,sessionId:{session_id}}});"
            ));
            view.hide().map_err(|e| e.to_string())?;
        }
        super::content_focus::focus_active_content_webview(
            app,
            &super::overlay_host::host_id(app, LABEL),
        )?;
        return Ok(());
    }
    if session_id <= inner.closed_session
        || inner.session.as_ref().is_some_and(|session| {
            session.id > session_id || (session.id == session_id && session.closing)
        })
    {
        return Ok(());
    }
    let presentation = presentation.ok_or("Missing permission popover presentation")?;
    let anchor = &presentation.anchor;
    if ![anchor.left, anchor.top, anchor.width, anchor.height]
        .iter()
        .all(|value| value.is_finite())
        || anchor.width <= 0.0
        || anchor.height <= 0.0
        || !matches!(presentation.mode.as_str(), "settings" | "request")
    {
        return Err("Invalid permission popover presentation".into());
    }
    let state = app.state::<BrowserState>();
    let workspace = state
        .session()
        .window(window_id)
        .ok_or("windows.not_found")?
        .active_workspace_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let opening = inner
        .session
        .as_ref()
        .is_none_or(|session| session.id != session_id);
    let mut session = Session {
        id: session_id,
        token: create_internal_bridge_token(),
        workspace,
        presentation,
        sequence: 0,
        closing: false,
    };
    if !opening {
        let previous = inner
            .session
            .as_ref()
            .ok_or("Missing permission popover session")?;
        if previous.presentation.tab_id != session.presentation.tab_id
            || previous.presentation.url != session.presentation.url
        {
            return Err("Permission popover context changed".into());
        }
        session.token.clone_from(&previous.token);
        session.sequence = previous.sequence;
    }
    if !valid_context(app, &state, &session) {
        return Err("Permission popover context changed".into());
    }
    let mut payload = serde_json::to_value(&session.presentation).map_err(|e| e.to_string())?;
    payload["visible"] = true.into();
    payload["sessionId"] = session_id.into();
    payload["token"] = session.token.clone().into();
    inner.session = Some(session);
    drop(inner);
    super::privacy_popover::dismiss(app);
    super::tools_popover::dismiss(app);
    super::downloads_popover::dismiss(app);
    super::quick_settings_popover::dismiss(app);
    super::zoom_popover::dismiss(app);
    super::media_popover::dismiss(app);
    let result = (|| {
        ensure_webview(app, window_id)?;
        resize(app)?;
        let view = app
            .get_webview(LABEL)
            .ok_or("Permission popover WebView not found")?;
        view.eval(format!("(() => {{ const payload = {payload}; let attempts = 0; const update = () => {{ if (window.__sparkSetPermissionPopover) window.__sparkSetPermissionPopover(payload); else if (++attempts < 120) setTimeout(update, 16); }}; update(); }})();")).map_err(|e| e.to_string())?;
        crate::window::show_overlay_webview(&view)?;
        if opening {
            view.set_focus().map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    if result.is_err() {
        dismiss(app);
    }
    result
}
