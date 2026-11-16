use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager};

pub(super) fn build_workspace_shortcut_script(token: &str, native_alt_capture: bool) -> String {
    format!(
        r#"
        (() => {{
          if (window.top !== window) return;
          const bridgeToken = "{token}";
          const nonce = new Uint32Array(4); crypto.getRandomValues(nonce);
          const documentId = Array.from(nonce, n => n.toString(16).padStart(8, "0")).join("");
          const nativeAltCapture = {native_alt_capture};
          const nativeCaretBrowsingToggle = {native_caret_browsing_toggle};
          window.__sparkWorkspaceSwitcherOpen = false;
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
              document: documentId,
              sequence: String(request.sequence)
            }});
            if (request.action) query.set("action", request.action);
            if (request.key) query.set(request.type === "link" ? "input" : "key", request.key);
            window.location.href = `spark-internal://keyboard-shortcut/${{request.type}}?${{query.toString()}}`;
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
              (binding.key === "+" && !binding.shift || event.shiftKey === binding.shift) && event.metaKey === binding.meta;
          }}

          function isHoldKey(event, binding) {{
            return binding && event.key === binding.key;
          }}

          function resetHold(type) {{
            if (!holdPressed) return;
            holdPressed = false;
            notifyBrowser(type);
          }}

          Object.defineProperty(window, "__sparkSetKeyboardShortcuts", {{
            configurable: false,
            enumerable: false,
            writable: false,
            value: (secret, value) => {{ if (secret === bridgeToken) shortcuts = value || {{}}; }}
          }});

          Object.defineProperty(window, "__sparkWorkspaceShortcutAck", {{
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

            if (event.key === "Escape" && document.fullscreenElement) {{
              event.preventDefault();
              void document.exitFullscreen();
              return;
            }}

            if (nativeCaretBrowsingToggle && event.key === "F7" && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {{
              event.preventDefault();
              event.stopImmediatePropagation();
              if (!event.repeat) notifyBrowser("action", "toggle-caret-browsing");
              return;
            }}

            if (event.key === "Escape" && window.__sparkWorkspaceSwitcherOpen) {{
              event.preventDefault();
              event.stopImmediatePropagation();
              notifyBrowser("alt-cancel");
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
                : normalizeKey(event.key) === normalizeKey(binding.key) || (binding.key === "+" && event.key === "=");
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

          window.addEventListener("mousedown", (event) => {{
            if (!event.isTrusted || (event.button !== 3 && event.button !== 4)) return;
            event.preventDefault();
            event.stopImmediatePropagation();
            const mouseKey = event.button === 3 ? "Mouse4" : "Mouse5";
            for (const [action, bindings] of Object.entries(shortcuts)) {{
              if (!Array.isArray(bindings)) continue;
              const binding = bindings[0];
              if (!binding || binding.key !== mouseKey || !modifierMatch(event, binding)) continue;
              notifyBrowser("action", action);
              return;
            }}
          }}, true);

          window.addEventListener("auxclick", (event) => {{
            if (event.isTrusted && event.button === 1 && !event.defaultPrevented) {{
              const anchor = event.target instanceof Element ? event.target.closest("a[href]") : null;
              if (anchor && anchor.href.length <= 1500 && !anchor.hasAttribute("download") && /^https?:$/.test(anchor.protocol)) {{
                const href = anchor.href;
                event.preventDefault(); setTimeout(() => notifyBrowser("link", undefined, href), 0); return;
              }}
            }}
            if (!event.isTrusted || (event.button !== 3 && event.button !== 4)) return;
            event.preventDefault();
            event.stopImmediatePropagation();
          }}, true);

          window.addEventListener("click", (event) => {{
            if (!event.isTrusted || event.defaultPrevented || event.button !== 0 || !(event.ctrlKey || event.metaKey) || event.shiftKey || event.altKey) return;
            const anchor = event.target instanceof Element ? event.target.closest("a[href]") : null;
            if (anchor && anchor.href.length <= 1500 && !anchor.hasAttribute("download") && /^https?:$/.test(anchor.protocol)) {{ const href = anchor.href; event.preventDefault(); setTimeout(() => notifyBrowser("link", undefined, href), 0); }}
          }});

          window.addEventListener("focus", () => {{
            if (!nativeAltCapture) window.location.href = "spark-internal://pane-focus?token=" + encodeURIComponent(bridgeToken);
            holdPressed = false;
          }}, true);
          document.addEventListener("visibilitychange", () => {{
            if (document.visibilityState === "hidden") resetHold("alt-cancel");
            else holdPressed = false;
          }}, true);
        }})();
        "#,
        token = token,
        native_alt_capture = native_alt_capture,
        native_caret_browsing_toggle = cfg!(target_os = "linux")
    )
}

/// Captures WebView2 accelerator events before the browser consumes the bare Alt key.
#[cfg(windows)]
pub(super) fn install_workspace_shortcut_accelerator(
    app: &AppHandle,
    webview: &tauri::Webview,
) -> Result<(), String> {
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

    let app_for_registration = app.clone();
    let label_for_registration = webview.label().to_string();
    webview
        .with_webview(move |platform: PlatformWebview| {
            let app_for_handler = app_for_registration.clone();
            let label_for_handler = label_for_registration.clone();
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

                    // WebView2 may consume browser accelerators before they reach the
                    // injected page listener. Capture every configured keyboard shortcut here.
                    if is_key_down {
                        let matched_shortcut =
                            state.policies().keyboard_shortcuts().lock().ok().and_then(
                                |shortcuts| {
                                    shortcuts.as_object().and_then(|shortcuts| {
                                        shortcuts.iter().find_map(|(action, bindings)| {
                                            if action == "workspace-picker" {
                                                return None;
                                            }

                                            let binding = bindings
                                                .as_array()
                                                .and_then(|bindings| bindings.first())?;
                                            let key = binding
                                                .get("key")
                                                .and_then(serde_json::Value::as_str)?;
                                            let shortcut_key = if key == "Digit" {
                                                let digit = match virtual_key {
                                                    0x31..=0x39 => virtual_key - 0x30,
                                                    0x61..=0x69 => virtual_key - 0x60,
                                                    _ => return None,
                                                };
                                                Some(Some(digit.to_string()))
                                            } else if shortcut_key_to_virtual_key(key)
                                                == Some(virtual_key)
                                                || (key == "+"
                                                    && matches!(virtual_key, 0xBB | 0x6B))
                                                || (key == "-" && virtual_key == 0x6D)
                                                || (key == "0" && virtual_key == 0x60)
                                            {
                                                Some(None)
                                            } else {
                                                None
                                            }?;

                                            if !shortcut_modifiers_match(binding)
                                                && !(key == "+"
                                                    && shortcut_plus_modifiers_match(binding))
                                            {
                                                return None;
                                            }

                                            Some((action.clone(), shortcut_key))
                                        })
                                    })
                                },
                            );

                        if let Some((action, key)) = matched_shortcut {
                            unsafe {
                                args.SetHandled(true)?;
                            }
                            if !is_repeat {
                                let mut payload = serde_json::json!({ "action": action });
                                if let Some(key) = key {
                                    payload["key"] = serde_json::json!(key);
                                }
                                crate::error_log::ignored(crate::security::emit_shell_for_webview(
                                    &app_for_handler,
                                    &label_for_handler,
                                    "browser-shortcut-action",
                                    payload,
                                ));
                            }
                            return Ok(());
                        }
                    }

                    if is_key_down && virtual_key == 0x1B {
                        let owners = super::fullscreen::all(&app_for_handler);
                        if !owners.is_empty() {
                            unsafe {
                                args.SetHandled(true)?;
                            }
                            for (owner, _) in owners {
                                super::fullscreen::exit(&app_for_handler, &owner, true);
                            }
                            return Ok(());
                        }
                    }

                    if is_key_down
                        && virtual_key == 0x1B
                        && state
                            .surfaces()
                            .workspace_switcher_open()
                            .lock()
                            .map(|open| *open)
                            .unwrap_or(false)
                    {
                        unsafe {
                            args.SetHandled(true)?;
                        }
                        crate::error_log::ignored(crate::security::emit_shell_for_webview(
                            &app_for_handler,
                            &label_for_handler,
                            "workspace-switcher-dismissed",
                            (),
                        ));
                        return Ok(());
                    }

                    // WebView2 reports the modifier as a native accelerator. Capture Alt only
                    // when it is still configured as the workspace-picker hold key.
                    let configured_hold_key = state
                        .policies()
                        .workspace_picker_hold_key()
                        .lock()
                        .map(|key| key.as_str() == "Alt")
                        .unwrap_or(false);
                    let workspace_shortcut_is_alt_digit = state
                        .policies()
                        .keyboard_shortcuts()
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
                                    && !state
                                        .surfaces()
                                        .workspace_alt_pressed()
                                        .swap(true, Ordering::SeqCst)
                                {
                                    Some(serde_json::json!({ "type": "alt-down" }))
                                } else {
                                    None
                                }
                            } else if state
                                .surfaces()
                                .workspace_alt_pressed()
                                .swap(false, Ordering::SeqCst)
                            {
                                Some(serde_json::json!({ "type": "alt-up" }))
                            } else {
                                None
                            }
                        } else {
                            if is_key_down && !is_repeat {
                                state
                                    .surfaces()
                                    .workspace_alt_pressed()
                                    .store(true, Ordering::SeqCst);
                            } else if is_key_up {
                                state
                                    .surfaces()
                                    .workspace_alt_pressed()
                                    .store(false, Ordering::SeqCst);
                            }
                            None
                        };

                        // WebView2 invokes this callback synchronously. Mark Alt handled
                        // before notifying the UI so its browser accelerator cannot flash.
                        unsafe {
                            args.SetHandled(true)?;
                        }

                        if let Some(payload) = payload {
                            crate::error_log::ignored(crate::security::emit_shell_for_webview(
                                &app_for_handler,
                                &label_for_handler,
                                "workspace-shortcut-event",
                                payload,
                            ));
                        }
                        return Ok(());
                    }

                    if is_key_down
                        && workspace_shortcut_is_alt_digit
                        && state
                            .surfaces()
                            .workspace_alt_pressed()
                            .load(Ordering::SeqCst)
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
                                crate::error_log::ignored(crate::security::emit_shell_for_webview(
                                    &app_for_handler,
                                    &label_for_handler,
                                    "browser-shortcut-action",
                                    serde_json::json!({
                                        "action": "switch-workspaces",
                                        "key": index.to_string()
                                    }),
                                ));
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

#[cfg(windows)]
fn is_windows_key_down(virtual_key: i32) -> bool {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetAsyncKeyState(nvirtkey: i32) -> i16;
    }

    // The newly focused WebView may not have received the modifier's key-down.
    // Read its current pressed bit, independently of a WebView thread's message queue.
    unsafe { GetAsyncKeyState(virtual_key) < 0 }
}

