use crate::state::browser_state::BrowserState;
use crate::state::workspace_model::WorkspaceItem;
use crate::window::webview_manager::{
    active_content_webview_in, build_bookmark_context_menu_script,
    build_workspace_switcher_update_script, create_internal_bridge_token, sync_all_webviews,
    BookmarkContextMenuAction, OverlayLanguage, OverlayPresentation,
    BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL, WORKSPACE_SWITCHER_WEBVIEW_LABEL,
};
use std::time::Duration;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Rect, State};

#[tauri::command]
pub async fn set_downloads_popover(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::downloads_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::downloads_popover::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}

#[tauri::command]
pub async fn set_resources_popover(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::resources_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::resources_popover::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}

#[tauri::command]
pub async fn set_commands_popover(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::commands_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::commands_popover::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}

#[tauri::command]
pub async fn set_suggestions_popover(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::suggestions_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::suggestions_popover::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}

#[tauri::command]
pub async fn set_tools_popover(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::tools_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::tools_popover::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}

#[tauri::command]
pub async fn set_quick_settings_popover(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::quick_settings_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::quick_settings_popover::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}

#[tauri::command]
pub async fn set_zoom_popover(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::zoom_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::zoom_popover::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}

const WORKSPACE_SWITCHER_CLOSE_ANIMATION: Duration = Duration::from_millis(140);

#[tauri::command]
// Preserve the existing IPC arguments, including Tauri-injected app/webview/state.
#[allow(clippy::too_many_arguments)]
pub async fn set_settings_overlay(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::settings_overlay::Presentation>,
    restore_focus: bool,
    animate_close: Option<bool>,
    tab_id: Option<String>,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    if let Some(tab_id) = tab_id {
        if !visible {
            return Ok(());
        }
        return crate::window::settings_tab::present(
            &app,
            &crate::commands::access::caller_window(&webview),
            &tab_id,
            session_id,
            presentation.ok_or("settings_tab.missing_presentation")?,
        );
    }
    if !visible && animate_close.unwrap_or(false) {
        return crate::window::settings_overlay::close_animated(&app, session_id, restore_focus);
    }
    crate::window::settings_overlay::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}

#[tauri::command]
pub async fn set_privacy_popover(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::privacy_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::privacy_popover::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}

#[tauri::command]
pub async fn set_permission_popover(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::permission_popover::Presentation>,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::permission_popover::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
    )
}

#[tauri::command]
pub async fn set_history_overlay(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::history_overlay::Presentation>,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::history_overlay::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
    )
}

#[tauri::command]
pub async fn set_media_popover(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    session_id: u64,
    presentation: Option<crate::window::media_popover::Presentation>,
    restore_focus: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    crate::window::media_popover::set(
        &app,
        &crate::commands::access::caller_window(&webview),
        visible,
        session_id,
        presentation,
        restore_focus,
    )
}

