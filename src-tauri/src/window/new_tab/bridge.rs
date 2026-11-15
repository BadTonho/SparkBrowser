use super::*;
use crate::database::{executor, navigation_writer::NavigationWriter};
use std::collections::HashMap;

pub(super) fn valid(app: &AppHandle, id: &str, label: &str, token: &str) -> bool {
    super::super::navigation_state::is_current_generation(app, label, token)
        && app
            .state::<BrowserState>()
            .snapshot_tabs()
            .iter()
            .any(|t| t.id == id && is_new_tab(&t.url))
        && app
            .get_webview(label)
            .and_then(|v| v.url().ok())
            .is_some_and(|url| is_document(app, &url))
}

pub fn handle(app: &AppHandle, id: &str, label: &str, token: &str, url: &Url) {
    let Some(values) = crate::security::bridge::query(
        url,
        &[
            "token",
            "sequence",
            "document",
            "request",
            "input",
            "site_key",
            "operation",
            "data",
        ],
    ) else {
        return;
    };
    if !super::message::valid(url.path(), &values) {
        return;
    }
    if values.get("token").map(String::as_str) != Some(token) || !valid(app, id, label, token) {
        return;
    }
    if !matches!(url.path(), "/read" | "/navigate" | "/visit" | "/widgets")
        || (url.path() == "/visit" && !values.get("site_key").is_some_and(|key| !key.is_empty()))
    {
        return;
    }
    let Some(document) = values
        .get("document")
        .filter(|v| !v.is_empty() && v.len() <= 64)
        .cloned()
    else {
        return;
    };
    let Some(sequence) = values
        .get("sequence")
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
    else {
        return;
    };
    {
        let service = app.state::<NewTabService>();
        let Ok(mut sessions) = service.sessions.lock() else {
            return;
        };
        if let Some(session) = sessions.get_mut(label) {
            if !session.accept(
                &document,
                sequence,
                matches!(url.path(), "/navigate" | "/visit")
                    || values.get("operation").is_some_and(|op| {
                        op == "open_gmail" || op == "connect" || op == "spotify_open"
                    }),
            ) {
                return;
            }
            if url.path() == "/read" {
                session.last_read = sequence;
            }
        } else if url.path() == "/read" {
            sessions.insert(
                label.into(),
                Session {
                    document: document.clone(),
                    sequence,
                    last_read: sequence,
                    navigation: 0,
                    presented: false,
                },
            );
        } else {
            return;
        }
    }
    if url.path() == "/widgets" {
        widgets::handle(app, id, label, token, &values);
        return;
    }
    let Some(request) = values.get("request").and_then(|v| v.parse::<u64>().ok()) else {
        if matches!(url.path(), "/navigate" | "/visit") {
            navigate(app, id, label, token, &values, url.path() == "/visit");
        }
        return;
    };
    if url.path() != "/read" {
        return;
    }
    let (app, id, label, token) = (
        app.clone(),
        id.to_owned(),
        label.to_owned(),
        token.to_owned(),
    );
    tauri::async_runtime::spawn(async move {
        let revision = app.state::<NewTabService>().revision.load(Ordering::SeqCst);
        let epoch = super::super::favicon::epoch(&app);
        let worker_app = app.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            worker_app.state::<NavigationWriter>().flush()
        })
        .await;
        let cache_app = app.clone();
        let rows = match result {
            Ok(Ok(())) => {
                executor::run(app.clone(), move |conn| {
                    let service = cache_app.state::<NewTabService>();
                    let mut cache = service.recent_cache.lock().map_err(|e| e.to_string())?;
                    cache.list(conn, revision)
                })
                .await
            }
            _ => Err("History unavailable".into()),
        };
        if !valid(&app, &id, &label, &token)
            || super::super::favicon::epoch(&app) != epoch
            || app.state::<NewTabService>().revision.load(Ordering::SeqCst) != revision
        {
            return;
        }
        let prefs = app
            .state::<NewTabService>()
            .preferences
            .lock()
            .map(|p| p.clone())
            .unwrap_or_default();
        let text: serde_json::Value = serde_json::from_str(if prefs.language == "pt-BR" {
            include_str!("../../../../src/lib/i18n/catalogs/pt-BR/newtab.json")
        } else {
            include_str!("../../../../src/lib/i18n/catalogs/en/newtab.json")
        })
        .expect("new tab translations");
        let first = {
            let service = app.state::<NewTabService>();
            let Ok(mut sessions) = service.sessions.lock() else {
                return;
            };
            let Some(session) = sessions
                .get_mut(&label)
                .filter(|s| s.document == document && s.last_read == sequence)
            else {
                return;
            };
            let first = !session.presented;
            session.presented = true;
            first
        };
        let autofocus = first
            && app.state::<BrowserState>().is_active_tab(&id)
            && !app
                .state::<BrowserState>()
                .surfaces()
                .is_overlay_open()
                .lock()
                .map_or(true, |v| *v);
        let private = app
            .try_state::<crate::private_session::PrivateRole>()
            .is_some_and(|role| role.is_private());
        let payload = serde_json::json!({"preferences":prefs,"labels":text,"sites":rows.as_ref().ok(),"error":rows.is_err(),"autofocus":autofocus,"private":private});
        if let Some(view) = app.get_webview(&label) {
            if autofocus {
                let _ = view.set_focus();
            }
            let _ = view.eval(format!(
                "window.__sparkNewTabReply?.({}, {}, {});",
                serde_json::to_string(&document).unwrap(),
                request,
                payload
            ));
        }
    });
}