#[cfg(windows)]
fn shortcut_modifiers_match(binding: &serde_json::Value) -> bool {
    binding
        .get("ctrl")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
        == is_windows_key_down(0x11)
        && binding
            .get("alt")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
            == is_windows_key_down(0x12)
        && binding
            .get("shift")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
            == is_windows_key_down(0x10)
        && binding
            .get("meta")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
            == (is_windows_key_down(0x5B) || is_windows_key_down(0x5C))
}

#[cfg(windows)]
fn shortcut_key_to_virtual_key(key: &str) -> Option<u32> {
    let named_key = match key {
        "Space" => Some(0x20),
        "Enter" => Some(0x0D),
        "Escape" => Some(0x1B),
        "Backspace" => Some(0x08),
        "Tab" => Some(0x09),
        "Delete" => Some(0x2E),
        "Insert" => Some(0x2D),
        "Home" => Some(0x24),
        "End" => Some(0x23),
        "PageUp" => Some(0x21),
        "PageDown" => Some(0x22),
        "ArrowLeft" => Some(0x25),
        "ArrowUp" => Some(0x26),
        "ArrowRight" => Some(0x27),
        "ArrowDown" => Some(0x28),
        _ if key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric() => {
            key.as_bytes().first().copied().map(u32::from)
        }
        _ => None,
    };
    if named_key.is_some() {
        return named_key;
    }

    if let Some(number) = key
        .strip_prefix('F')
        .and_then(|number| number.parse::<u32>().ok())
        .filter(|number| (1..=24).contains(number))
    {
        return Some(0x6F + number);
    }

    let mut chars = key.encode_utf16();
    let character = chars.next()?;
    if chars.next().is_some() {
        return None;
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn VkKeyScanW(ch: u16) -> i16;
    }

    let mapped = unsafe { VkKeyScanW(character) };
    (mapped != -1).then_some((mapped as u16 & 0xFF) as u32)
}

