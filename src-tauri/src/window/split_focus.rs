use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Emitter, Manager};
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
        let _operation = state.tab_operations.lock().await;
        if super::tab_drag_preview::active(&app)
            || !super::navigation_state::is_current_generation(&app, &label, &token)
            || state.is_overlay_open.lock().map(|v| *v).unwrap_or(true)
            || state
                .workspace_switcher_open
                .lock()
                .map(|v| *v)
                .unwrap_or(true)
            || state
                .bookmark_context_menu_open
                .lock()
                .map(|v| *v)
                .unwrap_or(true)
            || !state
                .current_split()
                .is_some_and(|layout| layout.contains(&id))
            || state
                .active_tab_id
                .lock()
                .map(|active| *active == id)
                .unwrap_or(true)
        {
            return;
        }
        if state
            .select_tab(&id)
            .and_then(|_| state.persist_session())
            .is_err()
        {
            let _ = app.emit("browser-error", "Could not select the focused split pane");
            return;
        }
        let _ = app.emit("tabs-updated", state.snapshot_packet());
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
            let _ = platform
                .controller()
                .add_GotFocus(&handler, &mut registration);
        }
    })
    .map_err(|e| e.to_string())
}