pub(super) fn navigate(
    app: &AppHandle,
    id: &str,
    label: &str,
    token: &str,
    values: &HashMap<String, String>,
    visit: bool,
) {
    let Some(input) = values
        .get("input")
        .filter(|s| !s.trim().is_empty() && s.len() <= 8192)
        .cloned()
    else {
        return;
    };
    let visit = if visit {
        values.get("site_key").cloned()
    } else {
        None
    };
    let epoch = super::super::favicon::epoch(app);
    let Some(document) = values.get("document").cloned() else {
        return;
    };
    let sequence = values
        .get("sequence")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let (app, id, label, token) = (
        app.clone(),
        id.to_owned(),
        label.to_owned(),
        token.to_owned(),
    );
    tauri::async_runtime::spawn(async move {
        let state = app.state::<BrowserState>();
        let _operation = state.session().operation().await;
        if !valid(&app, &id, &label, &token)
            || !app.state::<NewTabService>().sessions.lock().is_ok_and(|s| {
                s.get(&label)
                    .is_some_and(|s| s.document == document && s.navigation == sequence)
            })
            || !state
                .snapshot_tabs()
                .iter()
                .any(|tab| tab.id == id && tab.is_visible)
            || state
                .surfaces()
                .is_overlay_open()
                .lock()
                .map_or(true, |v| *v)
            || app.state::<crate::shutdown::ShutdownService>().blocked()
        {
            return;
        }
        let engine = {
            app.state::<NewTabService>()
                .preferences
                .lock()
                .map(|p| p.engine.clone())
                .unwrap_or_else(|_| "google".into())
        };
        let target = if let Some(key) = visit {
            let cache_app = app.clone();
            let read = crate::database::executor::run(app.clone(), move |conn| {
                let service = cache_app.state::<NewTabService>();
                let revision = service.revision.load(Ordering::SeqCst);
                let mut cache = service.recent_cache.lock().map_err(|e| e.to_string())?;
                cache.list(conn, revision).map(|sites| {
                    sites
                        .into_iter()
                        .find(|s| s.site_key == key && s.url == input)
                })
            })
            .await;
            match read {
                Ok(Some(site)) if super::super::favicon::epoch(&app) == epoch => {
                    Url::parse(&site.url).ok()
                }
                _ => None,
            }
        } else {
            super::super::url_input::resolve_browser_input(&app, &input, &engine).ok()
        };
        let Some(target) = target else {
            return;
        };
        let result = (|| {
            if !valid(&app, &id, &label, &token) || super::super::favicon::epoch(&app) != epoch {
                return Ok(());
            }
            if !app
                .state::<NewTabService>()
                .sessions
                .lock()
                .is_ok_and(|sessions| {
                    sessions.get(&label).is_some_and(|session| {
                        session.document == document && session.navigation == sequence
                    })
                })
                || state
                    .surfaces()
                    .is_overlay_open()
                    .lock()
                    .map_or(true, |v| *v)
                || app.state::<crate::shutdown::ShutdownService>().blocked()
            {
                return Ok(());
            }
            if crate::privacy::adblock_engine::site_is_protected(&state, target.as_str())
                && crate::privacy::adblock_engine::should_block_url(
                    target.as_str(),
                    *state
                        .policies()
                        .shield_enabled()
                        .lock()
                        .map_err(|e| e.to_string())?,
                )
            {
                return Err("Navigation blocked".to_string());
            }
            // Select the originating pane under the same operation lock before navigation.
            state.select_tab_committed(&id)?;
            super::super::webview_manager::navigate_tab_document(
                &app,
                &id,
                physical_url(&app, target.as_str())?,
            )?;
            crate::state::publish::publish_tabs(
                &app,
                &state,
                crate::state::tab_model::MAIN_WINDOW_ID,
            );
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            let _ = crate::security::emit_shell(&app, "browser-error", error);
        }
    });
}
