mod bridge;

use super::webview_manager::create_internal_bridge_token;
use crate::state::browser_state::BrowserState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
#[cfg_attr(target_os = "linux", allow(unused_imports))]
use tauri::{AppHandle, LogicalPosition, Manager, Rect, WebviewUrl};

pub const LABEL: &str = "privacy-popover-webview";

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
pub struct Presentation {
    tab_id: String,
    url: String,
    anchor: Anchor,
    shield: bool,
    auto_skip: bool,
    busy: bool,
    error: String,
    blocked: String,
    adblock_label: String,
    auto_skip_label: String,
    description: String,
    label: String,
    locale: String,
    theme: Theme,
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
pub struct PrivacyPopover(Mutex<Inner>);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Action {
    session_id: u64,
    tab_id: String,
    url: String,
    action: String,
    value: Option<bool>,
    reason: String,
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

fn current(_app: &AppHandle, state: &BrowserState, session: &Session) -> bool {
    !blocked(state)
        && state
            .session()
            .active_tab_id()
            .lock()
            .is_ok_and(|id| *id == session.presentation.tab_id)
        && state
            .session()
            .active_workspace_id()
            .lock()
            .is_ok_and(|id| *id == session.workspace)
        && state.session().tabs().lock().is_ok_and(|tabs| {
            tabs.iter().any(|tab| {
                tab.id == session.presentation.tab_id && tab.url == session.presentation.url
            })
        })
}

fn action(session: &Session, name: &str, value: Option<bool>, reason: &str) -> Action {
    Action {
        session_id: session.id,
        tab_id: session.presentation.tab_id.clone(),
        url: session.presentation.url.clone(),
        action: name.into(),
        value,
        reason: reason.into(),
    }
}

fn ensure_webview(app: &AppHandle) -> Result<(), String> {
    if app.get_webview(LABEL).is_some() {
        return Ok(());
    }
    let main = app.get_window("main").ok_or("Main window not found")?;
    let document = app
        .get_webview("main")
        .ok_or("Browser shell not found")?
        .url()
        .map_err(|e| e.to_string())?
        .join("/privacy-popover.html")
        .map_err(|e| e.to_string())?;
    let callback_app = app.clone();
    let view = main
        .add_child(
            crate::window::create_webview_builder(
                LABEL,
                WebviewUrl::App("privacy-popover.html".into()),
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
    if let (Some(_view), Some(window)) = (app.get_webview(LABEL), app.get_window("main")) {
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
    app.state::<PrivacyPopover>()
        .0
        .lock()
        .map_or(true, |inner| inner.session.is_some())
}

pub fn dismiss(app: &AppHandle) {
    let service = app.state::<PrivacyPopover>();
    let Ok(mut inner) = service.0.lock() else {
        return;
    };
    let Some(session) = inner.session.take() else {
        return;
    };
    inner.closed_session = inner.closed_session.max(session.id);
    drop(inner);
    let _ = crate::security::emit_shell(
        app,
        "privacy-popover-action",
        action(&session, "dismiss", None, "context"),
    );
    if let Some(view) = app.get_webview(LABEL) {
        let _ = view.hide();
    }
}

/// Runs after content reconciliation so the transparent popover stays above
/// both panes without changing their visibility or document lifetime.
pub fn sync(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let service = app.state::<PrivacyPopover>();
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
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let service = app.state::<PrivacyPopover>();
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
            let _ = view.eval("window.__sparkSetPrivacyPopover?.({visible:false});");
            view.hide().map_err(|e| e.to_string())?;
        }
        if restore_focus {
            if let Some(main) = app.get_webview("main") {
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
    let presentation = presentation.ok_or("Missing privacy popover presentation")?;
    let anchor = &presentation.anchor;
    if ![anchor.left, anchor.top, anchor.width, anchor.height]
        .iter()
        .all(|value| value.is_finite())
        || anchor.width <= 0.0
        || anchor.height <= 0.0
    {
        return Err("Invalid privacy popover presentation".into());
    }
    let state = app.state::<BrowserState>();
    let workspace = state
        .session()
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
            .ok_or("Missing privacy popover session")?;
        if previous.presentation.tab_id != session.presentation.tab_id
            || previous.presentation.url != session.presentation.url
        {
            return Err("Privacy popover context changed".into());
        }
        session.token.clone_from(&previous.token);
        session.sequence = previous.sequence;
    }
    if !current(app, &state, &session) {
        return Err("Privacy popover context changed".into());
    }
    let mut payload = serde_json::to_value(&session.presentation).map_err(|e| e.to_string())?;
    payload["visible"] = true.into();
    payload["token"] = session.token.clone().into();
    inner.session = Some(session);
    drop(inner);
    super::tools_popover::dismiss(app);
    super::downloads_popover::dismiss(app);
    super::quick_settings_popover::dismiss(app);
    super::zoom_popover::dismiss(app);
    super::media_popover::dismiss(app);
    let result = (|| {
        ensure_webview(app)?;
        resize(app)?;
        let view = app
            .get_webview(LABEL)
            .ok_or("Privacy popover WebView not found")?;
        view.eval(format!("(() => {{ const payload = {payload}; let attempts = 0; const update = () => {{ if (window.__sparkSetPrivacyPopover) window.__sparkSetPrivacyPopover(payload); else if (++attempts < 120) setTimeout(update, 16); }}; update(); }})();")).map_err(|e| e.to_string())?;
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
