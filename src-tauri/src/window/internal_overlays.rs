use super::webview_manager::{
    BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL, WORKSPACE_SWITCHER_WEBVIEW_LABEL,
};
use crate::state::browser_state::{BrowserState, ViewportBounds};
use crate::state::workspace_model::WorkspaceItem;
use serde::{Deserialize, Serialize};
use tauri::{
    webview::WebviewBuilder, AppHandle, LogicalPosition, LogicalSize, Manager, Url, WebviewUrl,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayLanguage {
    pub locale: String,
    pub label: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayPresentation {
    pub backdrop_blur_px: Option<u8>,
    pub language: Option<OverlayLanguage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkContextMenuAction {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub danger: bool,
}

pub fn create_internal_bridge_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS cryptographic randomness unavailable");
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn build_workspace_switcher_update_script(
    workspaces: &[WorkspaceItem],
    active_workspace_id: &str,
    token: &str,
    visible: bool,
    backdrop_blur_px: u8,
    language: Option<&OverlayLanguage>,
) -> Result<String, String> {
    let payload = serde_json::to_string(&serde_json::json!({
        "workspaces": workspaces,
        "activeWorkspaceId": active_workspace_id,
        "token": token,
        "visible": visible,
        "backdropBlurPx": backdrop_blur_px.min(24),
        "language": language,
    }))
    .map_err(|error| format!("Failed to serialize workspace switcher: {error}"))?;
    Ok(format!(
        "(() => {{ const payload = {payload}; let attempts = 0; const update = () => {{ if (typeof window.__veloceSetWorkspaceSwitcher === 'function') {{ window.__veloceSetWorkspaceSwitcher(payload); return; }} if (++attempts < 120) setTimeout(update, 16); }}; update(); }})();"
    ))
}

pub fn build_bookmark_context_menu_script(
    x: f64,
    y: f64,
    actions: &[BookmarkContextMenuAction],
    token: &str,
    visible: bool,
    backdrop_blur_px: u8,
    language: Option<&OverlayLanguage>,
) -> Result<String, String> {
    let payload = serde_json::to_string(&serde_json::json!({
        "x": x,
        "y": y,
        "actions": actions,
        "token": token,
        "visible": visible,
        "backdropBlurPx": backdrop_blur_px.min(24),
        "language": language,
    }))
    .map_err(|error| format!("Failed to serialize bookmark context menu: {error}"))?;
    Ok(format!(
        "(() => {{ const payload = {payload}; let attempts = 0; const update = () => {{ if (typeof window.__veloceSetBookmarkContextMenu === 'function') {{ window.__veloceSetBookmarkContextMenu(payload); return; }} if (++attempts < 120) setTimeout(update, 16); }}; update(); }})();"
    ))
}

pub(super) fn handle_workspace_switcher_navigation(app: &AppHandle, nav_url: &Url) {
    if nav_url.scheme() != "veloce-internal" || nav_url.host_str() != Some("workspace-switcher") {
        return;
    }

    let Some(parameters) = crate::security::bridge::query(nav_url, &["token", "workspace_id"])
    else {
        return;
    };
    let action = nav_url.path().trim_matches('/');
    let request_token = parameters.get("token").cloned();
    let window_is_focused = app
        .get_webview_window("main")
        .and_then(|window| window.is_focused().ok())
        .unwrap_or(false);
    if !window_is_focused {
        return;
    }

    let Some(state) = app.try_state::<BrowserState>() else {
        return;
    };
    let token_matches = request_token
        .as_deref()
        .and_then(|request_token| {
            state
                .workspace_switcher_token
                .lock()
                .ok()
                .map(|token| token.as_deref() == Some(request_token))
        })
        .unwrap_or(false);

    match action {
        "select" if token_matches => {
            let workspace_id = parameters.get("workspace_id").cloned();
            let Some(workspace_id) = workspace_id else {
                return;
            };
            let is_valid_workspace = state
                .workspaces
                .lock()
                .map(|workspaces| workspaces.iter().any(|item| item.id == workspace_id))
                .unwrap_or(false);
            if !is_valid_workspace {
                return;
            }
            if let Ok(mut token) = state.workspace_switcher_token.lock() {
                *token = None;
            }
            let _ = crate::security::emit_shell(app, "workspace-switcher-selected", workspace_id);
        }
        "dismiss" if token_matches && !parameters.contains_key("workspace_id") => {
            let is_open = state
                .workspace_switcher_open
                .lock()
                .map(|open| *open)
                .unwrap_or(false);
            if is_open {
                if let Ok(mut token) = state.workspace_switcher_token.lock() {
                    *token = None;
                }
                let _ = crate::security::emit_shell(app, "workspace-switcher-dismissed", ());
            }
        }
        _ => {}
    }
}

fn handle_bookmark_context_menu_navigation(app: &AppHandle, nav_url: &Url) {
    if crate::security::bridge::query(nav_url, &["token"]).is_none() {
        return;
    }
    if nav_url.scheme() != "veloce-internal" || nav_url.host_str() != Some("bookmark-context-menu")
    {
        return;
    }

    let action = nav_url.path().trim_matches('/').to_string();
    let request_token = nav_url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.into_owned());
    let Some(state) = app.try_state::<BrowserState>() else {
        return;
    };
    let menu_is_open = state
        .bookmark_context_menu_open
        .lock()
        .map(|open| *open)
        .unwrap_or(false);
    if !menu_is_open {
        return;
    }
    let token_matches = request_token
        .as_deref()
        .and_then(|request_token| {
            state
                .bookmark_context_menu_token
                .lock()
                .ok()
                .map(|token| token.as_deref() == Some(request_token))
        })
        .unwrap_or(false);
    if !token_matches {
        return;
    }

    if action != "dismiss"
        && !state
            .bookmark_context_menu_actions
            .lock()
            .map(|actions| actions.iter().any(|allowed| allowed == &action))
            .unwrap_or(false)
    {
        return;
    }

    if let Ok(mut token) = state.bookmark_context_menu_token.lock() {
        *token = None;
    }
    if let Ok(mut actions) = state.bookmark_context_menu_actions.lock() {
        actions.clear();
    }
    // Let the frontend close the overlay after it receives the action. Hiding
    // this WebView inside its own navigation callback can interrupt delivery.
    let _ = crate::security::emit_shell(app, "bookmark-context-menu-action", action);
}

pub(super) fn spawn_workspace_switcher_webview(
    app: &AppHandle,
    bounds: ViewportBounds,
) -> Result<(), String> {
    if app.get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL).is_some() {
        return Ok(());
    }

    let main_window = app
        .get_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;
    let app_for_navigation = app.clone();
    let document = crate::security::documents::asset_base(app)
        .join("/workspace-switcher.html")
        .map_err(|_| "security.invalid_document")?;
    let webview_builder = WebviewBuilder::new(
        WORKSPACE_SWITCHER_WEBVIEW_LABEL,
        WebviewUrl::App("workspace-switcher.html".into()),
    )
    .transparent(true)
    .focused(false)
    .on_navigation(move |nav_url| {
        if nav_url.scheme() == "veloce-internal" {
            handle_workspace_switcher_navigation(&app_for_navigation, nav_url);
            return false;
        }
        crate::security::documents::same_document(nav_url, &document)
    });

    main_window
        .add_child(
            webview_builder,
            LogicalPosition::new(bounds.x, bounds.y),
            LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0)),
        )
        .map_err(|error| format!("Failed to create workspace switcher WebView: {error}"))?;

    #[cfg(target_os = "linux")]
    super::linux_layout::update_bounds(
        app,
        WORKSPACE_SWITCHER_WEBVIEW_LABEL,
        bounds.x as i32,
        bounds.y as i32,
        bounds.width.max(100.0) as i32,
        bounds.height.max(100.0) as i32,
        false,
    )?;

    if let Some(webview) = app.get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL) {
        crate::security::harden(app, &webview, true)?;
        let _ = webview.hide();
    }

    Ok(())
}

