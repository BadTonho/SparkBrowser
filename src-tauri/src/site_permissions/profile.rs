use super::{
    PermissionKind, PermissionRequests, PermissionSnapshot, PermissionState, SitePermission,
};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use webview2_com::{
    CoTaskMemPWSTR, GetNonDefaultPermissionSettingsCompletedHandler,
    Microsoft::Web::WebView2::Win32::*, SetPermissionStateCompletedHandler,
};
use windows_core::Interface;
pub type Reply<T> = Arc<Mutex<Option<tokio::sync::oneshot::Sender<Result<T, String>>>>>;
pub fn reply<T>(sender: &Reply<T>, result: Result<T, String>) {
    if let Ok(mut sender) = sender.lock() {
        if let Some(sender) = sender.take() {
            let _ = sender.send(result);
        }
    }
}
pub fn profile(core: &ICoreWebView2) -> Result<ICoreWebView2Profile4, String> {
    unsafe {
        core.cast::<ICoreWebView2_13>()
            .and_then(|core| core.Profile())
            .and_then(|profile| profile.cast())
            .map_err(|e| e.to_string())
    }
}
pub fn native_kind(kind: &PermissionKind) -> COREWEBVIEW2_PERMISSION_KIND {
    match kind {
        PermissionKind::Camera => COREWEBVIEW2_PERMISSION_KIND_CAMERA,
        PermissionKind::Microphone => COREWEBVIEW2_PERMISSION_KIND_MICROPHONE,
        PermissionKind::Location => COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION,
        PermissionKind::Notifications => COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS,
    }
}
pub fn kind(kind: COREWEBVIEW2_PERMISSION_KIND) -> Option<PermissionKind> {
    match kind {
        COREWEBVIEW2_PERMISSION_KIND_CAMERA => Some(PermissionKind::Camera),
        COREWEBVIEW2_PERMISSION_KIND_MICROPHONE => Some(PermissionKind::Microphone),
        COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION => Some(PermissionKind::Location),
        COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS => Some(PermissionKind::Notifications),
        _ => None,
    }
}
pub fn native_state(state: &PermissionState) -> COREWEBVIEW2_PERMISSION_STATE {
    match state {
        PermissionState::Ask => COREWEBVIEW2_PERMISSION_STATE_DEFAULT,
        PermissionState::Allow => COREWEBVIEW2_PERMISSION_STATE_ALLOW,
        PermissionState::Block => COREWEBVIEW2_PERMISSION_STATE_DENY,
    }
}
async fn call<T: Send + 'static>(
    app: &AppHandle,
    operation: impl FnOnce(ICoreWebView2Profile4, Reply<T>) -> Result<(), String> + Send + 'static,
) -> Result<T, String> {
    let view =
        crate::window::webview_manager::active_content_webview(app).ok_or("No active page")?;
    let (send, receive) = tokio::sync::oneshot::channel();
    let send = Arc::new(Mutex::new(Some(send)));
    view.with_webview(move |platform| {
        let result = unsafe { platform.controller().CoreWebView2() }
            .map_err(|e| e.to_string())
            .and_then(|core| profile(&core))
            .and_then(|profile| operation(profile, send.clone()));
        if let Err(error) = result {
            reply(&send, Err(error));
        }
    })
    .map_err(|e| e.to_string())?;
    receive.await.map_err(|e| e.to_string())?
}
pub async fn snapshot(app: &AppHandle) -> PermissionSnapshot {
    let settings = call(app, |profile, send| {
        let callback = GetNonDefaultPermissionSettingsCompletedHandler::create(Box::new(
            move |status, collection| {
                let result = (|| -> Result<Vec<SitePermission>, String> {
                    status.map_err(|e| e.to_string())?;
                    let collection = collection.ok_or("Permission settings unavailable")?;
                    let mut count = 0;
                    unsafe {
                        collection.Count(&mut count).map_err(|e| e.to_string())?;
                    }
                    let mut rows = Vec::new();
                    for index in 0..count {
                        unsafe {
                            let item = collection
                                .GetValueAtIndex(index)
                                .map_err(|e| e.to_string())?;
                            let (mut native_kind, mut native_state, mut uri) =
                                (Default::default(), Default::default(), Default::default());
                            item.PermissionKind(&mut native_kind)
                                .map_err(|e| e.to_string())?;
                            item.PermissionState(&mut native_state)
                                .map_err(|e| e.to_string())?;
                            item.PermissionOrigin(&mut uri).map_err(|e| e.to_string())?;
                            let raw_origin = CoTaskMemPWSTR::from(uri).to_string();
                            if let (Some(kind), Ok(origin)) =
                                (kind(native_kind), super::origin(&raw_origin))
                            {
                                rows.push(SitePermission {
                                    origin,
                                    kind,
                                    state: match native_state {
                                        COREWEBVIEW2_PERMISSION_STATE_ALLOW => {
                                            PermissionState::Allow
                                        }
                                        COREWEBVIEW2_PERMISSION_STATE_DENY => {
                                            PermissionState::Block
                                        }
                                        _ => PermissionState::Ask,
                                    },
                                });
                            }
                        }
                    }
                    Ok(rows)
                })();
                reply(&send, result);
                Ok(())
            },
        ));
        unsafe {
            profile
                .GetNonDefaultPermissionSettings(&callback)
                .map_err(|e| e.to_string())
        }
    })
    .await;
    match settings {
        Ok(settings) => PermissionSnapshot {
            supported: true,
            message: String::new(),
            settings,
            requests: app
                .state::<PermissionRequests>()
                .pending
                .lock()
                .map(|v| v.clone())
                .unwrap_or_default(),
        },
        Err(_) => PermissionSnapshot {
            supported: false,
            message: "This WebView2 runtime does not expose site permission management.".into(),
            settings: vec![],
            requests: vec![],
        },
    }
}
pub async fn set(
    app: &AppHandle,
    origin: String,
    kind: PermissionKind,
    state: PermissionState,
) -> Result<(), String> {
    let origin = super::origin(&origin)?;
    call(app, move |profile, send| {
        let callback = SetPermissionStateCompletedHandler::create(Box::new(move |status| {
            reply(&send, status.map_err(|e| e.to_string()));
            Ok(())
        }));
        let origin = CoTaskMemPWSTR::from(origin.as_str());
        unsafe {
            profile
                .SetPermissionState(
                    native_kind(&kind),
                    *origin.as_ref().as_pcwstr(),
                    native_state(&state),
                    &callback,
                )
                .map_err(|e| e.to_string())
        }
    })
    .await
}
