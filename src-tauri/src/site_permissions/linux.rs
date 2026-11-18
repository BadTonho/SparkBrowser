//! Site permission prompts on WebKitGTK. The engine keeps no choices of its own, so saved choices
//! live in the profile database and every other request waits for the user in the shell.
use super::{
    PermissionKind, PermissionRequest, PermissionRequests, PermissionSnapshot, PermissionState,
};
use crate::{database::site_permissions_repo as repo, state::browser_state::BrowserState};
use gtk::glib::prelude::*;
use std::{cell::RefCell, collections::HashMap, sync::atomic::Ordering, time::Duration};
use tauri::{AppHandle, Manager};
use webkit2gtk::{
    GeolocationPermissionRequest, LoadEvent, NotificationPermissionRequest,
    PermissionRequest as WebKitRequest, PermissionRequestExt, UserMediaPermissionRequest,
    UserMediaPermissionRequestExt, WebViewExt,
};

struct Pending {
    request: WebKitRequest,
    meta: PermissionRequest,
    /// Everything the page asked for; one answer covers all of it.
    kinds: Vec<PermissionKind>,
    label: String,
    token: String,
    /// False until the saved choices were read and the user was asked.
    asked: bool,
}

thread_local! {
    static PENDING: RefCell<HashMap<String, Pending>> = RefCell::new(HashMap::new());
}

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

/// Runs `work` on the main thread, where the WebKit requests live, and returns its result.
async fn on_main<T: Send + 'static>(
    app: &AppHandle,
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    let (send, receive) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        crate::error_log::ignored(send.send(work()));
    })
    .map_err(|e| e.to_string())?;
    receive.await.map_err(|e| e.to_string())
}

fn valid(app: &AppHandle, pending: &Pending) -> bool {
    let current = app
        .state::<BrowserState>()
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .map(|tokens| tokens.get(&pending.label) == Some(&pending.token))
        .unwrap_or(false);
    current
        && app
            .state::<BrowserState>()
            .session()
            .tabs()
            .lock()
            .map(|tabs| {
                tabs.iter().any(|tab| {
                    tab.id == pending.meta.tab_id
                        && super::origin(&tab.url).ok().as_deref()
                            == Some(pending.meta.top_origin.as_str())
                })
            })
            .unwrap_or(false)
}

fn finish(app: &AppHandle, id: &str, state: PermissionState) {
    let Some(pending) = PENDING.with(|items| items.borrow_mut().remove(id)) else {
        return;
    };
    if state == PermissionState::Allow && valid(app, &pending) {
        pending.request.allow();
    } else {
        pending.request.deny();
    }
    if let Ok(mut items) = app.state::<PermissionRequests>().pending.lock() {
        items.retain(|item| item.id != id);
    }
    emit(app);
}

/// Denies what the page asked for on a document that is going away.
pub fn cancel(app: &AppHandle, label: &str) {
    let ids: Vec<String> = PENDING.with(|items| {
        items
            .borrow()
            .iter()
            .filter(|(_, item)| item.label == label)
            .map(|(id, _)| id.clone())
            .collect()
    });
    for id in ids {
        finish(app, &id, PermissionState::Block);
    }
}

fn kinds_of(request: &WebKitRequest) -> Option<Vec<PermissionKind>> {
    if let Some(media) = request.downcast_ref::<UserMediaPermissionRequest>() {
        let mut kinds = Vec::new();
        if media.is_for_video_device() {
            kinds.push(PermissionKind::Camera);
        }
        if media.is_for_audio_device() {
            kinds.push(PermissionKind::Microphone);
        }
        // Screen sharing has no setting here and stays denied.
        return (!kinds.is_empty()).then_some(kinds);
    }
    if request.is::<GeolocationPermissionRequest>() {
        return Some(vec![PermissionKind::Location]);
    }
    request
        .is::<NotificationPermissionRequest>()
        .then(|| vec![PermissionKind::Notifications])
}

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
        let web_view = platform.inner();
        let (a, l) = (app.clone(), label.clone());
        web_view.connect_load_changed(move |_, event| {
            if event == LoadEvent::Started {
                cancel(&a, &l);
            }
        });
        web_view.connect_permission_request(move |view, request| {
            let Some(kinds) = kinds_of(request) else {
                request.deny();
                return true;
            };
            let Some(top_origin) = view.uri().and_then(|uri| super::origin(uri.as_str()).ok())
            else {
                request.deny();
                return true;
            };
            let registry = app.state::<PermissionRequests>();
            let id = format!(
                "permission-{}",
                registry.next.fetch_add(1, Ordering::Relaxed)
            );
            let meta = PermissionRequest {
                id: id.clone(),
                tab_id: tab_id.clone(),
                origin: top_origin.clone(),
                top_origin,
                frame_origin: None,
                kind: kinds[0],
            };
            PENDING.with(|items| {
                items.borrow_mut().insert(
                    id.clone(),
                    Pending {
                        request: request.clone(),
                        meta,
                        kinds,
                        label: label.clone(),
                        token: token.clone(),
                        asked: false,
                    },
                )
            });
            let app = app.clone();
            tauri::async_runtime::spawn(async move { resolve(app, id).await });
            true
        });
    })
    .map_err(|e| e.to_string())
}

