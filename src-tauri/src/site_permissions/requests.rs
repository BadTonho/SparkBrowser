use super::{
    profile::{self, reply, Reply},
    PermissionRequest, PermissionRequests, PermissionState,
};
use crate::state::browser_state::BrowserState;
use crate::sync::Mutex;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};
use tauri::{AppHandle, Manager};
use webview2_com::{
    CoTaskMemPWSTR, Microsoft::Web::WebView2::Win32::*, NavigationStartingEventHandler,
    PermissionRequestedEventHandler, SetPermissionStateCompletedHandler,
};
use windows_core::Interface;
struct Pending {
    args: ICoreWebView2PermissionRequestedEventArgs,
    deferral: ICoreWebView2Deferral,
    profile: ICoreWebView2Profile4,
    request: PermissionRequest,
    label: String,
    token: String,
    busy: bool,
    frame_id: Option<usize>,
}
thread_local! {static PENDING:RefCell<HashMap<String,Pending>>=RefCell::new(HashMap::new());}
fn emit(app: &AppHandle) {
    let pending = app
        .state::<PermissionRequests>()
        .pending
        .lock()
        .map(|v| v.clone())
        .unwrap_or_default();
    crate::error_log::ignored(crate::security::emit_shell(
        app,
        "permission-requests-updated",
        pending,
    ));
}
fn valid(app: &AppHandle, pending: &Pending) -> bool {
    let generation_matches = app
        .state::<BrowserState>()
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .map(|tokens| tokens.get(&pending.label) == Some(&pending.token))
        .unwrap_or(false);
    generation_matches
        && app
            .state::<BrowserState>()
            .session()
            .tabs()
            .lock()
            .map(|tabs| {
                tabs.iter().any(|tab| {
                    tab.id == pending.request.tab_id
                        && super::origin(&tab.url).ok().as_deref()
                            == Some(pending.request.top_origin.as_str())
                })
            })
            .unwrap_or(false)
}
fn finish(app: &AppHandle, id: &str, state: PermissionState, remember: bool) {
    let pending = PENDING.with(|items| items.borrow_mut().remove(id));
    if pending.is_none() {
        return;
    }
    if let Some(pending) = pending {
        let is_valid = valid(app, &pending);
        let state = if is_valid {
            state
        } else {
            PermissionState::Block
        };
        unsafe {
            if let Ok(args) = pending
                .args
                .cast::<ICoreWebView2PermissionRequestedEventArgs3>()
            {
                crate::error_log::ignored(args.SetSavesInProfile(remember && is_valid));
            }
            crate::error_log::ignored(pending.args.SetState(profile::native_state(&state)));
            crate::error_log::ignored(pending.deferral.Complete());
        }
    }
    if let Ok(mut items) = app.state::<PermissionRequests>().pending.lock() {
        items.retain(|item| item.id != id);
    }
    emit(app);
}
pub fn cancel(app: &AppHandle, label: &str) {
    let ids = PENDING.with(|items| {
        items
            .borrow()
            .iter()
            .filter(|(_, item)| item.label == label)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>()
    });
    for id in ids {
        finish(app, &id, PermissionState::Block, false);
    }
}
pub async fn respond(app: &AppHandle, id: String, decision: String) -> Result<(), String> {
    let (state, remember) = match decision.as_str() {
        "allow_once" => (PermissionState::Allow, false),
        "allow_always" => (PermissionState::Allow, true),
        "block" => (PermissionState::Block, false),
        "block_always" => (PermissionState::Block, true),
        _ => return Err("Invalid permission decision".into()),
    };
    let (send, receive) = tokio::sync::oneshot::channel();
    let send: Reply<()> = Arc::new(Mutex::new(Some(send)));
    let app = app.clone();
    let dispatch = app.clone();
    dispatch
        .run_on_main_thread(move || {
            let pending: Result<_, String> = PENDING.with(|items| {
                let mut items = items.borrow_mut();
                let pending = items.get_mut(&id).ok_or("Permission request expired")?;
                if !valid(&app, pending) {
                    return Err("Permission request no longer belongs to this page".into());
                }
                if pending.busy {
                    return Err("Permission response is already pending".into());
                }
                pending.busy = true;
                Ok((pending.profile.clone(), pending.request.clone()))
            });
            let (profile, request) = match pending {
                Ok(value) => value,
                Err(error) => {
                    reply(&send, Err(error));
                    return;
                }
            };
            if !remember {
                finish(&app, &id, state, false);
                reply(&send, Ok(()));
                return;
            }
            let (callback_app, callback_id, callback_send) =
                (app.clone(), id.clone(), send.clone());
            let callback = SetPermissionStateCompletedHandler::create(Box::new(move |status| {
                let result = status.map_err(|e| e.to_string());
                if result.is_ok() {
                    finish(&callback_app, &callback_id, state, true);
                } else {
                    PENDING.with(|items| {
                        if let Some(item) = items.borrow_mut().get_mut(&callback_id) {
                            item.busy = false;
                        }
                    });
                }
                reply(&callback_send, result);
                Ok(())
            }));
            let origin = CoTaskMemPWSTR::from(request.origin.as_str());
            let result = unsafe {
                profile
                    .SetPermissionState(
                        profile::native_kind(&request.kind),
                        *origin.as_ref().as_pcwstr(),
                        profile::native_state(&state),
                        &callback,
                    )
                    .map_err(|e| e.to_string())
            };
            if let Err(error) = result {
                PENDING.with(|items| {
                    if let Some(item) = items.borrow_mut().get_mut(&id) {
                        item.busy = false;
                    }
                });
                reply(&send, Err(error));
            }
        })
        .map_err(|e| e.to_string())?;
    receive.await.map_err(|e| e.to_string())?
}

