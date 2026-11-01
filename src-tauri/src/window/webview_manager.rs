use crate::privacy::adblock_engine::{build_content_shield_script, should_block_url};
use crate::state::browser_state::{BrowserState, ViewportBounds, DEFAULT_HOME_URL};
use crate::state::tab_model::{derive_title_from_url, TabInfo};
use crate::state::workspace_model::WorkspaceItem;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    webview::WebviewBuilder, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, Url,
    WebviewUrl,
};

pub const CONTENT_WEBVIEW_LABEL: &str = "content-webview";
pub const WORKSPACE_SWITCHER_WEBVIEW_LABEL: &str = "workspace-switcher-webview";

pub fn create_internal_bridge_token() -> String {
    let mut hasher = RandomState::new().build_hasher();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn build_workspace_switcher_update_script(
    workspaces: &[WorkspaceItem],
    active_workspace_id: &str,
    token: &str,
    visible: bool,
    backdrop_blur_px: u8,
) -> Result<String, String> {
    let payload = serde_json::to_string(&serde_json::json!({
        "workspaces": workspaces,
        "activeWorkspaceId": active_workspace_id,
        "token": token,
        "visible": visible,
        "backdropBlurPx": backdrop_blur_px.min(24),
    }))
    .map_err(|error| format!("Failed to serialize workspace switcher: {error}"))?;
    Ok(format!(
        "(() => {{ const payload = {payload}; let attempts = 0; const update = () => {{ if (typeof window.__veloceSetWorkspaceSwitcher === 'function') {{ window.__veloceSetWorkspaceSwitcher(payload); return; }} if (++attempts < 120) setTimeout(update, 16); }}; update(); }})();"
    ))
}

pub const WORKSPACE_SWITCHER_ESCAPE_SCRIPT: &str = r#"
  (() => {
    window.__veloceWorkspaceSwitcherOpen = false;
    window.addEventListener("keydown", (event) => {
      if (event.key !== "Escape" || !window.__veloceWorkspaceSwitcherOpen) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      window.location.href = "veloce-internal://workspace-switcher/dismiss";
    }, true);
  })();
"#;

fn build_workspace_shortcut_script(token: &str, native_alt_capture: bool) -> String {
    format!(
        r#"
        (() => {{
          if (window.top !== window) return;
          const bridgeToken = "{token}";
          const nativeAltCapture = {native_alt_capture};
          let shortcuts = {{}};
          let holdPressed = false;
          let nextSequence = 0;
          let requestInFlight = false;
          const requestQueue = [];

          function flushRequestQueue() {{
            if (requestInFlight || requestQueue.length === 0) return;
            requestInFlight = true;
            const request = requestQueue[0];
            const query = new URLSearchParams({{
              token: bridgeToken,
              sequence: String(request.sequence)
            }});
            if (request.index !== undefined) query.set("index", String(request.index));
            if (request.action) query.set("action", request.action);
            if (request.key) query.set("key", request.key);
            window.location.href = `veloce-internal://keyboard-shortcut/${{request.type}}?${{query.toString()}}`;
          }}

          function notifyBrowser(type, action, key) {{
            requestQueue.push({{ type, action, key, sequence: ++nextSequence }});
            flushRequestQueue();
          }}

          function normalizeKey(key) {{
            return key.length === 1 ? key.toUpperCase() : key;
          }}

          function modifierMatch(event, binding) {{
            return event.ctrlKey === binding.ctrl && event.altKey === binding.alt &&
              event.shiftKey === binding.shift && event.metaKey === binding.meta;
          }}

          function isHoldKey(event, binding) {{
            return binding && event.key === binding.key;
          }}

          function resetHold(type) {{
            if (!holdPressed) return;
            holdPressed = false;
            notifyBrowser(type);
          }}

          Object.defineProperty(window, "__veloceSetKeyboardShortcuts", {{
            configurable: false,
            enumerable: false,
            writable: false,
            value: (secret, value) => {{ if (secret === bridgeToken) shortcuts = value || {{}}; }}
          }});

          Object.defineProperty(window, "__veloceWorkspaceShortcutAck", {{
            configurable: false,
            enumerable: false,
            writable: false,
            value: (sequence) => {{
              if (!requestInFlight || requestQueue[0]?.sequence !== sequence) return;
              requestQueue.shift();
              requestInFlight = false;
              flushRequestQueue();
            }}
          }});

          window.addEventListener("keydown", (event) => {{
            if (!event.isTrusted) return;

            if (event.key === "Escape" && window.__veloceWorkspaceSwitcherOpen) {{
              event.preventDefault();
              event.stopImmediatePropagation();
              window.location.href = "veloce-internal://workspace-switcher/dismiss";
              return;
            }}

            const holdBinding = shortcuts["workspace-picker"]?.[0];
            if (isHoldKey(event, holdBinding)) {{
              if (nativeAltCapture && holdBinding.key === "Alt") return;
              if (!event.repeat && !holdPressed) {{
                holdPressed = true;
                event.preventDefault();
                notifyBrowser("alt-down");
              }}
              return;
            }}

            if (holdPressed) resetHold("alt-up");

            if (event.repeat) return;
            for (const [action, bindings] of Object.entries(shortcuts)) {{
              if (action === "workspace-picker" || !Array.isArray(bindings)) continue;
              const binding = bindings[0];
              if (!binding || !modifierMatch(event, binding)) continue;
              if (
                nativeAltCapture && action === "switch-workspaces" &&
                binding.key === "Digit" && binding.alt && !binding.ctrl && !binding.shift && !binding.meta
              ) return;
              const keyMatches = binding.key === "Digit"
                ? /^[1-9]$/.test(event.key)
                : normalizeKey(event.key) === normalizeKey(binding.key);
              if (!keyMatches) continue;
              event.preventDefault();
              event.stopImmediatePropagation();
              notifyBrowser("action", action, event.key);
              return;
            }}
          }}, true);

          window.addEventListener("keyup", (event) => {{
            if (!event.isTrusted) return;
            const holdBinding = shortcuts["workspace-picker"]?.[0];
            if (!isHoldKey(event, holdBinding)) return;
            if (nativeAltCapture && holdBinding.key === "Alt") return;
            resetHold("alt-up");
          }}, true);

          window.addEventListener("focus", () => {{
            holdPressed = false;
          }}, true);
          document.addEventListener("visibilitychange", () => {{
            if (document.visibilityState === "hidden") resetHold("alt-cancel");
            else holdPressed = false;
          }}, true);
        }})();
        "#,
        token = token,
        native_alt_capture = native_alt_capture
    )
}

/// Captures WebView2 accelerator events before the browser consumes the bare Alt key.
#[cfg(windows)]
fn install_workspace_shortcut_accelerator(app: &AppHandle) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    use tauri::webview::PlatformWebview;
    use webview2_com::{
        AcceleratorKeyPressedEventHandler,
        Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_KEY_EVENT_KIND, COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN,
            COREWEBVIEW2_KEY_EVENT_KIND_KEY_UP, COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN,
            COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_UP,
        },
    };

    let webview = app
        .get_webview(CONTENT_WEBVIEW_LABEL)
        .ok_or_else(|| "Content WebView was not found for keyboard setup".to_string())?;
    let app_for_registration = app.clone();
    webview
        .with_webview(move |platform: PlatformWebview| {
            let app_for_handler = app_for_registration.clone();
            let handler =
                AcceleratorKeyPressedEventHandler::create(Box::new(move |_sender, args| {
                    let Some(args) = args else {
                        return Ok(());
                    };

                    let mut event_kind = COREWEBVIEW2_KEY_EVENT_KIND::default();
                    let mut virtual_key = 0_u32;
                    let mut key_lparam = 0_i32;
                    unsafe {
                        args.KeyEventKind(&mut event_kind)?;
                        args.VirtualKey(&mut virtual_key)?;
                        args.KeyEventLParam(&mut key_lparam)?;
                    }

                    let is_key_down = event_kind == COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN
                        || event_kind == COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN;
                    let is_key_up = event_kind == COREWEBVIEW2_KEY_EVENT_KIND_KEY_UP
                        || event_kind == COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_UP;
                    if !is_key_down && !is_key_up {
                        return Ok(());
                    }

                    let is_repeat = is_key_down && (key_lparam & (1 << 30)) != 0;
                    let state = app_for_handler.state::<BrowserState>();

                    if is_key_down
                        && virtual_key == 0x1B
                        && state
                            .workspace_switcher_open
                            .lock()
                            .map(|open| *open)
                            .unwrap_or(false)
                    {
                        unsafe {
                            args.SetHandled(true)?;
                        }
                        let _ = app_for_handler.emit("workspace-switcher-dismissed", ());
                        return Ok(());
                    }

                    // WebView2 reports the modifier as a native accelerator. Capture Alt only
                    // when it is still configured as the workspace-picker hold key.
                    let configured_hold_key = state
                        .workspace_picker_hold_key
                        .lock()
                        .map(|key| key.as_str() == "Alt")
                        .unwrap_or(false);
                    let workspace_shortcut_is_alt_digit = state
                        .keyboard_shortcuts
                        .lock()
                        .ok()
                        .and_then(|shortcuts| {
                            shortcuts
                                .get("switch-workspaces")
                                .and_then(serde_json::Value::as_array)
                                .and_then(|bindings| bindings.first())
                                .map(|binding| {
                                    binding.get("key").and_then(serde_json::Value::as_str)
                                        == Some("Digit")
                                        && binding.get("alt").and_then(serde_json::Value::as_bool)
                                            == Some(true)
                                        && binding.get("ctrl").and_then(serde_json::Value::as_bool)
                                            == Some(false)
                                        && binding.get("shift").and_then(serde_json::Value::as_bool)
                                            == Some(false)
                                        && binding.get("meta").and_then(serde_json::Value::as_bool)
                                            == Some(false)
                                })
                        })
                        .unwrap_or(false);
                    if virtual_key == 0x12
                        && (configured_hold_key || workspace_shortcut_is_alt_digit)
                    {
                        let payload = if configured_hold_key {
                            if is_key_down {
                                if !is_repeat
                                    && !state.workspace_alt_pressed.swap(true, Ordering::SeqCst)
                                {
                                    Some(serde_json::json!({ "type": "alt-down" }))
                                } else {
                                    None
                                }
                            } else if state.workspace_alt_pressed.swap(false, Ordering::SeqCst) {
                                Some(serde_json::json!({ "type": "alt-up" }))
                            } else {
                                None
                            }
                        } else {
                            if is_key_down && !is_repeat {
                                state.workspace_alt_pressed.store(true, Ordering::SeqCst);
                            } else if is_key_up {
                                state.workspace_alt_pressed.store(false, Ordering::SeqCst);
                            }
                            None
                        };

                        // WebView2 invokes this callback synchronously. Mark Alt handled
                        // before notifying the UI so its browser accelerator cannot flash.
                        unsafe {
                            args.SetHandled(true)?;
                        }

                        if let Some(payload) = payload {
                            let _ = app_for_handler.emit("workspace-shortcut-event", payload);
                        }
                        return Ok(());
                    }

                    if is_key_down
                        && workspace_shortcut_is_alt_digit
                        && state.workspace_alt_pressed.load(Ordering::SeqCst)
                    {
                        let index = match virtual_key {
                            49..=57 => Some((virtual_key - 48) as u8),
                            97..=105 => Some((virtual_key - 96) as u8),
                            _ => None,
                        };
                        if let Some(index) = index {
                            unsafe {
                                args.SetHandled(true)?;
                            }
                            if !is_repeat {
                                let _ = app_for_handler.emit(
                                    "browser-shortcut-action",
                                    serde_json::json!({
                                        "action": "switch-workspaces",
                                        "key": index.to_string()
                                    }),
                                );
                            }
                            return Ok(());
                        }
                    }

                    Ok(())
                }));

            let mut event_token = 0_i64;
            let result = unsafe {
                platform
                    .controller()
                    .add_AcceleratorKeyPressed(&handler, &mut event_token)
            };
            if let Err(error) = result {
                eprintln!("Failed to install workspace keyboard handler: {error}");
            }
        })
        .map_err(|error| format!("Failed to install workspace keyboard handler: {error}"))?;

    Ok(())
}