pub(super) fn spawn_bookmark_context_menu_webview(app: &AppHandle) -> Result<(), String> {
    if app
        .get_webview(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL)
        .is_some()
    {
        return Ok(());
    }

    let main_window = app
        .get_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;
    let app_for_navigation = app.clone();
    let document = crate::security::documents::asset_base(app)
        .join("/bookmark-context-menu.html")
        .map_err(|_| "security.invalid_document")?;
    let webview_builder = WebviewBuilder::new(
        BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL,
        WebviewUrl::App("bookmark-context-menu.html".into()),
    )
    .transparent(true)
    .focused(false)
    .on_navigation(move |nav_url| {
        if nav_url.scheme() == "veloce-internal"
            && nav_url.host_str() == Some("bookmark-context-menu")
        {
            handle_bookmark_context_menu_navigation(&app_for_navigation, nav_url);
            return false;
        }
        crate::security::documents::same_document(nav_url, &document)
    });

    main_window
        .add_child(
            webview_builder,
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(1200.0, 800.0),
        )
        .map_err(|error| format!("Failed to create bookmark context menu WebView: {error}"))?;

    #[cfg(target_os = "linux")]
    super::linux_layout::update_bounds(
        app,
        BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL,
        0,
        0,
        1200,
        800,
        false,
    )?;

    if let Some(webview) = app.get_webview(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL) {
        crate::security::harden(app, &webview, true)?;
        let _ = webview.hide();
    }
    Ok(())
}
