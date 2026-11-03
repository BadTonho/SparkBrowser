use crate::state::{browser_state::BrowserState, tab_model::TabInfo};
use tauri::{AppHandle, Emitter, Manager};

pub(super) fn is_current_generation(app: &AppHandle, label: &str, token: &str) -> bool {
    app.state::<BrowserState>()
        .keyboard_bridge_tokens
        .lock()
        .map(|tokens| tokens.get(label).map(String::as_str) == Some(token))
        .unwrap_or(false)
}

/// Generation checks discard callbacks from closed/recreated WebViews.
pub(super) fn update_navigation(
    app: &AppHandle,
    tab_id: &str,
    label: &str,
    token: &str,
    update: impl FnOnce(&mut TabInfo),
) {
    let state = app.state::<BrowserState>();
    if !state
        .keyboard_bridge_tokens
        .lock()
        .map(|tokens| tokens.get(label).map(String::as_str) == Some(token))
        .unwrap_or(false)
    {
        return;
    }
    let Ok(mut tabs) = state.tabs.lock() else {
        return;
    };
    let Some(tab) = tabs.iter_mut().find(|tab| tab.id == tab_id) else {
        return;
    };
    let before = (
        tab.url.clone(),
        tab.title.clone(),
        tab.navigation.clone(),
        tab.activity.clone(),
    );
    update(tab);
    if before
        == (
            tab.url.clone(),
            tab.title.clone(),
            tab.navigation.clone(),
            tab.activity.clone(),
        )
    {
        return;
    }
    tab.navigation.revision += 1;
    let payload = serde_json::json!({ "id": tab.id, "url": tab.url, "title": tab.title, "navigation": tab.navigation, "activity": tab.activity });
    drop(tabs);
    let _ = app.emit("tab-navigation-updated", payload);
}

#[cfg(windows)]
pub(super) fn install_native_navigation(
    app: &AppHandle,
    view: &tauri::Webview,
    tab_id: &str,
    token: &str,
) -> Result<(), String> {
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };
    use webview2_com::{
        CoTaskMemPWSTR, HistoryChangedEventHandler, NavigationCompletedEventHandler,
        NavigationStartingEventHandler, SourceChangedEventHandler,
    };
    let app = app.clone();
    let tab_id = tab_id.to_owned();
    let label = view.label().to_owned();
    let token = token.to_owned();
    view.with_webview(move |platform| {
        let result = (|| {
            let core = unsafe { platform.controller().CoreWebView2()? };
            let mut registration = 0;
            let current = Arc::new(AtomicU64::new(0));
            let starting_id = current.clone();
            let starting = NavigationStartingEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    let mut id = 0;
                    unsafe {
                        args.NavigationId(&mut id)?;
                    }
                    starting_id.store(id, Ordering::SeqCst);
                }
                Ok(())
            }));
            let (a, t, l, secret) = (app.clone(), tab_id.clone(), label.clone(), token.clone());
            let history = HistoryChangedEventHandler::create(Box::new(move |sender, _| {
                if let Some(core) = sender {
                    let mut back = Default::default();
                    let mut forward = Default::default();
                    unsafe {
                        core.CanGoBack(&mut back)?;
                        core.CanGoForward(&mut forward)?;
                    }
                    update_navigation(&a, &t, &l, &secret, |tab| {
                        tab.navigation.can_go_back = Some(back.as_bool());
                        tab.navigation.can_go_forward = Some(forward.as_bool());
                    });
                }
                Ok(())
            }));
            let (a, t, l, secret) = (app.clone(), tab_id.clone(), label.clone(), token.clone());
            let source = SourceChangedEventHandler::create(Box::new(move |sender, _| {
                if let Some(core) = sender {
                    let mut value = Default::default();
                    unsafe {
                        core.Source(&mut value)?;
                    }
                    let url = CoTaskMemPWSTR::from(value).to_string();
                    if url::Url::parse(&url)
                        .is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
                    {
                        update_navigation(&a, &t, &l, &secret, |tab| tab.url = url);
                    }
                }
                Ok(())
            }));
            let (a, t, l, secret) = (app.clone(), tab_id.clone(), label.clone(), token.clone());
            let completed = NavigationCompletedEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    let mut id = 0;
                    let mut success = Default::default();
                    let mut status = Default::default();
                    unsafe {
                        args.NavigationId(&mut id)?;
                        args.IsSuccess(&mut success)?;
                        args.WebErrorStatus(&mut status)?;
                    }
                    let latest = current.load(Ordering::SeqCst);
                    if latest == 0 || latest == id {
                        update_navigation(&a, &t, &l, &secret, |tab| {
                            tab.navigation.loading = false;
                            tab.navigation.error = (!success.as_bool()).then(|| {
                                format!("Navigation failed (WebView2 status {})", status.0)
                            });
                        });
                    }
                }
                Ok(())
            }));
            unsafe {
                core.add_NavigationStarting(&starting, &mut registration)?;
                core.add_HistoryChanged(&history, &mut registration)?;
                core.add_SourceChanged(&source, &mut registration)?;
                core.add_NavigationCompleted(&completed, &mut registration)?;
            }
            Ok::<(), webview2_com::Error>(())
        })();
        if result.is_err() {
            let _ = app.emit(
                "browser-error",
                "Could not attach native navigation observers",
            );
        }
    })
    .map_err(|e| e.to_string())
}

