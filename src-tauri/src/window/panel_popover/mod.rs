mod bridge;
mod focus;

use super::webview_manager::create_internal_bridge_token;
use crate::state::browser_state::BrowserState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
#[cfg_attr(target_os = "linux", allow(unused_imports))]
use tauri::{AppHandle, LogicalPosition, Manager, Rect, WebviewUrl};

pub trait Panel: Send + Sync + 'static {
    const LABEL: &'static str;
    const SURFACE: &'static str;
    const HOST: &'static str;
    const SETTER: &'static str;
    const EVENT: &'static str;
}
#[derive(Default)]
pub struct Tools;
#[derive(Default)]
pub struct Downloads;
#[derive(Default)]
pub struct QuickSettings;
#[derive(Default)]
pub struct Settings;
impl Panel for Settings {
    const LABEL: &'static str = "settings-overlay-webview";
    const SURFACE: &'static str = "/index.html?surface=settings";
    const HOST: &'static str = "settings-popover";
    const SETTER: &'static str = "__veloceSetSettingsPopover";
    const EVENT: &'static str = "settings-popover-action";
}
impl Panel for Tools {
    const LABEL: &'static str = "tools-popover-webview";
    const SURFACE: &'static str = "/index.html?surface=tools";
    const HOST: &'static str = "tools-popover";
    const SETTER: &'static str = "__veloceSetToolsPopover";
    const EVENT: &'static str = "tools-popover-action";
}
impl Panel for Downloads {
    const LABEL: &'static str = "downloads-popover-webview";
    const SURFACE: &'static str = "/index.html?surface=downloads";
    const HOST: &'static str = "downloads-popover";
    const SETTER: &'static str = "__veloceSetDownloadsPopover";
    const EVENT: &'static str = "downloads-popover-action";
}
impl Panel for QuickSettings {
    const LABEL: &'static str = "quick-settings-popover-webview";
    const SURFACE: &'static str = "/index.html?surface=settings-quick";
    const HOST: &'static str = "settings-quick-popover";
    const SETTER: &'static str = "__veloceSetQuickSettingsPopover";
    const EVENT: &'static str = "quick-settings-popover-action";
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Anchor {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    anchor: Anchor,
    tree: serde_json::Value,
    theme: std::collections::HashMap<String, String>,
    locale: String,
    ack: u64,
    revision: u64,
}

struct Session {
    id: u64,
    token: String,
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
pub struct Popover<P: Panel>(Mutex<Inner>, std::marker::PhantomData<P>);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Action {
    session_id: u64,
    sequence: u64,
    action: bridge::UiAction,
}

fn blocked(state: &BrowserState) -> bool {
    state.is_overlay_open.lock().map_or(true, |v| *v)
        || state.workspace_switcher_open.lock().map_or(true, |v| *v)
        || state.bookmark_context_menu_open.lock().map_or(true, |v| *v)
}

fn current(_app: &AppHandle, state: &BrowserState, _session: &Session) -> bool {
    !blocked(state)
}

fn action(session: &Session, value: bridge::UiAction) -> Action {
    Action {
        session_id: session.id,
        sequence: session.sequence + 1,
        action: value,
    }
}

fn ensure_webview<P: Panel>(app: &AppHandle) -> Result<(), String> {
    if app.get_webview(P::LABEL).is_some() {
        return Ok(());
    }
    let main = app.get_window("main").ok_or("Main window not found")?;
    let document = app
        .get_webview("main")
        .ok_or("Browser shell not found")?
        .url()
        .map_err(|e| e.to_string())?
        .join(P::SURFACE)
        .map_err(|e| e.to_string())?;
    let callback_app = app.clone();
    let view = main
        .add_child(
            crate::window::create_webview_builder(P::LABEL, WebviewUrl::External(document.clone()))
                .transparent(true)
                .focused(false)
                .on_navigation(move |url| {
                    if url.scheme() == "veloce-internal" {
                        bridge::navigation::<P>(&callback_app, url);
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

pub fn resize<P: Panel>(app: &AppHandle) -> Result<(), String> {
    if let (Some(_view), Some(window)) = (app.get_webview(P::LABEL), app.get_window("main")) {
        let size = window
            .inner_size()
            .map_err(|e| e.to_string())?
            .to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);
        #[cfg(target_os = "linux")]
        super::linux_layout::update_bounds(
            app,
            P::LABEL,
            0,
            0,
            size.width as i32,
            size.height as i32,
            is_open::<P>(app),
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

pub fn is_open<P: Panel>(app: &AppHandle) -> bool {
    app.state::<Popover<P>>()
        .0
        .lock()
        .map_or(true, |inner| inner.session.is_some())
}

pub fn focus_lost<P: Panel>(app: &AppHandle) {
    focus::lost::<P>(app);
}

pub fn dismiss<P: Panel>(app: &AppHandle) {
    let service = app.state::<Popover<P>>();
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
        P::EVENT,
        action(
            &session,
            bridge::UiAction::Dismiss {
                reason: bridge::DismissReason::Blur,
            },
        ),
    );
    if let Some(view) = app.get_webview(P::LABEL) {
        let _ = view.hide();
    }
}

/// Runs after content reconciliation so the transparent popover stays above
/// both panes without changing their visibility or document lifetime.
pub fn sync<P: Panel>(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let service = app.state::<Popover<P>>();
    let inner = service.0.lock().map_err(|e| e.to_string())?;
    let Some(session) = inner.session.as_ref() else {
        return Ok(());
    };
    let valid = current(app, state, session);
    drop(inner);
    if !valid {
        dismiss::<P>(app);
    } else if let Some(view) = app.get_webview(P::LABEL) {
        view.show().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn set<P: Panel>(
    app: &AppHandle,
    visible: bool,
    session_id: u64,
    presentation: Option<Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let service = app.state::<Popover<P>>();
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
        if let Some(view) = app.get_webview(P::LABEL) {
            let _ = view.eval(format!(
                "window.{}?.({{visible:false,sessionId:{session_id}}});",
                P::SETTER
            ));
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
    let presentation = presentation.ok_or("Missing tools popover presentation")?;
    let anchor = &presentation.anchor;
    if ![anchor.left, anchor.top, anchor.width, anchor.height]
        .iter()
        .all(|value| value.is_finite())
        || anchor.width <= 0.0
        || anchor.height <= 0.0
    {
        return Err("Invalid tools popover presentation".into());
    }
    if serde_json::to_vec(&presentation)
        .map_err(|e| e.to_string())?
        .len()
        > 2_000_000
    {
        return Err("Panel presentation too large".into());
    }
    let state = app.state::<BrowserState>();
    let opening = inner
        .session
        .as_ref()
        .is_none_or(|session| session.id != session_id);
    let mut session = Session {
        id: session_id,
        token: create_internal_bridge_token(),
        presentation,
        sequence: 0,
        closing: false,
    };
    if !opening {
        let previous = inner
            .session
            .as_ref()
            .ok_or("Missing tools popover session")?;
        if previous.presentation.revision > session.presentation.revision {
            return Ok(());
        }
        session.token.clone_from(&previous.token);
        session.sequence = previous.sequence;
    }
    if !current(app, &state, &session) {
        return Err("Panel popover context changed".into());
    }
    let mut payload = serde_json::to_value(&session.presentation).map_err(|e| e.to_string())?;
    payload["visible"] = true.into();
    payload["sessionId"] = session_id.into();
    payload["token"] = session.token.clone().into();
    inner.session = Some(session);
    drop(inner);
    if P::HOST == "tools-popover" {
        super::downloads_popover::dismiss(app);
        super::quick_settings_popover::dismiss(app);
    } else if P::HOST == "downloads-popover" {
        super::tools_popover::dismiss(app);
        super::quick_settings_popover::dismiss(app);
    } else if P::HOST == "settings-quick-popover" {
        super::tools_popover::dismiss(app);
        super::downloads_popover::dismiss(app);
        super::history_overlay::dismiss(app);
        super::permission_popover::dismiss(app);
    } else {
        super::tools_popover::dismiss(app);
    }
    super::zoom_popover::dismiss(app);
    super::media_popover::dismiss(app);
    super::privacy_popover::dismiss(app);
    let result = (|| {
        ensure_webview::<P>(app)?;
        resize::<P>(app)?;
        let view = app.get_webview(P::LABEL).ok_or("Panel WebView not found")?;
        view.eval(format!("(() => {{ const payload = {payload}; let attempts = 0; const update = () => {{ if (window.{setter}) window.{setter}(payload); else if (++attempts < 120) setTimeout(update, 16); }}; update(); }})();", setter = P::SETTER)).map_err(|e| e.to_string())?;
        view.show().map_err(|e| e.to_string())?;
        if opening {
            view.set_focus().map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    if result.is_err() {
        dismiss::<P>(app);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{Downloads, Panel, QuickSettings, Tools};

    #[test]
    fn quick_settings_action_host_matches_its_surface_route() {
        assert_eq!(Tools::HOST, "tools-popover");
        assert_eq!(Downloads::HOST, "downloads-popover");
        assert_eq!(QuickSettings::HOST, "settings-quick-popover");
    }
}