/// Parses user omnibox input into a valid URL or a Google search query.
pub fn resolve_input_to_url(raw_input: &str) -> Result<Url, String> {
    resolve_input_to_url_with_search_engine(raw_input, "google")
}

/// Parses user omnibox input using one of the supported search engines.
pub fn resolve_input_to_url_with_search_engine(
    raw_input: &str,
    search_engine: &str,
) -> Result<Url, String> {
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

    let search_url_base = match search_engine {
        "duckduckgo" => "https://duckduckgo.com/",
        _ => "https://www.google.com/search",
    };
    let mut search_url = Url::parse(search_url_base).map_err(|e| e.to_string())?;
    search_url.query_pairs_mut().append_pair("q", trimmed);
    Ok(search_url)
}

pub fn search_engine_home_url(search_engine: Option<&str>) -> &'static str {
    match search_engine {
        Some("duckduckgo") => "https://duckduckgo.com/",
        _ => "https://www.google.com/",
    }
}

fn handle_workspace_switcher_navigation(app: &AppHandle, nav_url: &Url) {
    if nav_url.scheme() != "veloce-internal" || nav_url.host_str() != Some("workspace-switcher") {
        return;
    }

    let action = nav_url.path().trim_matches('/');
    let request_token = nav_url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.into_owned());
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
            let workspace_id = nav_url
                .query_pairs()
                .find(|(key, _)| key == "workspace_id")
                .map(|(_, value)| value.into_owned());
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
            let _ = app.emit("workspace-switcher-selected", workspace_id);
        }
        "dismiss" => {
            let is_open = state
                .workspace_switcher_open
                .lock()
                .map(|open| *open)
                .unwrap_or(false);
            if is_open {
                let _ = app.emit("workspace-switcher-dismissed", ());
            }
        }
        _ => {}
    }
}

