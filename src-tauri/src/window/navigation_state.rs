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
