use super::*;
use crate::{
    database::{executor, navigation_writer::NavigationWriter},
    state::{tab_checkpoint::TabCheckpoint, tab_model::webview_label_for_tab},
};
use std::collections::HashMap;
use tauri::Emitter;

fn valid(app: &AppHandle, id: &str, label: &str, token: &str) -> bool {
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
    let values: HashMap<_, _> = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    if values.get("token").map(String::as_str) != Some(token) || !valid(app, id, label, token) {
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
            if !session.accept(&document, sequence, url.path() == "/navigate") {
                return;
            }
        } else if url.path() == "/read" {
            sessions.insert(
                label.into(),
                Session {
                    document: document.clone(),
                    sequence,
                    navigation: 0,
                    presented: false,
                },
            );
        } else {
            return;
        }
    }
    let Some(request) = values.get("request").and_then(|v| v.parse::<u64>().ok()) else {
        if url.path() == "/navigate" {
            navigate(app, id, label, token, &values);
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
        let worker_app = app.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            worker_app.state::<NavigationWriter>().flush()
        })
        .await;
        let rows = match result {
            Ok(Ok(())) => executor::run(app.clone(), recent::list).await,
            _ => Err("History unavailable".into()),
        };
        if !valid(&app, &id, &label, &token)
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
                .filter(|s| s.document == document && s.sequence == sequence)
            else {
                return;
            };
            let first = !session.presented;
            session.presented = true;
            first
        };
        let autofocus = first
            && app
                .state::<BrowserState>()
                .active_tab_id
                .lock()
                .is_ok_and(|active| *active == id)
            && !app
                .state::<BrowserState>()
                .is_overlay_open
                .lock()
                .map_or(true, |v| *v);
        let payload = serde_json::json!({"preferences":prefs,"labels":text,"searches":rows.as_ref().ok(),"error":rows.is_err(),"autofocus":autofocus});
        if let Some(view) = app.get_webview(&label) {
            if autofocus {
                let _ = view.set_focus();
            }
            let _ = view.eval(format!(
                "window.__veloceNewTabReply?.({}, {}, {});",
                serde_json::to_string(&document).unwrap(),
                request,
                payload
            ));
        }
    });
}

fn navigate(app: &AppHandle, id: &str, label: &str, token: &str, values: &HashMap<String, String>) {
    let Some(input) = values
        .get("input")
        .filter(|s| !s.trim().is_empty() && s.len() <= 8192)
        .cloned()
    else {
        return;
    };
    let requested = values
        .get("engine")
        .filter(|s| ["google", "duckduckgo"].contains(&s.as_str()))
        .cloned();
    let document = values["document"].clone();
    let sequence = values["sequence"].parse::<u64>().unwrap_or(0);
    let (app, id, label, token) = (
        app.clone(),
        id.to_owned(),
        label.to_owned(),
        token.to_owned(),
    );
    tauri::async_runtime::spawn(async move {
        let state = app.state::<BrowserState>();
        let _operation = state.tab_operations.lock().await;
        if !valid(&app, &id, &label, &token)
            || !app.state::<NewTabService>().sessions.lock().is_ok_and(|s| {
                s.get(&label)
                    .is_some_and(|s| s.document == document && s.navigation == sequence)
            })
            || !state
                .snapshot_tabs()
                .iter()
                .any(|tab| tab.id == id && tab.is_visible)
            || state.is_overlay_open.lock().map_or(true, |v| *v)
            || app.state::<crate::shutdown::ShutdownService>().blocked()
        {
            return;
        }
        let repeat = requested.is_some();
        let engine = requested.unwrap_or_else(|| {
            app.state::<NewTabService>()
                .preferences
                .lock()
                .map(|p| p.engine.clone())
                .unwrap_or_else(|_| "google".into())
        });
        let result = (|| {
            let target = if repeat {
                let mut url = Url::parse(if engine == "duckduckgo" {
                    "https://duckduckgo.com/"
                } else {
                    "https://www.google.com/search"
                })
                .unwrap();
                url.query_pairs_mut().append_pair("q", &input);
                url
            } else {
                super::super::url_input::resolve_browser_input(&app, &input, &engine)?
            };
            if crate::privacy::adblock_engine::should_block_url(
                target.as_str(),
                *state.shield_enabled.lock().map_err(|e| e.to_string())?,
            ) {
                return Err("Navigation blocked".to_string());
            }
            // Select the originating pane under the same operation lock before navigation.
            TabCheckpoint::commit_with_activation(&state, Some((&id, false)), || {
                state.select_tab(&id).map(|_| ())
            })?;
            app.get_webview(&webview_label_for_tab(&id))
                .ok_or("Page unavailable")?
                .navigate(physical_url(&app, target.as_str())?)
                .map_err(|e| e.to_string())?;
            let _ = app.emit("tabs-updated", state.snapshot_packet());
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            let _ = app.emit_to("main", "browser-error", error);
        }
    });
}