#[tauri::command]
pub async fn set_overlay_visibility(
    app: AppHandle,
    state: State<'_, BrowserState>,
    visible: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    if let Ok(mut overlay) = state.surfaces().is_overlay_open().lock() {
        *overlay = visible;
    }
    sync_all_webviews(&app, &state)?;
    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn set_workspace_switcher_overlay(
    app: AppHandle,
    caller: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    workspaces: Vec<WorkspaceItem>,
    active_workspace_id: String,
    backdrop_blur_px: u8,
    language: Option<OverlayLanguage>,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    let window_id = crate::commands::access::caller_window(&caller);
    if visible {
        crate::window::webview_manager::host_workspace_switcher(&app, &window_id)?;
    }
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

    if let Ok(mut current_token) = state.surfaces().workspace_switcher_token().lock() {
        *current_token = Some(token.clone());
    }
    if let Ok(mut open) = state.surfaces().workspace_switcher_open().lock() {
        // Keep the native WebView shown during its exit animation.
        *open = true;
    }

    if let Some(content_webview) = active_content_webview_in(&app, &window_id) {
        crate::error_log::ignored(
            content_webview.eval(format!("window.__sparkWorkspaceSwitcherOpen = {visible};")),
        );
    }

    if let Err(error) = webview.eval(&script) {
        if let Ok(mut current_token) = state.surfaces().workspace_switcher_token().lock() {
            *current_token = None;
        }
        if let Ok(mut open) = state.surfaces().workspace_switcher_open().lock() {
            *open = false;
        }
        if let Some(content_webview) = active_content_webview_in(&app, &window_id) {
            crate::error_log::ignored(
                content_webview.eval("window.__sparkWorkspaceSwitcherOpen = false;"),
            );
        }
        crate::error_log::ignored(sync_all_webviews(&app, &state));
        return Err(format!("Failed to show workspace switcher: {error}"));
    }

    if !visible {
        let app_for_hide = app.clone();
        let hide_window = window_id.clone();
        let closing_token = token;
        std::thread::spawn(move || {
            std::thread::sleep(WORKSPACE_SWITCHER_CLOSE_ANIMATION);
            let Some(state) = app_for_hide.try_state::<BrowserState>() else {
                return;
            };

            let _operation = state.session().blocking_operation();
            // Serialize the token check and state update with a possible reopen.
            let Ok(mut current_token) = state.surfaces().workspace_switcher_token().lock() else {
                return;
            };
            if current_token.as_deref() != Some(closing_token.as_str()) {
                return;
            }
            let Ok(mut open) = state.surfaces().workspace_switcher_open().lock() else {
                return;
            };
            *current_token = None;
            *open = false;
            drop(open);
            drop(current_token);

            crate::error_log::ignored(sync_all_webviews(&app_for_hide, &state));
            let another_overlay_open = state
                .surfaces()
                .is_overlay_open()
                .lock()
                .map(|overlay| *overlay)
                .unwrap_or(true);
            let app_is_focused = app_for_hide
                .get_webview_window(&hide_window)
                .and_then(|window| window.is_focused().ok())
                .unwrap_or(false);
            if app_is_focused {
                if another_overlay_open {
                    if let Some(main_window) = app_for_hide.get_webview_window(&hide_window) {
                        crate::error_log::ignored(main_window.set_focus());
                    }
                } else if let Some(content_webview) =
                    active_content_webview_in(&app_for_hide, &hide_window)
                {
                    crate::error_log::ignored(content_webview.set_focus());
                }
            }
        });
        return Ok(());
    }

    sync_all_webviews(&app, &state)?;
    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn set_bookmark_context_menu_overlay(
    app: AppHandle,
    caller: tauri::Webview,
    state: State<'_, BrowserState>,
    visible: bool,
    x: Option<f64>,
    y: Option<f64>,
    actions: Option<Vec<BookmarkContextMenuAction>>,
    presentation: Option<OverlayPresentation>,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    let window_id = crate::commands::access::caller_window(&caller);
    if visible {
        crate::window::webview_manager::host_bookmark_context_menu(&app, &window_id)?;
    }
    let webview = app
        .get_webview(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL)
        .ok_or_else(|| "Bookmark context menu WebView was not found".to_string())?;

    if !visible {
        if let Ok(mut token) = state.surfaces().bookmark_context_menu_token().lock() {
            *token = None;
        }
        if let Ok(mut allowed_actions) = state.surfaces().bookmark_context_menu_actions().lock() {
            allowed_actions.clear();
        }
        if let Ok(mut open) = state.surfaces().bookmark_context_menu_open().lock() {
            *open = false;
        }
        crate::error_log::ignored(webview.hide());
        sync_all_webviews(&app, &state)?;
        return Ok(());
    }

    let presentation = presentation.unwrap_or_default();
    let token = create_internal_bridge_token();
    let actions: Vec<_> = actions.unwrap_or_default().into_iter().map(BookmarkContextMenuAction::sanitized).collect();
    let script = build_bookmark_context_menu_script(
        x.unwrap_or(8.0),
        y.unwrap_or(8.0),
        &actions,
        &token,
        true,
        presentation.backdrop_blur_px.unwrap_or(16).min(24),
        presentation.language.as_ref(),
    )?;

    if let Ok(mut current_token) = state.surfaces().bookmark_context_menu_token().lock() {
        *current_token = Some(token);
    }
    if let Ok(mut allowed_actions) = state.surfaces().bookmark_context_menu_actions().lock() {
        *allowed_actions = actions
            .iter()
            .flat_map(|action| {
                let own = (!action.heading && !action.separator && action.swatches.is_empty())
                    .then(|| action.id.clone());
                own.into_iter()
                    .chain(action.swatches.iter().map(|swatch| swatch.id.clone()))
            })
            .collect();
    }
    if let Ok(mut open) = state.surfaces().bookmark_context_menu_open().lock() {
        *open = true;
    }

    if let Some(window) = app.get_webview_window(&window_id) {
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
    crate::window::show_overlay_webview(&webview)
        .map_err(|error| format!("Failed to show bookmark menu overlay: {error}"))?;
    webview
        .eval(&script)
        .map_err(|error| format!("Failed to render bookmark menu: {error}"))?;
    sync_all_webviews(&app, &state)?;
    Ok(())
}
