//! Coordinates a page's native fullscreen request with the host window.
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Webview};

#[derive(Default)]
pub struct FullscreenState(Mutex<Option<(String, bool)>>);

pub fn active(app: &AppHandle) -> Option<String> {
    app.state::<FullscreenState>()
        .0
        .lock()
        .ok()?
        .as_ref()
        .map(|item| item.0.clone())
}

fn enter(app: &AppHandle, label: &str) -> Result<(), String> {
    let browser = app.state::<crate::state::browser_state::BrowserState>();
    let selected = browser
        .session()
        .active_tab_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    if crate::state::tab_model::webview_label_for_tab(&selected) != label {
        if let Some(view) = app.get_webview(label) {
            let _ = view.eval("if (document.fullscreenElement) void document.exitFullscreen();");
        }
        return Err("Fullscreen requested by an inactive tab".into());
    }
    let window = app.get_window("main").ok_or("Main window unavailable")?;
    let state = app.state::<FullscreenState>();
    let mut current = state.0.lock().map_err(|e| e.to_string())?;
    if current.is_some() {
        return Ok(());
    }
    let was_fullscreen = window.is_fullscreen().map_err(|e| e.to_string())?;
    *current = Some((label.to_string(), was_fullscreen));
    drop(current);
    if let Err(error) = window.set_fullscreen(true) {
        *state.0.lock().map_err(|e| e.to_string())? = None;
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
        if current.as_ref().is_some_and(|(owner, _)| owner == label) {
            current.take()
        } else {
            None
        }
    });
    if let Some((_, was_fullscreen)) = previous {
        if request_document_exit {
            if let Some(view) = app.get_webview(label) {
                let _ =
                    view.eval("if (document.fullscreenElement) void document.exitFullscreen();");
            }
        }
        if let Some(window) = app.get_window("main") {
            let _ = window.set_fullscreen(was_fullscreen);
        }
        let _ = super::viewport::sync_content_geometry(
            app,
            &app.state::<crate::state::browser_state::BrowserState>(),
        );
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
                                let _ = enter(&app, &label);
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
            eprintln!("Fullscreen handler unavailable: {error}");
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