fn spawn_workspace_switcher_webview(app: &AppHandle, bounds: ViewportBounds) -> Result<(), String> {
    if app.get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL).is_some() {
        return Ok(());
    }

    let main_window = app
        .get_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;
    let app_for_navigation = app.clone();
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
        true
    });

    main_window
        .add_child(
            webview_builder,
            LogicalPosition::new(bounds.x, bounds.y),
            LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0)),
        )
        .map_err(|error| format!("Failed to create workspace switcher WebView: {error}"))?;

    if let Some(webview) = app.get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL) {
        let _ = webview.hide();
    }

    Ok(())
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
    let keyboard_bridge_token = create_internal_bridge_token();
    if let Some(state) = app.try_state::<BrowserState>() {
        if let Ok(mut token) = state.keyboard_bridge_token.lock() {
            *token = Some(keyboard_bridge_token.clone());
        }
    }
    #[cfg(windows)]
    let native_alt_capture = true;
    #[cfg(not(windows))]
    let native_alt_capture = false;

    let app_handle_for_nav = app.clone();

    let webview_builder =
        WebviewBuilder::new(CONTENT_WEBVIEW_LABEL, WebviewUrl::External(initial_url))
            .initialization_script(&build_content_shield_script(shield_enabled))
            .initialization_script(WORKSPACE_SWITCHER_ESCAPE_SCRIPT);

    let webview_builder = webview_builder.initialization_script(build_workspace_shortcut_script(
        &keyboard_bridge_token,
        native_alt_capture,
    ));

    let app_for_page_load = app.clone();
    let keyboard_token_for_load = keyboard_bridge_token.clone();
    let webview_builder = webview_builder.on_page_load(move |webview, payload| {
        if payload.event() != tauri::webview::PageLoadEvent::Finished {
            return;
        }
        let Some(state) = app_for_page_load.try_state::<BrowserState>() else {
            return;
        };
        let Ok(shortcuts) = state.keyboard_shortcuts.lock() else {
            return;
        };
        if let Ok(serialized) = serde_json::to_string(&*shortcuts) {
            let _ = webview.eval(&format!(
                "window.__veloceSetKeyboardShortcuts?.(\"{}\", {serialized});",
                keyboard_token_for_load
            ));
        }
    });

    let webview_builder = webview_builder.on_navigation(move |nav_url| {
        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("workspace-switcher")
        {
            handle_workspace_switcher_navigation(&app_handle_for_nav, nav_url);
            return false;
        }

        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("keyboard-shortcut")
        {
            let params = nav_url.query_pairs().collect::<Vec<_>>();
            let request_token = params
                .iter()
                .find(|(key, _)| key == "token")
                .map(|(_, value)| value.as_ref());
            let sequence = params
                .iter()
                .find(|(key, _)| key == "sequence")
                .and_then(|(_, value)| value.parse::<u64>().ok());
            let token_matches = app_handle_for_nav
                .try_state::<BrowserState>()
                .and_then(|state| {
                    state.keyboard_bridge_token.lock().ok().map(|token| {
                        token.as_deref() == request_token
                            && token.as_deref() == Some(keyboard_bridge_token.as_str())
                    })
                })
                .unwrap_or(false);
            let focused = app_handle_for_nav
                .get_webview_window("main")
                .and_then(|window| window.is_focused().ok())
                .unwrap_or(false);

            if token_matches && focused {
                let event_type = nav_url.path().trim_matches('/');
                match event_type {
                    "alt-down" | "alt-up" | "alt-cancel" => {
                        let _ = app_handle_for_nav.emit(
                            "workspace-shortcut-event",
                            serde_json::json!({ "type": event_type }),
                        );
                    }
                    "action" => {
                        let action = params
                            .iter()
                            .find(|(key, _)| key == "action")
                            .map(|(_, value)| value.as_ref());
                        let key = params
                            .iter()
                            .find(|(key, _)| key == "key")
                            .map(|(_, value)| value.as_ref());
                        if let Some(action) = action.filter(|action| {
                            matches!(
                                *action,
                                "switch-tabs"
                                    | "switch-workspaces"
                                    | "move-tab-up"
                                    | "move-tab-up-ctrl-shift"
                                    | "move-tab-down"
                                    | "move-tab-down-ctrl-shift"
                                    | "new-tab"
                                    | "command-bar"
                                    | "edit-url"
                                    | "sidebar"
                                    | "zen"
                                    | "history"
                                    | "downloads"
                                    | "settings"
                                    | "close-tab"
                                    | "reload-tab"
                            )
                        }) {
                            let _ = app_handle_for_nav.emit(
                                "browser-shortcut-action",
                                serde_json::json!({ "action": action, "key": key }),
                            );
                        }
                    }
                    _ => {}
                }
            } else if !focused {
                let _ = app_handle_for_nav.emit("workspace-shortcut-cancel", ());
            }

            if let Some(sequence) = sequence {
                if let Some(webview) = app_handle_for_nav.get_webview(CONTENT_WEBVIEW_LABEL) {
                    let _ =
                        webview.eval(&format!("window.__veloceWorkspaceShortcutAck({sequence});"));
                }
            }
            return false;
        }

        // No internal URL is a supported keyboard transport on Windows; keep
        // the channel scheme from ever reaching the web content as navigation.
        if nav_url.scheme() == "veloce-internal" {
            return false;
        }

        if let Some(state) = app_handle_for_nav.try_state::<BrowserState>() {
            state
                .workspace_alt_pressed
                .store(false, std::sync::atomic::Ordering::SeqCst);
            if let Ok(mut token) = state.workspace_switcher_token.lock() {
                *token = None;
            }
            if let Ok(mut open) = state.workspace_switcher_open.lock() {
                *open = false;
            }
        }
        let _ = app_handle_for_nav.emit("workspace-shortcut-cancel", ());
        let _ = app_handle_for_nav.emit("workspace-switcher-dismissed", ());

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

    spawn_workspace_switcher_webview(app, bounds)?;

    #[cfg(windows)]
    install_workspace_shortcut_accelerator(app)?;

    Ok(())
}

/// Navigates the persistent child WebView to the specified URL without holding any state locks.
pub fn navigate_content_webview(app: &AppHandle, target_url_str: &str) -> Result<String, String> {
    navigate_content_webview_with_search_engine(app, target_url_str, "google")
}

/// Navigates the persistent child WebView using a supported search engine for text queries.
pub fn navigate_content_webview_with_search_engine(
    app: &AppHandle,
    target_url_str: &str,
    search_engine: &str,
) -> Result<String, String> {
    let parsed_url = resolve_input_to_url_with_search_engine(target_url_str, search_engine)?;
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
    let workspace_switcher_open = state
        .workspace_switcher_open
        .lock()
        .map(|open| *open)
        .unwrap_or(false);

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

    if let Some(webview) = app.get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL) {
        if workspace_switcher_open {
            let _ = webview.set_bounds(Rect {
                position: LogicalPosition::new(bounds.x, bounds.y).into(),
                size: LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0)).into(),
            });
            let _ = webview.show();
        } else {
            let _ = webview.hide();
        }
    }

    Ok(())
}
