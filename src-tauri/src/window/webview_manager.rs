use crate::privacy::adblock_engine::{build_content_shield_script, should_block_url};
use crate::state::browser_state::{BrowserState, ViewportBounds, DEFAULT_HOME_URL};
use crate::state::tab_model::{derive_title_from_url, TabInfo};
use tauri::{
    webview::WebviewBuilder, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, Url,
    WebviewUrl,
};

pub const CONTENT_WEBVIEW_LABEL: &str = "content-webview";

/// Parses user omnibox input into a valid URL or a DuckDuckGo search query.
pub fn resolve_input_to_url(raw_input: &str) -> Result<Url, String> {
    let trimmed = raw_input.trim();
    if trimmed.is_empty() {
        return Url::parse(DEFAULT_HOME_URL).map_err(|e| e.to_string());
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        if let Ok(parsed) = Url::parse(trimmed) {
            return Ok(parsed);
        }
    }

    if !trimmed.contains(' ') && trimmed.contains('.') {
        let with_https = format!("https://{}", trimmed);
        if let Ok(parsed) = Url::parse(&with_https) {
            return Ok(parsed);
        }
    }

    let mut search_url =
        Url::parse("https://duckduckgo.com/").map_err(|e| e.to_string())?;
    search_url.query_pairs_mut().append_pair("q", trimmed);
    Ok(search_url)
}

/// Spawns the single persistent child WebView during application startup.
/// Using a single multiplexed child WebView completely avoids Windows WebView2
/// COM controller creation deadlocks when opening or switching tabs at runtime.
pub fn spawn_content_webview(
    app: &AppHandle,
    initial_url_str: &str,
    bounds: ViewportBounds,
) -> Result<(), String> {
    if app.get_webview(CONTENT_WEBVIEW_LABEL).is_some() {
        return Ok(());
    }

    let main_window = app
        .get_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    let initial_url = resolve_input_to_url(initial_url_str)?;
    let shield_enabled = app
        .try_state::<BrowserState>()
        .and_then(|s| s.shield_enabled.lock().ok().map(|v| *v))
        .unwrap_or(true);

    let app_handle_for_nav = app.clone();

    let webview_builder = WebviewBuilder::new(
        CONTENT_WEBVIEW_LABEL,
        WebviewUrl::External(initial_url),
    )
    .initialization_script(&build_content_shield_script(shield_enabled))
    .on_navigation(move |nav_url| {
        let new_url_str = nav_url.to_string();

        if let Some(state) = app_handle_for_nav.try_state::<BrowserState>() {
            let shield_on = state.shield_enabled.lock().map(|v| *v).unwrap_or(true);

            if should_block_url(&new_url_str, shield_on) {
                if let Ok(mut count) = state.blocked_count.lock() {
                    *count += 1;
                }
                let stats = state.snapshot_privacy_stats();
                let _ = app_handle_for_nav.emit("privacy-stats-updated", &stats);
                return false;
            }

            let active_id = state
                .active_tab_id
                .lock()
                .map(|id| id.clone())
                .unwrap_or_default();

            let new_title = derive_title_from_url(&new_url_str);
            {
                if let Ok(mut tabs) = state.tabs.lock() {
                    if let Some(tab) = tabs.iter_mut().find(|t| t.id == active_id) {
                        tab.url = new_url_str.clone();
                        tab.title = new_title.clone();
                        tab.is_hibernated = false;
                        tab.last_active_epoch = TabInfo::current_epoch_secs();
                    }
                }
            }

            state.record_history_visit(&new_title, &new_url_str);
            state.persist_session();

            let snapshot = state.snapshot_tabs();
            let _ = app_handle_for_nav.emit("tabs-updated", &snapshot);
            let _ = app_handle_for_nav.emit("content-url-changed", new_url_str);
        }
        true
    });

    main_window
        .add_child(
            webview_builder,
            LogicalPosition::new(bounds.x, bounds.y),
            LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0)),
        )
        .map_err(|e| format!("Failed to create content WebView: {}", e))?;

    Ok(())
}

/// Navigates the persistent child WebView to the specified URL without holding any state locks.
pub fn navigate_content_webview(app: &AppHandle, target_url_str: &str) -> Result<String, String> {
    let parsed_url = resolve_input_to_url(target_url_str)?;
    let resolved_str = parsed_url.to_string();

    if let Some(webview) = app.get_webview(CONTENT_WEBVIEW_LABEL) {
        webview
            .navigate(parsed_url)
            .map_err(|e| format!("Navigation error: {}", e))?;
    }

    Ok(resolved_str)
}

/// Synchronizes visibility and bounds of the child WebView.
pub fn sync_all_webviews(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let bounds = state.bounds.lock().map(|b| *b).unwrap_or_default();
    let overlay_open = state.is_overlay_open.lock().map(|o| *o).unwrap_or(false);

    if let Some(webview) = app.get_webview(CONTENT_WEBVIEW_LABEL) {
        if overlay_open {
            let _ = webview.hide();
        } else {
            let _ = webview.set_bounds(Rect {
                position: LogicalPosition::new(bounds.x, bounds.y).into(),
                size: LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0)).into(),
            });
            let _ = webview.show();
        }
    }

    Ok(())
}