pub(super) struct RequestSource<'a> {
    pub profile: ICoreWebView2Profile4,
    pub top_origin: String,
    pub label: &'a str,
    pub tab_id: &'a str,
    pub token: &'a str,
    pub frame_id: Option<usize>,
    pub frame_origin: Option<String>,
}
pub(super) fn enqueue(
    app: &AppHandle,
    args: ICoreWebView2PermissionRequestedEventArgs,
    source: RequestSource<'_>,
) -> Result<(), String> {
    let RequestSource {
        profile,
        top_origin,
        label,
        tab_id,
        token,
        frame_id,
        frame_origin,
    } = source;
    let mut native_kind = Default::default();
    unsafe {
        args.PermissionKind(&mut native_kind)
            .map_err(|e| e.to_string())?;
    }
    let Some(kind) = profile::kind(native_kind) else {
        return Ok(());
    };
    let mut current = Default::default();
    unsafe {
        args.State(&mut current).map_err(|e| e.to_string())?;
    }
    if current != COREWEBVIEW2_PERMISSION_STATE_DEFAULT {
        return Ok(());
    }
    let extended = args
        .cast::<ICoreWebView2PermissionRequestedEventArgs3>()
        .map_err(|e| e.to_string())?;
    unsafe {
        extended
            .SetSavesInProfile(false)
            .map_err(|e| e.to_string())?;
    }
    let mut uri = Default::default();
    unsafe {
        args.Uri(&mut uri).map_err(|e| e.to_string())?;
    }
    let origin = super::origin(&CoTaskMemPWSTR::from(uri).to_string())?;
    let registry = app.state::<PermissionRequests>();
    let deferral = unsafe { args.GetDeferral().map_err(|e| e.to_string())? };
    let id = format!(
        "permission-{}",
        registry.next.fetch_add(1, Ordering::Relaxed)
    );
    let request = PermissionRequest {
        id: id.clone(),
        tab_id: tab_id.into(),
        origin,
        top_origin,
        frame_origin,
        kind,
    };
    if !registry.admit(request.clone()) {
        unsafe {
            crate::error_log::ignored(args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY));
            crate::error_log::ignored(deferral.Complete());
        }
        return Ok(());
    }
    PENDING.with(|items| {
        items.borrow_mut().insert(
            id.clone(),
            Pending {
                args,
                deferral,
                profile,
                request,
                label: label.into(),
                token: token.into(),
                busy: false,
                frame_id,
            },
        )
    });
    emit(app);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let dispatch = app.clone();
        let _ =
            dispatch.run_on_main_thread(move || finish(&app, &id, PermissionState::Block, false));
    });
    Ok(())
}
/// All handlers are attached to the blank view before its first external navigation.
pub fn install_and_navigate(
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
        let result = (|| -> Result<(), String> {
            let core = unsafe {
                platform
                    .controller()
                    .CoreWebView2()
                    .map_err(|e| e.to_string())?
            };
            let mut registration = 0;
            let (a, l, t, secret) = (app.clone(), label.clone(), tab_id.clone(), token.clone());
            let handler = PermissionRequestedEventHandler::create(Box::new(move |sender, args| {
                if let (Some(core), Some(args)) = (sender, args) {
                    let mut kind = Default::default();
                    unsafe {
                        args.PermissionKind(&mut kind)?;
                    }
                    if profile::kind(kind).is_none() {
                        return Ok(());
                    }
                    let source = (|| -> Result<RequestSource<'_>, String> {
                        let profile = profile::profile(&core)?;
                        let mut top = Default::default();
                        unsafe {
                            core.Source(&mut top).map_err(|e| e.to_string())?;
                        }
                        let top_origin = super::origin(&CoTaskMemPWSTR::from(top).to_string())?;
                        Ok(RequestSource {
                            profile,
                            top_origin,
                            label: &l,
                            tab_id: &t,
                            token: &secret,
                            frame_id: None,
                            frame_origin: None,
                        })
                    })();
                    if source
                        .and_then(|source| enqueue(&a, args.clone(), source))
                        .is_err()
                    {
                        unsafe {
                            crate::error_log::ignored(
                                args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY),
                            );
                        }
                    }
                }
                Ok(())
            }));
            let (a, l) = (app.clone(), label.clone());
            let navigation = NavigationStartingEventHandler::create(Box::new(move |_, args| {
                let navigating = args.is_some_and(navigating);
                if navigating {
                    cancel(&a, &l);
                }
                Ok(())
            }));
            // Older runtimes still navigate; unsupported managed requests are denied below.
            crate::error_log::ignored(super::frames::install(&app, &core, &label, &tab_id, &token));
            unsafe {
                core.add_PermissionRequested(&handler, &mut registration)
                    .map_err(|e| e.to_string())?;
                core.add_NavigationStarting(&navigation, &mut registration)
                    .map_err(|e| e.to_string())?;
            }
            app.state::<BrowserState>()
                .surfaces()
                .content_bootstrap()
                .finish(&label, &token, |target| {
                    let url = CoTaskMemPWSTR::from(target);
                    unsafe { core.Navigate(*url.as_ref().as_pcwstr()) }.map_err(|e| e.to_string())
                })?;
            Ok(())
        })();
        if result.is_err() {
            crate::error_log::ignored(crate::security::emit_shell(
                &app,
                "browser-error",
                "Could not initialize native page permissions",
            ));
        }
    })
    .map_err(|e| e.to_string())
}

pub(super) fn navigating(args: ICoreWebView2NavigationStartingEventArgs) -> bool {
    unsafe {
        let (mut cancelled, mut uri) = (Default::default(), Default::default());
        if args.Cancel(&mut cancelled).is_err() || args.Uri(&mut uri).is_err() {
            return false;
        }
        let uri = CoTaskMemPWSTR::from(uri).to_string();
        !cancelled.as_bool()
            && url::Url::parse(&uri).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
    }
}
pub(super) fn cancel_frame(app: &AppHandle, label: &str, key: usize) {
    let ids = PENDING.with(|items| {
        items
            .borrow()
            .iter()
            .filter(|(_, item)| item.label == label && item.frame_id == Some(key))
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>()
    });
    for id in ids {
        finish(app, &id, PermissionState::Block, false);
    }
}
