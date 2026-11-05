mod bridge;

use super::webview_manager::create_internal_bridge_token;
use crate::state::browser_state::BrowserState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
#[cfg_attr(target_os = "linux", allow(unused_imports))]
use tauri::{webview::WebviewBuilder, AppHandle, LogicalPosition, Manager, Rect, WebviewUrl};

pub const LABEL: &str = "history-overlay-webview";

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    id: i64,
    title: String,
    url: String,
    time: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Labels {
    title: String,
    clear: String,
    close: String,
    search: String,
    empty: String,
    load_more: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Theme {
    surface: String,
    border: String,
    text: String,
    muted: String,
    accent: String,
    hover: String,
    field: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Presentation {
    tab_id: String,
    url: String,
    locale: String,
    labels: Labels,
    theme: Theme,
    query: String,
    entries: Vec<Entry>,
    busy: bool,
    error: String,
    has_more: bool,
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
pub struct HistoryOverlay(Mutex<Inner>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ActionPayload {
    action: String,
    #[serde(default)]
    query: String,
    #[serde(default)]
    entry_id: Option<i64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionEvent {
    session_id: u64,
    tab_id: String,
    url: String,
    action: String,
    query: String,
    entry_id: Option<i64>,
    entry_url: Option<String>,
}

fn valid_context(state: &BrowserState, session: &Session) -> bool {
    if state.is_overlay_open.lock().map_or(true, |value| *value)
        || state
            .workspace_switcher_open
            .lock()
            .map_or(true, |value| *value)
        || state
            .bookmark_context_menu_open
            .lock()
            .map_or(true, |value| *value)
    {
        return false;
    }
    let active = state
        .active_tab_id
        .lock()
        .is_ok_and(|id| *id == session.presentation.tab_id);
    let workspace = state
        .active_workspace_id
        .lock()
        .is_ok_and(|id| *id == session.workspace);
    let page = state.tabs.lock().is_ok_and(|tabs| {
        tabs.iter()
            .any(|tab| tab.id == session.presentation.tab_id && tab.url == session.presentation.url)
    });
    active && workspace && page
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
        .join("/history-overlay.html")
        .map_err(|e| e.to_string())?;
    let callback_app = app.clone();
    let view = main
        .add_child(
            WebviewBuilder::new(LABEL, WebviewUrl::App("history-overlay.html".into()))
                .transparent(true)
                .focused(false)
                .on_navigation(move |url| {
                    if url.scheme() == "veloce-internal" {
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
    app.state::<HistoryOverlay>()
        .0
        .lock()
        .map_or(true, |inner| inner.session.is_some())
}

pub fn dismiss(app: &AppHandle) {
    let service = app.state::<HistoryOverlay>();
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
    let service = app.state::<HistoryOverlay>();
    let inner = service.0.lock().map_err(|e| e.to_string())?;
    let Some(session) = inner.session.as_ref() else {
        return Ok(());
    };
    let valid = valid_context(state, session);
    drop(inner);
    if !valid {
        dismiss(app);
    } else if let Some(view) = app.get_webview(LABEL) {
        view.show().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn set(
    app: &AppHandle,
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
) -> Result<(), String> {
    let service = app.state::<HistoryOverlay>();
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
                "window.__veloceSetHistoryOverlay?.({{visible:false,sessionId:{session_id}}});"
            ));
            view.hide().map_err(|e| e.to_string())?;
        }
        super::content_focus::focus_active_content_webview(app)?;
        return Ok(());
    }
    if session_id <= inner.closed_session
        || inner.session.as_ref().is_some_and(|session| {
            session.id > session_id || (session.id == session_id && session.closing)
        })
    {
        return Ok(());
    }
    let presentation = presentation.ok_or("Missing history overlay presentation")?;
    if presentation.tab_id.is_empty()
        || presentation.url.is_empty()
        || presentation.entries.len() > 60
        || !matches!(presentation.locale.as_str(), "pt-BR" | "en")
    {
        return Err("Invalid history overlay presentation".into());
    }
    let state = app.state::<BrowserState>();
    let workspace = state
        .active_workspace_id
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
            .ok_or("Missing history overlay session")?;
        if previous.presentation.tab_id != session.presentation.tab_id
            || previous.presentation.url != session.presentation.url
        {
            return Err("History overlay context changed".into());
        }
        session.token.clone_from(&previous.token);
        session.sequence = previous.sequence;
    }
    if !valid_context(&state, &session) {
        return Err("History overlay context changed".into());
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
    ensure_webview(app)?;
    resize(app)?;
    let view = app
        .get_webview(LABEL)
        .ok_or("History overlay WebView not found")?;
    view.eval(format!("(() => {{ const payload = {payload}; let attempts = 0; const update = () => {{ if (window.__veloceSetHistoryOverlay) window.__veloceSetHistoryOverlay(payload); else if (++attempts < 120) setTimeout(update, 16); }}; update(); }})();")).map_err(|e| e.to_string())?;
    view.show().map_err(|e| e.to_string())?;
    if opening {
        view.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}