/// Applies the saved choices, or asks the user when there are none.
async fn resolve(app: AppHandle, id: String) {
    let wanted = {
        let id = id.clone();
        on_main(&app, move || {
            PENDING.with(|items| {
                items
                    .borrow()
                    .get(&id)
                    .map(|p| (p.meta.origin.clone(), p.kinds.clone()))
            })
        })
        .await
    };
    let Ok(Some((origin, kinds))) = wanted else {
        return;
    };
    let saved = crate::database::executor::run(app.clone(), move |conn| {
        kinds
            .iter()
            .map(|kind| repo::get(conn, &origin, *kind).map(|state| (*kind, state)))
            .collect::<Result<Vec<_>, _>>()
    })
    .await
    .unwrap_or_default();
    let handle = app.clone();
    crate::error_log::ignored(
        on_main(&app, move || {
            let app = handle;
            if saved
                .iter()
                .any(|(_, state)| *state == PermissionState::Block)
            {
                return finish(&app, &id, PermissionState::Block);
            }
            if !saved.is_empty()
                && saved
                    .iter()
                    .all(|(_, state)| *state == PermissionState::Allow)
            {
                return finish(&app, &id, PermissionState::Allow);
            }
            let admitted = PENDING.with(|items| {
                let mut items = items.borrow_mut();
                let pending = items.get_mut(&id)?;
                pending.asked = true;
                // Ask about the first thing without a saved answer.
                if let Some((kind, _)) = saved.iter().find(|(_, s)| *s == PermissionState::Ask) {
                    pending.meta.kind = *kind;
                }
                Some(
                    app.state::<PermissionRequests>()
                        .admit(pending.meta.clone()),
                )
            });
            match admitted {
                Some(true) => {
                    emit(&app);
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(60)).await;
                        let dispatch = app.clone();
                        crate::error_log::ignored(
                            dispatch.run_on_main_thread(move || {
                                finish(&app, &id, PermissionState::Block)
                            }),
                        );
                    });
                }
                Some(false) => finish(&app, &id, PermissionState::Block),
                None => {}
            }
        })
        .await,
    );
}

pub async fn respond(app: &AppHandle, id: String, decision: String) -> Result<(), String> {
    let (state, remember) = match decision.as_str() {
        "allow_once" => (PermissionState::Allow, false),
        "allow_always" => (PermissionState::Allow, true),
        "block" => (PermissionState::Block, false),
        "block_always" => (PermissionState::Block, true),
        _ => return Err("Invalid permission decision".into()),
    };
    let lookup = id.clone();
    let handle = app.clone();
    let target = on_main(app, move || {
        PENDING.with(|items| {
            let items = items.borrow();
            let pending = items.get(&lookup).filter(|p| p.asked)?;
            valid(&handle, pending).then(|| (pending.meta.origin.clone(), pending.kinds.clone()))
        })
    })
    .await?
    .ok_or("Permission request expired")?;
    if remember {
        let (origin, kinds) = target;
        crate::database::executor::run(app.clone(), move |conn| {
            kinds
                .iter()
                .try_for_each(|kind| repo::set(conn, &origin, *kind, state))
        })
        .await?;
    }
    let handle = app.clone();
    on_main(app, move || finish(&handle, &id, state)).await
}

pub async fn snapshot(app: &AppHandle) -> PermissionSnapshot {
    let settings = crate::database::executor::run(app.clone(), repo::list)
        .await
        .unwrap_or_default();
    let requests = app
        .state::<PermissionRequests>()
        .pending
        .lock()
        .map(|items| items.clone())
        .unwrap_or_default();
    PermissionSnapshot {
        supported: true,
        message: String::new(),
        settings,
        requests,
    }
}

pub async fn set(
    app: &AppHandle,
    origin: String,
    kind: PermissionKind,
    state: PermissionState,
) -> Result<(), String> {
    let origin = super::origin(&origin)?;
    crate::database::executor::run(app.clone(), move |conn| {
        repo::set(conn, &origin, kind, state)
    })
    .await
}