pub(super) fn with_page_events(
    webview_builder: tauri::webview::WebviewBuilder<tauri::Wry>,
    app: &AppHandle,
    tab_id: &str,
    label: &str,
    keyboard_bridge_token: &str,
) -> tauri::webview::WebviewBuilder<tauri::Wry> {
    let app_for_page_load = app.clone();
    let keyboard_token_for_load = keyboard_bridge_token.to_owned();
    let tab_for_load = tab_id.to_string();
    let label_for_load = label.to_owned();
    let webview_builder = webview_builder.on_page_load(move |webview, payload| {
        if !super::navigation_state::is_current_generation(
            &app_for_page_load,
            &label_for_load,
            &keyboard_token_for_load,
        ) {
            return;
        }
        let started = payload.event() == tauri::webview::PageLoadEvent::Started;
        update_navigation(
            &app_for_page_load,
            &tab_for_load,
            &label_for_load,
            &keyboard_token_for_load,
            |tab| {
                tab.navigation.loading = started;
                if started {
                    tab.navigation.error = None;
                    tab.activity.dirty_form = false;
                    tab.activity.document_media = false;
                    tab.activity.frame_media = false;
                    tab.activity.native_audio = false;
                    tab.activity.picture_in_picture = false;
                    tab.activity.refresh_media();
                }
                if matches!(payload.url().scheme(), "http" | "https") {
                    tab.url = payload.url().to_string();
                }
            },
        );
        if matches!(payload.url().scheme(), "http" | "https") {
            let _ = crate::zoom::apply(&app_for_page_load, &label_for_load, payload.url().as_str());
        }
        if started {
            return;
        }
        let Some(state) = app_for_page_load.try_state::<BrowserState>() else {
            return;
        };
        crate::privacy::auto_skip::loaded(&webview, &state, &keyboard_token_for_load);
        let title = state
            .tabs
            .lock()
            .ok()
            .and_then(|tabs| {
                tabs.iter()
                    .find(|tab| tab.id == tab_for_load)
                    .map(|tab| tab.title.clone())
            })
            .unwrap_or_default();
        if matches!(payload.url().scheme(), "http" | "https") {
            if let Some(writer) = app_for_page_load
                .try_state::<crate::database::navigation_writer::NavigationWriter>(
            ) {
                if writer.visit(title, payload.url().to_string()).is_err() {
                    let _ = app_for_page_load.emit(
                        "browser-error",
                        "Navigation persistence queue is unavailable",
                    );
                }
            }
        }
        let Ok(shortcuts) = state.keyboard_shortcuts.lock() else {
            return;
        };
        if let Ok(serialized) = serde_json::to_string(&*shortcuts) {
            let _ = webview.eval(format!(
                "window.__veloceSetKeyboardShortcuts?.(\"{}\", {serialized});",
                keyboard_token_for_load
            ));
        }
    });

    let (title_app, title_tab, title_label, title_token) = (
        app.clone(),
        tab_id.to_string(),
        label.to_owned(),
        keyboard_bridge_token.to_owned(),
    );
    let webview_builder = webview_builder.on_document_title_changed(move |_, title| {
        if !super::navigation_state::is_current_generation(&title_app, &title_label, &title_token) {
            return;
        }
        update_navigation(&title_app, &title_tab, &title_label, &title_token, |tab| {
            if !title.trim().is_empty() {
                tab.title = title.chars().take(512).collect();
            }
        });
        if let Some(writer) =
            title_app.try_state::<crate::database::navigation_writer::NavigationWriter>()
        {
            if writer.session().is_err() {
                let _ = title_app.emit("browser-error", "Session persistence queue is unavailable");
            }
        }
    });

    webview_builder
}
