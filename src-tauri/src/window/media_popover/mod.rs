mod bridge;

use super::webview_manager::create_internal_bridge_token;
use crate::{media::MediaService, state::browser_state::BrowserState};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
#[cfg_attr(target_os = "linux", allow(unused_imports))]
use tauri::{AppHandle, LogicalPosition, Manager, Rect, WebviewUrl};

pub const LABEL: &str = "media-popover-webview";

#[derive(Clone, Deserialize, Serialize)]
pub struct Anchor {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Theme {
    background: String,
    border: String,
    accent: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    tab_id: String,
    tab_title: String,
    workspace_name: String,
    frame_label: String,
    playing: bool,
    audible: bool,
    muted: bool,
    mute_available: bool,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaLabels {
    title: String,
    playing: String,
    audible: String,
    mute: String,
    unmute: String,
    no_media: String,
    frame_limit: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    tab_id: String,
    document_key: String,
    anchor: Anchor,
    volume: u16,
    disabled: bool,
    label: String,
    locale: String,
    theme: Theme,
    items: Vec<MediaItem>,
    frame_tracking_supported: bool,
    labels: MediaLabels,
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
pub struct MediaPopover(Mutex<Inner>);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Action {
    session_id: u64,
    tab_id: String,
    document_key: String,
    action: String,
    value: Option<u16>,
    reason: String,
    target_tab_id: Option<String>,
}

fn blocked(state: &BrowserState) -> bool {
    state
        .surfaces()
        .is_overlay_open()
        .lock()
        .map_or(true, |v| *v)
        || state
            .surfaces()
            .workspace_switcher_open()
            .lock()
            .map_or(true, |v| *v)
        || state
            .surfaces()
            .bookmark_context_menu_open()
            .lock()
            .map_or(true, |v| *v)
}

fn current(app: &AppHandle, state: &BrowserState, session: &Session) -> bool {
    !blocked(state)
        && state.is_active_tab(&session.presentation.tab_id)
        && state
            .active_workspace_of_tab(&session.presentation.tab_id)
            .is_some_and(|id| id == session.workspace)
        && app
            .state::<MediaService>()
            .tab(&session.presentation.tab_id)
            .document_key
            == session.presentation.document_key
}

fn action(session: &Session, name: &str, value: Option<u16>, reason: &str) -> Action {
    Action {
        session_id: session.id,
        tab_id: session.presentation.tab_id.clone(),
        document_key: session.presentation.document_key.clone(),
        action: name.into(),
        value,
        reason: reason.into(),
        target_tab_id: None,
    }
}

/// Creates the web view, hidden, ahead of the click. See `panel_popover::prepare`.
pub fn prepare(app: &AppHandle, window_id: &str) -> Result<(), String> {
    ensure_webview(app, window_id)
}

fn ensure_webview(app: &AppHandle, window_id: &str) -> Result<(), String> {
    let _creating = super::panel_popover::creation_guard()?;
    if super::overlay_host::reuse(app, LABEL, window_id) {
        return Ok(());
    }
    let main = app.get_window(window_id).ok_or("Main window not found")?;
    let document = app
        .get_webview(window_id)
        .ok_or("Browser shell not found")?
        .url()
        .map_err(|e| e.to_string())?
        .join("/media-popover.html")
        .map_err(|e| e.to_string())?;
    let callback_app = app.clone();
    let view = main
        .add_child(
            crate::window::create_webview_builder(
                LABEL,
                WebviewUrl::App("media-popover.html".into()),
            )
            .transparent(true)
            .focused(false)
            .on_navigation(move |url| {
                if url.scheme() == "spark-internal" {
                    bridge::navigation(&callback_app, url);
                    return false;
                }
                // Keep the auxiliary WebView on its packaged document.
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
    app.state::<MediaPopover>()
        .0
        .lock()
        .map_or(true, |inner| inner.session.is_some())
}

pub fn dismiss(app: &AppHandle) {
    let service = app.state::<MediaPopover>();
    let Ok(mut inner) = service.0.lock() else {
        return;
    };
    let Some(session) = inner.session.take() else {
        return;
    };
    inner.closed_session = inner.closed_session.max(session.id);
    drop(inner);
    let _ = super::overlay_host::emit_to_host(
        app,
        LABEL,
        "media-popover-action",
        action(&session, "dismiss", None, "context"),
    );
    if let Some(view) = app.get_webview(LABEL) {
        let _ = view.hide();
    }
}

/// Runs after content reconciliation so the transparent popover stays above
/// both panes without changing their visibility or document lifetime.
pub fn sync(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let service = app.state::<MediaPopover>();
    let inner = service.0.lock().map_err(|e| e.to_string())?;
    let Some(session) = inner.session.as_ref() else {
        return Ok(());
    };
    let valid = current(app, state, session);
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
    restore_focus: bool,
) -> Result<(), String> {
    let service = app.state::<MediaPopover>();
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
            let _ = view.eval("window.__sparkSetMediaPopover?.({visible:false});");
            view.hide().map_err(|e| e.to_string())?;
        }
        if restore_focus {
            if let Some(main) = app.get_webview(&super::overlay_host::host_id(app, LABEL)) {
                main.set_focus().map_err(|e| e.to_string())?;
            }
        }
        return Ok(());
    }
    if session_id <= inner.closed_session
        || inner.session.as_ref().is_some_and(|session| {
            session.id > session_id || (session.id == session_id && session.closing)
        })
    {
        return Ok(());
    }
    let presentation = presentation.ok_or("Missing media popover presentation")?;
    let anchor = &presentation.anchor;
    if ![anchor.left, anchor.top, anchor.width, anchor.height]
        .iter()
        .all(|value| value.is_finite())
        || anchor.width <= 0.0
        || anchor.height <= 0.0
        || presentation.volume > 200
        || presentation.items.len() > 128
        || presentation.items.iter().any(|item| {
            item.tab_id.len() > 128
                || item.tab_title.len() > 500
                || item.workspace_name.len() > 120
                || item.frame_label.len() > 120
        })
    {
        return Err("Invalid media popover presentation".into());
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
            .ok_or("Missing media popover session")?;
        if previous.presentation.tab_id != session.presentation.tab_id
            || previous.presentation.document_key != session.presentation.document_key
        {
            return Err("media.stale".into());
        }
        session.token.clone_from(&previous.token);
        session.sequence = previous.sequence;
    }
    if !current(app, &state, &session) {
        return Err("media.stale".into());
    }
    let mut payload = serde_json::to_value(&session.presentation).map_err(|e| e.to_string())?;
    payload["visible"] = true.into();
    payload["token"] = session.token.clone().into();
    inner.session = Some(session);
    drop(inner);
    super::zoom_popover::dismiss(app);
    super::tools_popover::dismiss(app);
    super::downloads_popover::dismiss(app);
    super::quick_settings_popover::dismiss(app);
    super::privacy_popover::dismiss(app);
    let result = (|| {
        ensure_webview(app, window_id)?;
        resize(app)?;
        let view = app
            .get_webview(LABEL)
            .ok_or("Media popover WebView not found")?;
        view.eval(format!("(() => {{ const payload = {payload}; let attempts = 0; const update = () => {{ if (window.__sparkSetMediaPopover) window.__sparkSetMediaPopover(payload); else if (++attempts < 120) setTimeout(update, 16); }}; update(); }})();")).map_err(|e| e.to_string())?;
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
