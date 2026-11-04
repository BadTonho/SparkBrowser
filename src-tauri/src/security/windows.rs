use tauri::{AppHandle, Webview};
use webview2_com::{
    CoTaskMemPWSTR, NavigationStartingEventHandler, NewWindowRequestedEventHandler,
};

pub(super) fn install(app: &AppHandle, view: &Webview, internal: bool) -> Result<(), String> {
    let app = app.clone();
    let shell = view.label() == "main";
    view.with_webview(move |platform| {
        let result = (|| -> windows_core::Result<()> {
            let core = unsafe { platform.controller().CoreWebView2()? };
            unsafe {
                core.Settings()?.SetAreHostObjectsAllowed(false)?;
            }
            let mut registration = 0;
            if internal {
                let deny = NewWindowRequestedEventHandler::create(Box::new(|_, args| {
                    if let Some(args) = args {
                        unsafe {
                            args.SetHandled(true)?;
                        }
                    }
                    Ok(())
                }));
                unsafe {
                    core.add_NewWindowRequested(&deny, &mut registration)?;
                }
            }
            if shell {
                let policy_app = app.clone();
                let guard = NavigationStartingEventHandler::create(Box::new(move |_, args| {
                    if let Some(args) = args {
                        let mut uri = Default::default();
                        unsafe {
                            args.Uri(&mut uri)?;
                        }
                        let allowed = tauri::Url::parse(&CoTaskMemPWSTR::from(uri).to_string())
                            .is_ok_and(|url| super::documents::is_shell(&policy_app, &url));
                        if !allowed {
                            unsafe {
                                args.SetCancel(true)?;
                            }
                        }
                    }
                    Ok(())
                }));
                unsafe {
                    core.add_NavigationStarting(&guard, &mut registration)?;
                }
            }
            Ok(())
        })();
        if result.is_err() {
            // Fail closed rather than continue with partially installed boundaries.
            let _ = super::emit_shell(&app, "browser-error", "security.initialization_failed");
            app.exit(1);
        }
    })
    .map_err(|_| "security.initialization_failed".into())
}