pub(super) fn handle_keyboard_navigation(
    app_handle_for_nav: &AppHandle,
    nav_url: &tauri::Url,
    tab_for_nav: &str,
    label_for_nav: &str,
    keyboard_bridge_token: &str,
) {
    if crate::security::bridge::query(
        nav_url,
        &["token", "sequence", "document", "action", "key", "input"],
    )
    .is_none()
    {
        return;
    }
    let params = nav_url.query_pairs().collect::<Vec<_>>();
    let event_type = nav_url.path().trim_matches('/');
    let action = params
        .iter()
        .find(|(key, _)| key == "action")
        .map(|(_, value)| value.as_ref());
    let key = params
        .iter()
        .find(|(key, _)| key == if event_type == "link" { "input" } else { "key" })
        .map(|(_, value)| value.as_ref());
    match event_type {
        "action" if action.is_some_and(valid_action) => {}
        "link" if action.is_none() && key.is_some() => {}
        "alt-down" | "alt-up" | "alt-cancel" if action.is_none() && key.is_none() => {}
        _ => return,
    }
    let request_token = params
        .iter()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.as_ref());
    let sequence = params
        .iter()
        .find(|(key, _)| key == "sequence")
        .and_then(|(_, value)| value.parse::<u64>().ok())
        .filter(|v| *v > 0);
    if sequence.is_none() {
        return;
    }
    let token_matches = app_handle_for_nav
        .try_state::<BrowserState>()
        .and_then(|state| {
            let is_active = state.is_active_tab(tab_for_nav);
            state
                .security()
                .keyboard_bridge_tokens()
                .lock()
                .ok()
                .map(|tokens| {
                    is_active
                        && tokens.get(label_for_nav).map(String::as_str) == request_token
                        && request_token == Some(keyboard_bridge_token)
                })
        })
        .unwrap_or(false);
    let focused = app_handle_for_nav
        .try_state::<BrowserState>()
        .and_then(|state| state.window_of_tab(tab_for_nav).ok())
        .and_then(|window_id| app_handle_for_nav.get_webview_window(&window_id))
        .and_then(|window| window.is_focused().ok())
        .unwrap_or(false);

    if !token_matches {
        return;
    }
    // Link messages additionally require a trusted link gesture in the injected
    // document script. WebView2 may report the child focused while its host
    // window is not the foreground WebviewWindow.
    if focused || event_type == "link" {
        let Some(sequence) = sequence else {
            return;
        };
        let document = params
            .iter()
            .find(|(key, _)| key == "document")
            .map(|(_, value)| value.as_ref())
            .unwrap_or("");
        if !app_handle_for_nav
            .state::<BrowserState>()
            .security()
            .keyboard_bridge_sequences()
            .lock()
            .is_ok_and(|mut states| states.admit(label_for_nav, document, sequence))
        {
            return;
        }

        match event_type {
            "link" => {
                if let Some(url) = key.and_then(|raw| tauri::Url::parse(raw).ok()) {
                    crate::commands::tabs::open_user_link(
                        app_handle_for_nav.clone(),
                        tab_for_nav.to_string(),
                        keyboard_bridge_token.to_string(),
                        url,
                    );
                }
            }
            "alt-down" | "alt-up" | "alt-cancel" => {
                crate::error_log::ignored(crate::security::emit_shell_for_webview(
                    app_handle_for_nav,
                    label_for_nav,
                    "workspace-shortcut-event",
                    serde_json::json!({ "type": event_type }),
                ));
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
                if let Some(action) = action {
                    if action == "toggle-caret-browsing" {
                        #[cfg(target_os = "linux")]
                        if let Err(error) =
                            crate::window::caret_browsing::toggle(app_handle_for_nav)
                        {
                            eprintln!("Unable to toggle native caret browsing: {error}");
                        }
                    } else {
                        crate::error_log::ignored(crate::security::emit_shell_for_webview(
                            app_handle_for_nav,
                            label_for_nav,
                            "browser-shortcut-action",
                            serde_json::json!({ "action": action, "key": key }),
                        ));
                    }
                }
            }
            _ => {}
        }
    } else if !focused {
        crate::error_log::ignored(crate::security::emit_shell_for_webview(
            app_handle_for_nav,
            label_for_nav,
            "workspace-shortcut-cancel",
            (),
        ));
    }

    if let Some(sequence) = sequence {
        if let Some(webview) = app_handle_for_nav.get_webview(label_for_nav) {
            crate::error_log::ignored(
                webview.eval(format!("window.__sparkWorkspaceShortcutAck({sequence});")),
            );
        }
    }
}

fn valid_action(action: &str) -> bool {
    matches!(
        action,
        "zoom-in"
            | "zoom-out"
            | "zoom-reset"
            | "switch-tabs"
            | "next-tab"
            | "previous-tab"
            | "switch-workspaces"
            | "move-tab-up"
            | "move-tab-up-ctrl-shift"
            | "move-tab-down"
            | "move-tab-down-ctrl-shift"
            | "new-tab"
            | "navigate-back"
            | "navigate-forward"
            | "command-bar"
            | "edit-url"
            | "focus-address"
            | "sidebar"
            | "zen"
            | "history"
            | "downloads"
            | "settings"
            | "reopen-tab"
            | "close-tab"
            | "reload-tab"
            | "toggle-caret-browsing"
    )
}

#[cfg(windows)]
fn shortcut_plus_modifiers_match(binding: &serde_json::Value) -> bool {
    let mut equivalent = binding.clone();
    equivalent["shift"] = serde_json::Value::Bool(is_windows_key_down(0x10));
    shortcut_modifiers_match(&equivalent)
}
