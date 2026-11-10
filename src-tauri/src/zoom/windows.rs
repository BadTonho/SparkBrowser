use super::ZoomService;
use crate::{database::site_zoom_repo as repo, state::browser_state::BrowserState};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use tauri::{AppHandle, Manager};
use webview2_com::{
    CoTaskMemPWSTR, NavigationCompletedEventHandler, NavigationStartingEventHandler,
    SourceChangedEventHandler, ZoomFactorChangedEventHandler,
};

fn current(app: &AppHandle, label: &str, token: &str) -> bool {
    app.state::<BrowserState>()
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .is_ok_and(|m| m.get(label).is_some_and(|t| t == token))
}
pub fn apply(view: &tauri::Webview, url: &str, factor: f64) -> Result<(), String> {
    let expected = repo::origin(url).ok();
    view.with_webview(move |platform| unsafe {
        let controller = platform.controller();
        if let Ok(core) = controller.CoreWebView2() {
            let mut source = Default::default();
            if core.Source(&mut source).is_ok()
                && repo::origin(&CoTaskMemPWSTR::from(source).to_string()).ok() == expected
            {
                let _ = controller.SetZoomFactor(factor);
            }
        }
    })
    .map_err(|e| e.to_string())
}
/// All COM calls stay on the UI thread. Navigation events never persist engine defaults.
pub fn install(
    app: &AppHandle,
    view: &tauri::Webview,
    tab_id: &str,
    token: &str,
) -> Result<(), String> {
    let (app, label, tab_id, token) = (
        app.clone(),
        view.label().to_owned(),
        tab_id.to_owned(),
        token.to_owned(),
    );
    view.with_webview(move |platform| {
        let result = (|| -> webview2_com::Result<()> {
            unsafe {
                let controller = platform.controller();
                let core = controller.CoreWebView2()?;
                let ready = Arc::new(AtomicBool::new(false));
                let navigation = Arc::new(AtomicU64::new(0));
                let (r, n) = (ready.clone(), navigation.clone());
                core.add_NavigationStarting(
                    &NavigationStartingEventHandler::create(Box::new(move |_, args| {
                        r.store(false, Ordering::Release);
                        if let Some(args) = args {
                            let mut id = 0;
                            args.NavigationId(&mut id)?;
                            n.store(id, Ordering::Release);
                        }
                        Ok(())
                    })),
                    &mut 0,
                )?;
                let (a, l, t) = (app.clone(), label.clone(), token.clone());
                core.add_SourceChanged(
                    &SourceChangedEventHandler::create(Box::new(move |sender, _| {
                        if current(&a, &l, &t) {
                            if let Some(core) = sender {
                                let mut source = Default::default();
                                core.Source(&mut source)?;
                                let url = CoTaskMemPWSTR::from(source).to_string();
                                let _ = super::apply(&a, &l, &url);
                            }
                        }
                        Ok(())
                    })),
                    &mut 0,
                )?;
                let (a, l, t, r, n) = (
                    app.clone(),
                    label.clone(),
                    token.clone(),
                    ready.clone(),
                    navigation.clone(),
                );
                core.add_NavigationCompleted(
                    &NavigationCompletedEventHandler::create(Box::new(move |sender, args| {
                        if current(&a, &l, &t) {
                            if let (Some(core), Some(args)) = (sender, args) {
                                let mut id = 0;
                                args.NavigationId(&mut id)?;
                                if id == n.load(Ordering::Acquire) {
                                    let mut source = Default::default();
                                    core.Source(&mut source)?;
                                    let url = CoTaskMemPWSTR::from(source).to_string();
                                    let _ = super::apply(&a, &l, &url);
                                    r.store(true, Ordering::Release);
                                }
                            }
                        }
                        Ok(())
                    })),
                    &mut 0,
                )?;
                let (a, l, t, id) = (app.clone(), label.clone(), token.clone(), tab_id);
                controller.add_ZoomFactorChanged(
                    &ZoomFactorChangedEventHandler::create(Box::new(move |sender, _| {
                        if !ready.load(Ordering::Acquire) || !current(&a, &l, &t) {
                            return Ok(());
                        }
                        if let Some(controller) = sender {
                            let mut factor = 1.0;
                            controller.ZoomFactor(&mut factor)?;
                            let core = controller.CoreWebView2()?;
                            let mut source = Default::default();
                            core.Source(&mut source)?;
                            let url = CoTaskMemPWSTR::from(source).to_string();
                            let Ok(origin) = repo::origin(&url) else {
                                return Ok(());
                            };
                            if !(0.25..=5.0).contains(&factor) {
                                return Ok(());
                            }
                            let service = a.state::<ZoomService>();
                            if (service.factor(&url) - factor).abs() < 0.0001 {
                                return Ok(());
                            }
                            let revision = service.snapshot().map(|s| s.revision).unwrap_or(0);
                            let generation = navigation.load(Ordering::Acquire);
                            let (a, l, t, id, n) = (
                                a.clone(),
                                l.clone(),
                                t.clone(),
                                id.clone(),
                                navigation.clone(),
                            );
                            tauri::async_runtime::spawn(async move {
                                let browser = a.state::<BrowserState>();
                                let _operation = browser.session().operation().await;
                                if !current(&a, &l, &t)
                                    || generation != n.load(Ordering::Acquire)
                                    || a.state::<crate::shutdown::ShutdownService>().blocked()
                                {
                                    return;
                                }
                                let service = a.state::<ZoomService>();
                                if service.snapshot().is_ok_and(|s| s.revision == revision)
                                    && browser.snapshot_tabs().iter().any(|tab| {
                                        tab.id == id
                                            && repo::origin(&tab.url).ok().as_ref() == Some(&origin)
                                    })
                                {
                                    let _ = service.set(&a, repo::SiteZoom { origin, factor });
                                }
                            });
                        }
                        Ok(())
                    })),
                    &mut 0,
                )?;
                Ok(())
            }
        })();
        if let Err(e) = result {
            let _ = crate::security::emit_shell(
                &app,
                "browser-error",
                format!("zoom.unavailable: {e}"),
            );
        }
    })
    .map_err(|e| e.to_string())
}
