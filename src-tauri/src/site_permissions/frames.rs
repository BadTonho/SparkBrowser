use std::{cell::RefCell, rc::Rc};
use tauri::{AppHandle, Manager};
use webview2_com::{
    FrameChildFrameCreatedEventHandler, FrameCreatedEventHandler, FrameDestroyedEventHandler,
    FrameNavigationStartingEventHandler, FramePermissionRequestedEventHandler,
    Microsoft::Web::WebView2::Win32::*,
};
use windows_core::Interface;
#[derive(Clone)]
struct Context {
    app: AppHandle,
    profile: ICoreWebView2Profile4,
    label: String,
    tab_id: String,
    token: String,
}
fn attach(context: Context, frame: ICoreWebView2Frame) -> Result<(), String> {
    let key = frame.as_raw() as usize;
    let frame3 = frame
        .cast::<ICoreWebView2Frame3>()
        .map_err(|e| e.to_string())?;
    let mut registration = 0;
    let frame_origin = Rc::new(RefCell::new(None));
    let permission_origin = frame_origin.clone();
    let c = context.clone();
    let permission = FramePermissionRequestedEventHandler::create(Box::new(move |_, args| {
        if let Some(args) = args {
            unsafe {
                let mut kind = Default::default();
                args.PermissionKind(&mut kind)?;
                if super::profile::kind(kind).is_none() {
                    return Ok(());
                }
                args.SetHandled(true)?;
                if let Ok(base) = args.cast::<ICoreWebView2PermissionRequestedEventArgs>() {
                    let top_origin = c
                        .app
                        .state::<crate::state::browser_state::BrowserState>()
                        .session()
                        .tabs()
                        .lock()
                        .ok()
                        .and_then(|tabs| {
                            tabs.iter()
                                .find(|tab| tab.id == c.tab_id)
                                .and_then(|tab| super::origin(&tab.url).ok())
                        });
                    let result = top_origin
                        .ok_or("Missing containing page".to_string())
                        .and_then(|top_origin| {
                            super::requests::enqueue(
                                &c.app,
                                base,
                                super::requests::RequestSource {
                                    profile: c.profile.clone(),
                                    top_origin,
                                    label: &c.label,
                                    tab_id: &c.tab_id,
                                    token: &c.token,
                                    frame_id: Some(key),
                                    frame_origin: permission_origin.borrow().clone(),
                                },
                            )
                        });
                    if result.is_err() {
                        args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY)?;
                    }
                } else {
                    args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY)?;
                }
            }
        }
        Ok(())
    }));
    let c = context.clone();
    let destroyed = FrameDestroyedEventHandler::create(Box::new(move |_, _| {
        super::requests::cancel_frame(&c.app, &c.label, key);
        Ok(())
    }));
    let c = context.clone();
    let navigation = FrameNavigationStartingEventHandler::create(Box::new(move |_, args| {
        if let Some(args) = args {
            if super::requests::navigating(args.clone()) {
                super::requests::cancel_frame(&c.app, &c.label, key);
                let mut uri = Default::default();
                if unsafe { args.Uri(&mut uri) }.is_ok() {
                    *frame_origin.borrow_mut() =
                        super::origin(&webview2_com::CoTaskMemPWSTR::from(uri).to_string()).ok();
                }
            }
        }
        Ok(())
    }));
    unsafe {
        frame3
            .add_PermissionRequested(&permission, &mut registration)
            .map_err(|e| e.to_string())?;
        frame
            .add_Destroyed(&destroyed, &mut registration)
            .map_err(|e| e.to_string())?;
        frame3
            .add_NavigationStarting(&navigation, &mut registration)
            .map_err(|e| e.to_string())?;
        if let Ok(nested) = frame.cast::<ICoreWebView2Frame7>() {
            let child = FrameChildFrameCreatedEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    if let Ok(frame) = args.Frame() {
                        let _ = attach(context.clone(), frame);
                    }
                }
                Ok(())
            }));
            nested
                .add_FrameCreated(&child, &mut registration)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
pub fn install(
    app: &AppHandle,
    core: &ICoreWebView2,
    label: &str,
    tab_id: &str,
    token: &str,
) -> Result<(), String> {
    let context = Context {
        app: app.clone(),
        profile: super::profile::profile(core)?,
        label: label.into(),
        tab_id: tab_id.into(),
        token: token.into(),
    };
    let frames = core.cast::<ICoreWebView2_4>().map_err(|e| e.to_string())?;
    let handler = FrameCreatedEventHandler::create(Box::new(move |_, args| {
        if let Some(args) = args {
            if let Ok(frame) = unsafe { args.Frame() } {
                let _ = attach(context.clone(), frame);
            }
        }
        Ok(())
    }));
    let mut registration = 0;
    unsafe {
        frames
            .add_FrameCreated(&handler, &mut registration)
            .map_err(|e| e.to_string())
    }
}
