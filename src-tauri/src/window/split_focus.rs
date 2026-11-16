use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Manager};
pub fn focus_pane(app: &AppHandle, tab_id: &str, label: &str, token: &str) {
    // Capture this guard before queuing: presentation events may finish after drop.
    if super::tab_drag_preview::active(app) {
        return;
    }
    let (app, id, label, token) = (
        app.clone(),
        tab_id.to_owned(),
        label.to_owned(),
        token.to_owned(),
    );
    tauri::async_runtime::spawn(async move {
        let state = app.state::<BrowserState>();
        let _operation = state.session().operation().await;
        if super::tab_drag_preview::active(&app)
            || !super::navigation_state::is_current_generation(&app, &label, &token)
            || state
                .surfaces()
                .is_overlay_open()
                .lock()
                .map(|v| *v)
                .unwrap_or(true)
            || state
                .surfaces()
                .workspace_switcher_open()
                .lock()
                .map(|v| *v)
                .unwrap_or(true)
            || state
                .surfaces()
                .bookmark_context_menu_open()
                .lock()
                .map(|v| *v)
                .unwrap_or(true)
            || !state
                .window_of_tab(&id)
                .ok()
                .and_then(|window| state.current_split_in(&window))
                .is_some_and(|layout| layout.contains(&id))
            || state.active_tab_ids().contains(&id)
        {
            return;
        }
        if state.select_tab_committed(&id).is_err() {
            crate::error_log::ignored(crate::security::emit_shell(
                &app,
                "browser-error",
                "Could not select the focused split pane",
            ));
            return;
        }
        if let Ok(window) = state.window_of_tab(&id) {
            crate::state::publish::publish_tabs(&app, &state, &window);
        }
    });
}
#[cfg(windows)]
pub fn install(
    app: &AppHandle,
    view: &tauri::Webview,
    tab_id: &str,
    token: &str,
) -> Result<(), String> {
    let (app, id, label, token) = (
        app.clone(),
        tab_id.to_owned(),
        view.label().to_owned(),
        token.to_owned(),
    );
    view.with_webview(move |platform| {
        let handler = webview2_com::FocusChangedEventHandler::create(Box::new(move |_, _| {
            focus_pane(&app, &id, &label, &token);
            Ok(())
        }));
        let mut registration = 0;
        unsafe {
            crate::error_log::ignored(
                platform
                    .controller()
                    .add_GotFocus(&handler, &mut registration),
            );
        }
    })
    .map_err(|e| e.to_string())
}

/// GTK counterpart: a pane becomes active when its WebView receives keyboard focus.
#[cfg(target_os = "linux")]
pub fn install(
    app: &AppHandle,
    view: &tauri::Webview,
    tab_id: &str,
    token: &str,
) -> Result<(), String> {
    let (app, id, label, token) = (
        app.clone(),
        tab_id.to_owned(),
        view.label().to_owned(),
        token.to_owned(),
    );
    view.with_webview(move |platform| {
        use gtk::prelude::WidgetExt;
        platform.inner().connect_focus_in_event(move |_, _| {
            focus_pane(&app, &id, &label, &token);
            gtk::glib::Propagation::Proceed
        });
    })
    .map_err(|e| e.to_string())
}
