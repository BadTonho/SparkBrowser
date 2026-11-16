use super::*;
use std::collections::HashMap;

#[derive(Clone)]
struct Source {
    app: AppHandle,
    id: String,
    label: String,
    token: String,
    document: String,
    request: u64,
    operation: String,
}
impl Source {
    fn valid(&self) -> bool {
        super::super::bridge::valid(&self.app, &self.id, &self.label, &self.token)
            && self
                .app
                .state::<super::super::NewTabService>()
                .sessions
                .lock()
                .is_ok_and(|items| {
                    items
                        .get(&self.label)
                        .is_some_and(|s| s.document == self.document)
                })
    }
    fn active(&self) -> bool {
        self.valid()
            && visible(&self.app)
            && self
                .app
                .state::<BrowserState>()
                .snapshot_tabs()
                .iter()
                .any(|t| t.id == self.id && t.is_visible)
    }
    fn reply(&self, extra: serde_json::Value) {
        if !self.valid() {
            return;
        }
        let mut payload = snapshot(&self.app);
        payload["operation"] = serde_json::json!(self.operation);
        if let Some(values) = extra.as_object() {
            for (key, value) in values {
                payload[key] = value.clone();
            }
        }
        if let Some(view) = self.app.get_webview(&self.label) {
            crate::error_log::ignored(view.eval(format!(
                "window.__sparkWidgetsReply?.({}, {}, {});",
                serde_json::to_string(&self.document).unwrap(),
                self.request,
                payload
            )));
        }
    }
}
pub fn handle(
    app: &AppHandle,
    id: &str,
    label: &str,
    token: &str,
    values: &HashMap<String, String>,
) {
    let Some(request) = values.get("request").and_then(|v| v.parse::<u64>().ok()) else {
        return;
    };
    let operation = values.get("operation").cloned().unwrap_or_default();
    let source = Source {
        app: app.clone(),
        id: id.into(),
        label: label.into(),
        token: token.into(),
        document: values["document"].clone(),
        request,
        operation: operation.clone(),
    };
    if operation == "read" {
        source.reply(serde_json::json!({}));
        schedule(app);
        return;
    }
    if operation == "audio_unsubscribe" && source.valid() {
        crate::media::visualizer::unsubscribe(app, label);
        source.reply(serde_json::json!({}));
        return;
    }
    if !source.active() {
        return;
    }
    if operation == "spotify_open" {
        let mut values = values.clone();
        values.insert("input".into(), "https://open.spotify.com/".into());
        super::super::bridge::navigate(app, id, label, token, &values, false);
        return;
    }
    if operation == "open_gmail" || (operation == "connect" && oauth::config(app).is_err()) {
        let email = app.state::<Widgets>().runtime.lock().ok().and_then(|s| {
            s.gmail
                .email
                .clone()
                .or_else(|| s.credential.as_ref().map(|c| c.email.clone()))
        });
        let email_url = email
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(gmail::inbox_url)
            .map(|u| u.to_string())
            .unwrap_or_else(|| "https://mail.google.com/".into());
        {
            let mut values = values.clone();
            values.insert("input".into(), email_url);
            super::super::bridge::navigate(app, id, label, token, &values, false);
        }
        return;
    }
    let data = values
        .get("data")
        .filter(|s| s.len() <= 8192)
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .unwrap_or_default();
    tauri::async_runtime::spawn(async move {
        let result = perform(&source, &operation, data).await;
        source.reply(match result {
            Ok(value) => value,
            Err(error) => serde_json::json!({"action_error":error}),
        });
    });
}
async fn perform(
    source: &Source,
    operation: &str,
    data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if !source.active() {
        return Err("cancelled".into());
    }
    let app = &source.app;
    if (operation.starts_with("spotify_") || operation.starts_with("youtube_"))
        && spotify::private(app)
    {
        return Err("cancelled".into());
    }
    match operation {
        "audio_subscribe" => {
            crate::media::visualizer::subscribe(
                app,
                super::super::audio::Consumer {
                    id: source.id.clone(),
                    label: source.label.clone(),
                    token: source.token.clone(),
                    document: source.document.clone(),
                },
            );
            Ok(serde_json::json!({}))
        }
        "audio_unsubscribe" => {
            crate::media::visualizer::unsubscribe(app, &source.label);
            Ok(serde_json::json!({}))
        }
        "spotify_connect" => {
            spotify::connect(app, source.label.clone()).await?;
            Ok(serde_json::json!({}))
        }
        "spotify_cancel" => {
            spotify::revoke(app, &source.label);
            broadcast(app);
            Ok(serde_json::json!({}))
        }
        "spotify_disconnect" => {
            let app = app.clone();
            tauri::async_runtime::spawn_blocking(move || spotify::disconnect(&app))
                .await
                .map_err(|_| "secure_storage")??;
            Ok(serde_json::json!({}))
        }
        "spotify_play" | "spotify_pause" | "spotify_next" | "spotify_previous" => {
            spotify::schedule(app, Some(operation));
            Ok(serde_json::json!({}))
        }
        "youtube_toggle" | "youtube_next" | "youtube_previous" | "youtube_open" => {
            youtube::control(app, operation).await?;
            Ok(serde_json::json!({}))
        }
        "panel" => {
            crate::window::tab_drag_preview::cancel(app, &app.state::<BrowserState>())?;
            Ok(serde_json::json!({}))
        }
        "save" => {
            let service = app.state::<Widgets>();
            let _guard = service.mutations.lock().await;
            if !source.active() {
                return Err("cancelled".into());
            }
            let settings: Settings =
                serde_json::from_value(data["settings"].clone()).map_err(|_| "invalid_settings")?;
            settings.validate()?;
            let revision = data["revision"].as_u64().ok_or("invalid_settings")?;
            if service.runtime.lock().unwrap().revision != revision {
                return Err("settings_changed".into());
            }
            let saved = settings.clone();
            executor::run(app.clone(), move |conn| new_tab_repo::save(conn, &saved)).await?;
            let mut state = service.runtime.lock().unwrap();
            let old = state.settings.clone();
            if old.weather_enabled != settings.weather_enabled
                || old.city != settings.city
                || old.automatic_location != settings.automatic_location
            {
                state.weather_epoch += 1;
                state.location_busy = false;
                state.location_attempted = false;
                state.weather = WeatherState::default();
                state.weather_due = 0;
                if !settings.automatic_location {
                    state.coordinates = None;
                }
            }
            if old.gmail_enabled != settings.gmail_enabled {
                state.mail_epoch += 1;
                state.gmail.busy = false;
                state.gmail.data = None;
                state.mail_due = 0;
                if let Some((_, cancel)) = state.connection.take() {
                    cancel.store(true, Ordering::SeqCst);
                }
                state.gmail.connecting = false;
            }
            spotify::configure(app, &settings);
            crate::media::visualizer::configure(app, &settings.audio_background);
            state.settings = settings;
            state.settings_error = None;
            state.revision += 1;
            drop(state);
            broadcast(app);
            schedule(app);
            Ok(serde_json::json!({}))
        }
        "cities" => {
            let query = data["query"]
                .as_str()
                .map(str::trim)
                .filter(|s| s.chars().count() >= 2 && s.len() <= 200)
                .ok_or("invalid_city")?
                .to_owned();
            let endpoint = app
                .state::<Widgets>()
                .runtime
                .lock()
                .unwrap()
                .endpoints
                .cities
                .clone();
            let language = app
                .state::<super::super::NewTabService>()
                .preferences
                .lock()
                .unwrap()
                .language
                .clone();
            let cities = tauri::async_runtime::spawn_blocking(move || {
                weather::cities(&endpoint, &query, &language)
            })
            .await
            .map_err(|_| "network_error")??;
            if !source.active() {
                return Err("cancelled".into());
            }
            Ok(serde_json::json!({"cities":cities,"city_query":data["query"]}))
        }
        "locate" => {
            let service = app.state::<Widgets>();
            let epoch = {
                let mut state = service.runtime.lock().unwrap();
                if !state.settings.weather_enabled
                    || !state.settings.automatic_location
                    || state.location_busy
                {
                    return Err("cancelled".into());
                }
                state.location_busy = true;
                state.weather_epoch
            };
            source.reply(serde_json::json!({}));
            let result = platform::locate(app).await;
            let current = source.valid();
            let mut state = service.runtime.lock().unwrap();
            if state.weather_epoch != epoch {
                return Err("cancelled".into());
            }
            state.location_busy = false;
            if !current {
                return Err("cancelled".into());
            }
            state.location_attempted = true;
            match result {
                Ok(coordinates) => {
                    state.coordinates = Some(coordinates);
                    state.weather_due = 0;
                    state.weather.error = None;
                }
                Err(error) => {
                    state.weather.error = Some(error.clone());
                    return Err(error);
                }
            }
            drop(state);
            broadcast(app);
            schedule(app);
            Ok(serde_json::json!({}))
        }
        "refresh_weather" | "refresh_gmail" => {
            let service = app.state::<Widgets>();
            let mut state = service.runtime.lock().unwrap();
            // Manual retries have a small floor to prevent click storms.
            if operation == "refresh_weather" {
                if now() >= state.weather_started + 2 {
                    state.weather_due = 0;
                }
            } else {
                if now() >= state.mail_started + 2 {
                    state.mail_due = 0;
                }
            }
            drop(state);
            schedule(app);
            Ok(serde_json::json!({}))
        }
        "connect" => connect(source).await,
        "cancel_connect" => {
            revoke(app, &source.label);
            broadcast(app);
            Ok(serde_json::json!({}))
        }
        "disconnect" => {
            let service = app.state::<Widgets>();
            let _guard = service.mutations.lock().await;
            let mut state = service.runtime.lock().unwrap();
            let token = state.credential.take().map(|c| c.refresh);
            state.access = None;
            state.mail_epoch += 1;
            if let Some((_, cancel)) = state.connection.take() {
                cancel.store(true, Ordering::SeqCst);
            }
            let configured = state.gmail.configured;
            state.gmail = GmailState {
                configured,
                ..Default::default()
            };
            let endpoints = state.endpoints.clone();
            // A durable tombstone prevents a failed vault deletion from
            // silently restoring the connection on the next launch.
            let blocked = {
                let browser = app.state::<BrowserState>();
                let db = browser
                    .infrastructure()
                    .db()
                    .lock()
                    .map_err(|_| "settings_unavailable")?;
                db.as_ref().ok_or("settings_unavailable").and_then(|conn| {
                    new_tab_repo::set_account_disconnected(conn, true)
                        .map_err(|_| "settings_unavailable")
                })
            };
            let deleted = if endpoints.fixture.is_none() {
                platform::delete(&platform::credential_key(app))
            } else {
                Ok(())
            };
            if let Err(error) = &deleted {
                state.gmail.error = Some(error.clone());
            }
            drop(state);
            broadcast(app);
            if let Some(token) = token {
                tauri::async_runtime::spawn_blocking(move || {
                    oauth::revoke(&endpoints.revoke, &token)
                });
            }
            deleted?;
            blocked?;
            Ok(serde_json::json!({}))
        }
        _ => Err("invalid_action".into()),
    }
}
async fn connect(source: &Source) -> Result<serde_json::Value, String> {
    let app = &source.app;
    let config = oauth::config(app)?;
    let service = app.state::<Widgets>();
    let (epoch, endpoints, cancel) = {
        let mut state = service.runtime.lock().unwrap();
        if !state.settings.gmail_enabled || state.gmail.connecting || state.gmail.busy {
            return Err("cancelled".into());
        }
        state.gmail.configured = true;
        state.mail_epoch += 1;
        state.gmail.connecting = true;
        state.gmail.error = None;
        let cancel = Arc::new(AtomicBool::new(false));
        state.connection = Some((source.label.clone(), cancel.clone()));
        (state.mail_epoch, state.endpoints.clone(), cancel)
    };
    source.reply(serde_json::json!({}));
    broadcast(app);
    let client_id = config.client_id.clone();
    let worker_endpoints = endpoints.clone();
    let worker_cancel = cancel.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let grant = oauth::connect(&config, &worker_endpoints, worker_cancel.clone())?;
        if worker_cancel.load(Ordering::SeqCst) {
            return Err("cancelled".into());
        }
        let email = gmail::profile(&worker_endpoints.gmail, &grant.access.token)?;
        Ok::<_, String>((grant, email))
    })
    .await
    .map_err(|_| "oauth_error")?;
    let _guard = service.mutations.lock().await;
    let current = source.valid();
    let mut state = service.runtime.lock().unwrap();
    if state.mail_epoch != epoch || cancel.load(Ordering::SeqCst) || !current {
        return Err("cancelled".into());
    }
    state.gmail.connecting = false;
    state.connection = None;
    match result {
        Ok((grant, email)) => {
            let refresh = grant.refresh.ok_or("reconnect")?;
            let credential = oauth::Credential {
                refresh,
                email: email.clone(),
                client_id,
            };
            if endpoints.fixture.is_none() {
                let raw = serde_json::to_string(&credential).map_err(|_| "secure_storage")?;
                if let Err(error) = platform::write(&platform::credential_key(app), &raw) {
                    state.gmail.error = Some(error.clone());
                    return Err(error);
                }
            }
            {
                let browser = app.state::<BrowserState>();
                let db = browser
                    .infrastructure()
                    .db()
                    .lock()
                    .map_err(|_| "settings_unavailable")?;
                new_tab_repo::set_account_disconnected(
                    db.as_ref().ok_or("settings_unavailable")?,
                    false,
                )?;
            }
            state.gmail.email = Some(email);
            state.credential = Some(credential);
            state.access = Some(grant.access);
            state.mail_due = 0;
        }
        Err(error) => {
            state.gmail.error = Some(error.clone());
            return Err(error);
        }
    }
    drop(state);
    broadcast(app);
    schedule(app);
    Ok(serde_json::json!({}))
}
