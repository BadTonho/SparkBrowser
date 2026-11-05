//! Shared, device-local widgets. Neither account data nor tokens enter backups.
mod actions;
mod cookies;
mod gmail;
mod http;
mod oauth;
mod platform;
mod weather;
use crate::{
    database::{
        executor,
        new_tab_repo::{self, Settings},
    },
    state::browser_state::BrowserState,
};
pub use actions::handle;
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{AppHandle, Manager};

#[derive(Clone)]
struct Endpoints {
    weather: String,
    cities: String,
    gmail: String,
    authorize: String,
    token: String,
    revoke: String,
    fixture: Option<String>,
}
impl Default for Endpoints {
    fn default() -> Self {
        Self {
            weather: "https://api.open-meteo.com/v1/forecast".into(),
            cities: "https://geocoding-api.open-meteo.com/v1/search".into(),
            gmail: "https://gmail.googleapis.com/gmail/v1".into(),
            authorize: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token: "https://oauth2.googleapis.com/token".into(),
            revoke: "https://oauth2.googleapis.com/revoke".into(),
            fixture: None,
        }
    }
}
impl Endpoints {
    fn for_app(app: &AppHandle) -> Self {
        if app.config().identifier == "org.veloce.browser.native-tests" {
            if let Some(base) = app
                .path()
                .app_config_dir()
                .ok()
                .and_then(|dir| std::fs::read_to_string(dir.join("widget-fixture-url")).ok())
                .and_then(|raw| url::Url::parse(raw.trim()).ok())
                .filter(|u| {
                    u.scheme() == "http"
                        && u.host_str() == Some("127.0.0.1")
                        && u.port().is_some()
                        && u.username().is_empty()
                        && u.password().is_none()
                        && u.path() == "/"
                        && u.query().is_none()
                })
            {
                let base = base.as_str().trim_end_matches('/').to_owned();
                return Self {
                    weather: format!("{base}/forecast"),
                    cities: format!("{base}/cities"),
                    gmail: format!("{base}/gmail"),
                    authorize: format!("{base}/oauth"),
                    token: format!("{base}/token"),
                    revoke: format!("{base}/revoke"),
                    fixture: Some(base),
                };
            }
        }
        Self::default()
    }
}
#[derive(Clone, Default, Serialize)]
struct WeatherState {
    data: Option<weather::Forecast>,
    error: Option<String>,
    busy: bool,
}
#[derive(Clone, Default, Serialize)]
struct GmailState {
    data: Option<gmail::Inbox>,
    email: Option<String>,
    error: Option<String>,
    busy: bool,
    connecting: bool,
    configured: bool,
}
#[derive(Default)]
struct Runtime {
    settings: Settings,
    revision: u64,
    settings_error: Option<String>,
    weather: WeatherState,
    gmail: GmailState,
    coordinates: Option<(f64, f64)>,
    location_busy: bool,
    location_attempted: bool,
    weather_started: u64,
    mail_started: u64,
    weather_epoch: u64,
    mail_epoch: u64,
    weather_due: u64,
    mail_due: u64,
    weather_failures: u32,
    mail_failures: u32,
    credential: Option<oauth::Credential>,
    access: Option<oauth::Access>,
    connection: Option<(String, Arc<AtomicBool>)>,
    endpoints: Endpoints,
}
#[derive(Default)]
pub struct Widgets {
    runtime: Mutex<Runtime>,
    mutations: tauri::async_runtime::Mutex<()>,
    version: std::sync::atomic::AtomicU64,
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn visible(app: &AppHandle) -> bool {
    let browser = app.state::<BrowserState>();
    browser
        .snapshot_tabs()
        .iter()
        .any(|t| t.is_visible && super::is_new_tab(&t.url))
        && !browser.is_overlay_open.lock().map_or(true, |v| *v)
        && app
            .get_window("main")
            .is_some_and(|w| w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(true))
        && !app.state::<crate::shutdown::ShutdownService>().blocked()
}
pub fn initialize(app: &AppHandle) {
    let service = app.state::<Widgets>();
    if let Ok(mut state) = service.runtime.lock() {
        let browser = app.state::<BrowserState>();
        match browser
            .db
            .lock()
            .ok()
            .and_then(|db| db.as_ref().map(new_tab_repo::load))
        {
            Some(Ok(settings)) => state.settings = settings,
            _ => state.settings_error = Some("settings_unavailable".into()),
        }
        state.endpoints = Endpoints::for_app(app);
        let oauth_configured = oauth::config(app).is_ok();
        state.gmail.configured = true;
        // Native fixtures never read the user's Windows credential vault.
        let disconnected = browser
            .db
            .lock()
            .ok()
            .and_then(|db| db.as_ref().map(new_tab_repo::account_disconnected))
            .and_then(Result::ok)
            .unwrap_or(true);
        if state.endpoints.fixture.is_none() && oauth_configured && !disconnected {
            // The OS credential vault can be slow; read it off the startup path.
            let app = app.clone();
            std::thread::spawn(move || {
                let read = platform::read(&platform::credential_key(&app));
                let widgets = app.state::<Widgets>();
                let Ok(mut state) = widgets.runtime.lock() else {
                    return;
                };
                match read {
                    Ok(Some(raw)) => {
                        if let Ok(credential) = serde_json::from_str::<oauth::Credential>(&raw) {
                            if oauth::config(&app).is_ok_and(|c| c.client_id == credential.client_id)
                            {
                                state.gmail.email = Some(credential.email.clone());
                                state.credential = Some(credential);
                            }
                        }
                    }
                    Err(error) => state.gmail.error = Some(error),
                    _ => {}
                }
                drop(state);
                broadcast(&app);
            });
        }
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            if app.state::<crate::shutdown::ShutdownService>().blocked() {
                continue;
            }
            schedule(&app);
        }
    });
}
pub fn revoke(app: &AppHandle, label: &str) {
    if let Ok(mut state) = app.state::<Widgets>().runtime.lock() {
        if state
            .connection
            .as_ref()
            .is_some_and(|(owner, _)| owner == label)
        {
            if let Some((_, cancel)) = state.connection.take() {
                cancel.store(true, Ordering::SeqCst);
            }
            state.mail_epoch += 1;
            state.gmail.connecting = false;
        }
    }
}
fn snapshot(app: &AppHandle) -> serde_json::Value {
    let service = app.state::<Widgets>();
    let state = service.runtime.lock().unwrap();
    serde_json::json!({"version":service.version.fetch_add(1,Ordering::SeqCst),"settings":state.settings,"revision":state.revision,"settings_error":state.settings_error,"weather":state.weather,"gmail":if state.settings.gmail_enabled {state.gmail.clone()}else{GmailState {configured:state.gmail.configured,email:state.gmail.email.clone(),..Default::default()}},"location_busy":state.location_busy})
}
fn broadcast(app: &AppHandle) {
    for tab in app.state::<BrowserState>().snapshot_tabs() {
        if super::is_new_tab(&tab.url) {
            if let Some(view) =
                app.get_webview(&crate::state::tab_model::webview_label_for_tab(&tab.id))
            {
                let _ = view.eval("window.__veloceRefreshWidgets?.()");
            }
        }
    }
}
fn retry_delay(failures: u32, normal: u64) -> u64 {
    normal
        .max(15u64.saturating_mul(1u64 << failures.min(8)))
        .min(3600)
}
fn schedule(app: &AppHandle) {
    if !visible(app) {
        return;
    }
    let focused = app
        .get_window("main")
        .is_some_and(|w| w.is_focused().unwrap_or(false));
    let service = app.state::<Widgets>();
    let mut state = service.runtime.lock().unwrap();
    let time = now();
    if state.settings.weather_enabled
        && state.settings.automatic_location
        && state.coordinates.is_none()
        && !state.location_busy
        && !state.location_attempted
        && focused
    {
        state.location_busy = true;
        state.location_attempted = true;
        let app = app.clone();
        let epoch = state.weather_epoch;
        tauri::async_runtime::spawn(async move {
            locate(&app, epoch).await;
        });
    }
    let coordinates = if state.settings.automatic_location {
        state.coordinates
    } else {
        state
            .settings
            .city
            .as_ref()
            .map(|c| (c.latitude, c.longitude))
    };
    if state.settings.weather_enabled && !state.weather.busy && time >= state.weather_due {
        if let Some(coordinates) = coordinates {
            state.weather.busy = true;
            state.weather_started = time;
            let (app, epoch, endpoint) = (
                app.clone(),
                state.weather_epoch,
                state.endpoints.weather.clone(),
            );
            tauri::async_runtime::spawn_blocking(move || {
                let result = weather::fetch(&endpoint, coordinates);
                let service = app.state::<Widgets>();
                let mut state = service.runtime.lock().unwrap();
                if state.weather_epoch != epoch {
                    return;
                }
                state.weather.busy = false;
                match result {
                    Ok(data) => {
                        state.weather.data = Some(data);
                        state.weather.error = None;
                        state.weather_failures = 0;
                    }
                    Err(error) => {
                        state.weather.error = Some(error);
                        state.weather_failures += 1;
                    }
                }
                state.weather_due = now() + retry_delay(state.weather_failures, 900);
                drop(state);
                broadcast(&app);
            });
        }
    }
    if state.settings.gmail_enabled
        && !state.gmail.busy
        && !state.gmail.connecting
        && time >= state.mail_due
    {
        if let Some(credential) = state.credential.clone() {
            state.gmail.busy = true;
            state.mail_started = time;
            let (app, epoch, endpoints, access) = (
                app.clone(),
                state.mail_epoch,
                state.endpoints.clone(),
                state.access.clone(),
            );
            tauri::async_runtime::spawn_blocking(move || {
                let result = (|| {
                    let mut credential = credential;
                    let access = match access.filter(|a| a.expires > now()) {
                        Some(access) => access,
                        None => {
                            let config = oauth::config(&app)?;
                            if config.client_id != credential.client_id {
                                return Err("reconnect".into());
                            }
                            let grant = oauth::refresh(&config, &endpoints, &credential.refresh)?;
                            if let Some(refresh) = grant.refresh {
                                credential.refresh = refresh;
                            }
                            grant.access
                        }
                    };
                    let inbox = gmail::inbox(&endpoints.gmail, &access.token, &credential.email)?;
                    Ok::<_, String>((access, credential, inbox))
                })();
                let service = app.state::<Widgets>();
                let mut state = service.runtime.lock().unwrap();
                if state.mail_epoch != epoch {
                    return;
                }
                state.gmail.busy = false;
                match result {
                    Ok((access, credential, inbox)) => {
                        if endpoints.fixture.is_none() {
                            let saved = serde_json::to_string(&credential)
                                .map_err(|_| "secure_storage".to_owned())
                                .and_then(|raw| {
                                    platform::write(&platform::credential_key(&app), &raw)
                                });
                            if let Err(error) = saved {
                                state.gmail.error = Some(error);
                                state.mail_due = now() + 60;
                                drop(state);
                                broadcast(&app);
                                return;
                            }
                        }
                        state.credential = Some(credential);
                        state.access = Some(access);
                        state.gmail.data = Some(inbox);
                        state.gmail.error = None;
                        state.mail_failures = 0;
                    }
                    Err(error) => {
                        if error == "reconnect" {
                            state.access = None;
                            state.gmail.data = None;
                        }
                        state.gmail.error = Some(error);
                        state.mail_failures += 1;
                    }
                }
                state.mail_due = now() + retry_delay(state.mail_failures, 60);
                drop(state);
                broadcast(&app);
            });
        } else if state.endpoints.fixture.is_none() {
            state.gmail.busy = true;
            state.mail_started = time;
            let (app, epoch) = (app.clone(), state.mail_epoch);
            tauri::async_runtime::spawn(async move {
                let cookies = cookies::get_cookies_for_url(&app, "https://mail.google.com").await;
                let result = match cookies {
                    Ok(ref c)
                        if c.contains("SID=") || c.contains("OSID=") || c.contains("SAPISID=") =>
                    {
                        let c_str = c.clone();
                        tauri::async_runtime::spawn_blocking(move || {
                            gmail::fetch_feed("https://mail.google.com/mail/feed/atom", &c_str)
                        })
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r)
                    }
                    _ => Err("not_connected".into()),
                };
                let service = app.state::<Widgets>();
                let mut state = service.runtime.lock().unwrap();
                if state.mail_epoch != epoch {
                    return;
                }
                state.gmail.busy = false;
                match result {
                    Ok(inbox) => {
                        state.gmail.email = if inbox.email.is_empty() {
                            None
                        } else {
                            Some(inbox.email.clone())
                        };
                        state.gmail.data = Some(inbox);
                        state.gmail.error = None;
                        state.mail_failures = 0;
                    }
                    Err(e) => {
                        if e == "not_connected" || e == "not_authenticated" {
                            state.gmail.email = None;
                            state.gmail.data = None;
                            state.gmail.error = None;
                        } else {
                            state.gmail.error = Some(e);
                            state.mail_failures += 1;
                        }
                    }
                }
                state.mail_due = now() + retry_delay(state.mail_failures, 60);
                drop(state);
                broadcast(&app);
            });
        }
    }
}
async fn locate(app: &AppHandle, epoch: u64) {
    let result = platform::locate(app).await;
    let service = app.state::<Widgets>();
    let mut state = service.runtime.lock().unwrap();
    if state.weather_epoch != epoch {
        return;
    }
    state.location_busy = false;
    match result {
        Ok(coordinates) => {
            state.coordinates = Some(coordinates);
            state.weather_due = 0;
            state.weather.error = None;
        }
        Err(error) => state.weather.error = Some(error),
    }
    drop(state);
    broadcast(app);
    schedule(app);
}
#[cfg(test)]
mod tests {
    #[test]
    fn retries_are_bounded_and_slow_down() {
        assert_eq!(super::retry_delay(0, 60), 60);
        assert!(super::retry_delay(5, 60) > 60);
        assert_eq!(super::retry_delay(99, 900), 3600);
    }
}
