//! Coordinates a page's native fullscreen request with the host window.
use crate::sync::Mutex;
use tauri::{AppHandle, Manager, Webview};

/// One entry per window showing a fullscreen page: (page label, window id, was fullscreen before).
#[derive(Default)]
pub struct FullscreenState(Mutex<Vec<(String, String, bool)>>);

/// The fullscreen page of every window, as (page label, window id).
pub fn all(app: &AppHandle) -> Vec<(String, String)> {
    app.state::<FullscreenState>()
        .0
        .lock()
        .map(|items| {
            items
                .iter()
                .map(|(label, window, _)| (label.clone(), window.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// The page that is fullscreen in one window.
pub fn active_in(app: &AppHandle, window_id: &str) -> Option<String> {
    all(app)
        .into_iter()
        .find(|(_, window)| window == window_id)
        .map(|(label, _)| label)
}

pub fn any_active(app: &AppHandle) -> bool {
    !all(app).is_empty()
}

fn enter(app: &AppHandle, label: &str) -> Result<(), String> {
    let browser = app.state::<crate::state::browser_state::BrowserState>();
    let tab_id = browser
        .session()
        .tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|tab| crate::state::tab_model::webview_label_for_tab(&tab.id) == label)
        .map(|tab| (tab.id.clone(), tab.window_id.clone()));
    let Some((tab_id, window_id)) = tab_id else {
        return Err("Fullscreen requested by an unknown tab".into());
    };
    let selected = browser
        .session()
        .window(&window_id)
        .ok_or("windows.not_found")?
        .active_tab_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    if selected != tab_id {
        if let Some(view) = app.get_webview(label) {
            crate::error_log::ignored(
                view.eval("if (document.fullscreenElement) void document.exitFullscreen();"),
            );
        }
        return Err("Fullscreen requested by an inactive tab".into());
    }
    let window = app
        .get_window(&window_id)
        .ok_or("Browser window unavailable")?;
    let state = app.state::<FullscreenState>();
    let mut current = state.0.lock().map_err(|e| e.to_string())?;
    if current.iter().any(|(_, window, _)| window == &window_id) {
        return Ok(());
    }
    let was_fullscreen = window.is_fullscreen().map_err(|e| e.to_string())?;
    current.push((label.to_string(), window_id.clone(), was_fullscreen));
    drop(current);
    if let Err(error) = window.set_fullscreen(true) {
        state
            .0
            .lock()
            .map_err(|e| e.to_string())?
            .retain(|(owner, _, _)| owner != label);
        return Err(error.to_string());
    }
    super::viewport::sync_content_geometry(
        app,
        &app.state::<crate::state::browser_state::BrowserState>(),
    )
}

pub fn exit(app: &AppHandle, label: &str, request_document_exit: bool) {
    let state = app.state::<FullscreenState>();
    let previous = state.0.lock().ok().and_then(|mut current| {
        let index = current.iter().position(|(owner, _, _)| owner == label)?;
        Some(current.remove(index))
    });
    if let Some((_, window_id, was_fullscreen)) = previous {
        if request_document_exit {
            if let Some(view) = app.get_webview(label) {
                let _ =
                    view.eval("if (document.fullscreenElement) void document.exitFullscreen();");
            }
        }
        if let Some(window) = app.get_window(&window_id) {
            crate::error_log::ignored(window.set_fullscreen(was_fullscreen));
        }
        crate::error_log::ignored(super::viewport::sync_content_geometry(
            app,
            &app.state::<crate::state::browser_state::BrowserState>(),
        ));
    }
}

#[cfg(windows)]
pub fn install(app: &AppHandle, view: &Webview) -> Result<(), String> {
    use webview2_com::ContainsFullScreenElementChangedEventHandler;
    let app = app.clone();
    let label = view.label().to_string();
    view.with_webview(move |platform| unsafe {
        let result = (|| {
            let core = platform.controller().CoreWebView2()?;
            let mut registration = 0;
            core.add_ContainsFullScreenElementChanged(
                &ContainsFullScreenElementChangedEventHandler::create(Box::new(
                    move |sender, _| {
                        if let Some(core) = sender {
                            let mut fullscreen = Default::default();
                            core.ContainsFullScreenElement(&mut fullscreen)?;
                            if fullscreen.as_bool() {
                                crate::error_log::ignored(enter(&app, &label));
                            } else {
                                exit(&app, &label, false);
                            }
                        }
                        Ok(())
                    },
                )),
                &mut registration,
            )?;
            Ok::<(), webview2_com::Error>(())
        })();
        if let Err(error) = result {
            crate::error_log::warn(&format!("Fullscreen handler unavailable: {error}"));
        }
    })
    .map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
pub fn install(app: &AppHandle, view: &Webview) -> Result<(), String> {
    use webkit2gtk::WebViewExt;
    let app = app.clone();
    let label = view.label().to_string();
    view.with_webview(move |platform| {
        let native = platform.inner();
        let a = app.clone();
        let l = label.clone();
        native.connect_enter_fullscreen(move |_| enter(&a, &l).is_ok());
        native.connect_leave_fullscreen(move |_| {
            exit(&app, &label, false);
            true
        });
    })
    .map_err(|e| e.to_string())
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn install(_: &AppHandle, _: &Webview) -> Result<(), String> {
    Ok(())
}
