use crate::state::browser_state::BrowserState;
use crate::state::workspace_model::WorkspaceItem;
use crate::window::webview_manager::{
    active_content_webview, build_bookmark_context_menu_script,
    build_workspace_switcher_update_script, create_internal_bridge_token, sync_all_webviews,
    BookmarkContextMenuAction, OverlayLanguage, OverlayPresentation,
    BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL, WORKSPACE_SWITCHER_WEBVIEW_LABEL,
};
use std::time::Duration;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Rect, State};

const WORKSPACE_SWITCHER_CLOSE_ANIMATION: Duration = Duration::from_millis(140);

#[tauri::command]
pub async fn set_privacy_popover(
    app: AppHandle,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::privacy_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.tab_operations.lock().await;
    crate::window::privacy_popover::set(&app, visible, session_id, presentation, restore_focus)
}

#[tauri::command]
pub async fn set_media_popover(
    app: AppHandle,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::media_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.tab_operations.lock().await;
    crate::window::media_popover::set(&app, visible, session_id, presentation, restore_focus)
}

#[tauri::command]
pub async fn set_overlay_visibility(
    app: AppHandle,
    state: State<'_, BrowserState>,
    visible: bool,
) -> Result<(), String> {
    let _operation = state.tab_operations.lock().await;
    if let Ok(mut overlay) = state.is_overlay_open.lock() {
        *overlay = visible;
    }
    sync_all_webviews(&app, &state)?;
    Ok(())
}

#[tauri::command]
pub async fn set_workspace_switcher_overlay(
    app: AppHandle,
    state: State<'_, BrowserState>,
    visible: bool,
    workspaces: Vec<WorkspaceItem>,
    active_workspace_id: String,
    backdrop_blur_px: u8,
    language: Option<OverlayLanguage>,
) -> Result<(), String> {
    let _operation = state.tab_operations.lock().await;
    let webview = app
        .get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL)
        .ok_or_else(|| "Workspace switcher WebView was not found".to_string())?;

    // A fresh token also identifies this close animation, so a stale delayed
    // hide cannot hide the WebView after the picker has been reopened.
    let token = create_internal_bridge_token();
    let script = build_workspace_switcher_update_script(
        &workspaces,
        &active_workspace_id,
        &token,
        visible,
        backdrop_blur_px.min(24),
        language.as_ref(),
    )?;

    if let Ok(mut current_token) = state.workspace_switcher_token.lock() {
        *current_token = Some(token.clone());
    }
    if let Ok(mut open) = state.workspace_switcher_open.lock() {
        // Keep the native WebView shown during its exit animation.
        *open = true;
    }

    if let Some(content_webview) = active_content_webview(&app) {
        let _ = content_webview.eval(format!("window.__veloceWorkspaceSwitcherOpen = {visible};"));
    }

    if let Err(error) = webview.eval(&script) {
        if let Ok(mut current_token) = state.workspace_switcher_token.lock() {
            *current_token = None;
        }
        if let Ok(mut open) = state.workspace_switcher_open.lock() {
            *open = false;
        }
        if let Some(content_webview) = active_content_webview(&app) {
            let _ = content_webview.eval("window.__veloceWorkspaceSwitcherOpen = false;");
        }
        let _ = sync_all_webviews(&app, &state);
        return Err(format!("Failed to show workspace switcher: {error}"));
    }

    if !visible {
        let app_for_hide = app.clone();
        let closing_token = token;
        std::thread::spawn(move || {
            std::thread::sleep(WORKSPACE_SWITCHER_CLOSE_ANIMATION);
            let Some(state) = app_for_hide.try_state::<BrowserState>() else {
                return;
            };

            let _operation = state.tab_operations.blocking_lock();
            // Serialize the token check and state update with a possible reopen.
            let Ok(mut current_token) = state.workspace_switcher_token.lock() else {
                return;
            };
            if current_token.as_deref() != Some(closing_token.as_str()) {
                return;
            }
            let Ok(mut open) = state.workspace_switcher_open.lock() else {
                return;
            };
            *current_token = None;
            *open = false;
            drop(open);
            drop(current_token);

            let _ = sync_all_webviews(&app_for_hide, &state);
            let another_overlay_open = state
                .is_overlay_open
                .lock()
                .map(|overlay| *overlay)
                .unwrap_or(true);
            let app_is_focused = app_for_hide
                .get_webview_window("main")
                .and_then(|window| window.is_focused().ok())
                .unwrap_or(false);
            if app_is_focused {
                if another_overlay_open {
                    if let Some(main_window) = app_for_hide.get_webview_window("main") {
                        let _ = main_window.set_focus();
                    }
                } else if let Some(content_webview) = active_content_webview(&app_for_hide) {
                    let _ = content_webview.set_focus();
                }
            }
        });
        return Ok(());
    }

    sync_all_webviews(&app, &state)?;
    Ok(())
}

#[tauri::command]
pub async fn set_bookmark_context_menu_overlay(
    app: AppHandle,
    state: State<'_, BrowserState>,
    visible: bool,
    x: Option<f64>,
    y: Option<f64>,
    actions: Option<Vec<BookmarkContextMenuAction>>,
    presentation: Option<OverlayPresentation>,
) -> Result<(), String> {
    let _operation = state.tab_operations.lock().await;
    let webview = app
        .get_webview(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL)
        .ok_or_else(|| "Bookmark context menu WebView was not found".to_string())?;

    if !visible {
        if let Ok(mut token) = state.bookmark_context_menu_token.lock() {
            *token = None;
        }
        if let Ok(mut allowed_actions) = state.bookmark_context_menu_actions.lock() {
            allowed_actions.clear();
        }
        if let Ok(mut open) = state.bookmark_context_menu_open.lock() {
            *open = false;
        }
        let _ = webview.hide();
        sync_all_webviews(&app, &state)?;
        return Ok(());
    }

    let presentation = presentation.unwrap_or_default();
    let token = create_internal_bridge_token();
    let actions = actions.unwrap_or_default();
    let script = build_bookmark_context_menu_script(
        x.unwrap_or(8.0),
        y.unwrap_or(8.0),
        &actions,
        &token,
        true,
        presentation.backdrop_blur_px.unwrap_or(16).min(24),
        presentation.language.as_ref(),
    )?;

    if let Ok(mut current_token) = state.bookmark_context_menu_token.lock() {
        *current_token = Some(token);
    }
    if let Ok(mut allowed_actions) = state.bookmark_context_menu_actions.lock() {
        *allowed_actions = actions.into_iter().map(|action| action.id).collect();
    }
    if let Ok(mut open) = state.bookmark_context_menu_open.lock() {
        *open = true;
    }

    if let Some(window) = app.get_webview_window("main") {
        let scale_factor = window.scale_factor().unwrap_or(1.0);
        let logical_size = window
            .inner_size()
            .map(|size| size.to_logical::<f64>(scale_factor))
            .unwrap_or_else(|_| LogicalSize::new(1200.0, 800.0));
        webview
            .set_bounds(Rect {
                position: LogicalPosition::new(0.0, 0.0).into(),
                size: logical_size.into(),
            })
            .map_err(|error| format!("Failed to size bookmark menu overlay: {error}"))?;
    }
    webview
        .show()
        .map_err(|error| format!("Failed to show bookmark menu overlay: {error}"))?;
    webview
        .eval(&script)
        .map_err(|error| format!("Failed to render bookmark menu: {error}"))?;
    sync_all_webviews(&app, &state)?;
    Ok(())
}
