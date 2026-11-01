use crate::state::browser_state::BrowserState;
#[cfg(windows)]
use tauri::{AppHandle, Emitter, Manager};

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

pub(super) fn build_workspace_shortcut_script(token: &str, native_alt_capture: bool) -> String {
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
            if (!event.isTrusted || (event.button !== 3 && event.button !== 4)) return;
            event.preventDefault();
            event.stopImmediatePropagation();
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

                    // WebView2 may consume browser accelerators before they reach the
                    // injected page listener. Capture every configured keyboard shortcut here.
                    if is_key_down {
                        let matched_shortcut =
                            state.keyboard_shortcuts.lock().ok().and_then(|shortcuts| {
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
                                        {
                                            Some(None)
                                        } else {
                                            None
                                        }?;

                                        if !shortcut_modifiers_match(binding) {
                                            return None;
                                        }

                                        Some((action.clone(), shortcut_key))
                                    })
                                })
                            });

                        if let Some((action, key)) = matched_shortcut {
                            unsafe {
                                args.SetHandled(true)?;
                            }
                            if !is_repeat {
                                let mut payload = serde_json::json!({ "action": action });
                                if let Some(key) = key {
                                    payload["key"] = serde_json::json!(key);
                                }
                                let _ = app_for_handler.emit("browser-shortcut-action", payload);
                            }
                            return Ok(());
                        }
                    }

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

#[cfg(windows)]
fn is_windows_key_down(virtual_key: i32) -> bool {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetKeyState(nvirtkey: i32) -> i16;
    }

    unsafe { GetKeyState(virtual_key) < 0 }
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
