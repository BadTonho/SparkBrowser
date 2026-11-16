use super::RuntimeService;
use std::cell::RefCell;
use tauri::{AppHandle, Manager, Webview};
use webview2_com::{
    CoTaskMemPWSTR, Microsoft::Web::WebView2::Win32::*, NewBrowserVersionAvailableEventHandler,
};
use windows_core::{Interface, PCWSTR};

thread_local! { static WATCHER: RefCell<Option<(ICoreWebView2Environment, i64)>> = const { RefCell::new(None) }; }

fn versions(environment: &ICoreWebView2Environment) -> windows_core::Result<(String, String)> {
    let (mut current, mut available) = (Default::default(), Default::default());
    unsafe {
        environment.BrowserVersionString(&mut current)?;
        GetAvailableCoreWebView2BrowserVersionString(PCWSTR::null(), &mut available)?;
    }
    Ok((
        CoTaskMemPWSTR::from(current).to_string(),
        CoTaskMemPWSTR::from(available).to_string(),
    ))
}
fn report(app: &AppHandle, environment: &ICoreWebView2Environment) {
    if super::testing::apply(app) {
        return;
    }
    match versions(environment) {
        Ok((current, available)) => {
            app.state::<RuntimeService>()
                .update(app, Some(current), Some(available), None)
        }
        Err(_) => app.state::<RuntimeService>().update(
            app,
            None,
            None,
            Some("runtime.unavailable".into()),
        ),
    }
}
pub(super) fn install(app: &AppHandle, view: &Webview) -> Result<(), String> {
    // A fixed runtime is not managed by Evergreen; do not compare it with an
    // unrelated installed runtime or promise that restarting will update it.
    if std::env::var_os("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER").is_some_and(|v| !v.is_empty()) {
        app.state::<RuntimeService>()
            .update(app, None, None, Some("runtime.unavailable".into()));
        return Ok(());
    }
    let app = app.clone();
    view.with_webview(move |platform| {
        let result = (|| -> windows_core::Result<()> {
            let environment =
                unsafe { platform.controller().CoreWebView2()? }.cast::<ICoreWebView2_2>()?;
            let environment = unsafe { environment.Environment()? };
            let notify = app.clone();
            let callback =
                NewBrowserVersionAvailableEventHandler::create(Box::new(move |sender, _| {
                    if let Some(sender) = sender {
                        report(&notify, &sender);
                    }
                    Ok(())
                }));
            let mut registration = 0;
            unsafe {
                environment.add_NewBrowserVersionAvailable(&callback, &mut registration)?;
            }
            report(&app, &environment);
            WATCHER.with(|watcher| {
                if let Some((previous, token)) =
                    watcher.borrow_mut().replace((environment, registration))
                {
                    unsafe {
                        crate::error_log::ignored(
                            previous.remove_NewBrowserVersionAvailable(token),
                        );
                    }
                }
            });
            Ok(())
        })();
        if result.is_err() {
            app.state::<RuntimeService>().update(
                &app,
                None,
                None,
                Some("runtime.unavailable".into()),
            );
        }
    })
    .map_err(|_| "runtime.unavailable".into())
}
