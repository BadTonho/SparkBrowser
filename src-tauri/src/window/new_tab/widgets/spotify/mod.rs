//! One shared Spotify connection per normal profile. The renderer receives display data only.
mod api;
mod platform;
#[cfg(test)]
mod tests;
use super::{broadcast, now, oauth, BrowserState, Settings, Widgets};
use crate::database::new_tab_repo;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{AppHandle, Manager};

const SCOPE: &str = "user-read-playback-state user-modify-playback-state";
#[derive(Clone, Serialize, Deserialize)]
struct Credential {
    client_id: String,
    refresh: String,
}
#[derive(Clone, Default, Serialize)]
pub struct View {
    connected: bool,
    connecting: bool,
    busy: bool,
    error: Option<String>,
    data: Option<api::Playback>,
}
#[derive(Default)]
struct Runtime {
    settings: Settings,
    view: View,
    credential: Option<Credential>,
    access: Option<oauth::Access>,
    connection: Option<(String, Arc<AtomicBool>)>,
    epoch: u64,
    due: u64,
    retry_until: u64,
    fixture: Option<String>,
    isolated: bool,
}
#[derive(Default)]
pub struct Service {
    runtime: Mutex<Runtime>,
}
pub(super) fn private(app: &AppHandle) -> bool {
    app.try_state::<crate::private_session::PrivateRole>()
        .is_some_and(|r| r.is_private())
}
fn blocked(app: &AppHandle, value: Option<bool>) -> Result<bool, String> {
    let browser = app.state::<BrowserState>();
    let db = browser
        .infrastructure()
        .db()
        .lock()
        .map_err(|_| "settings_unavailable")?;
    let db = db.as_ref().ok_or("settings_unavailable")?;
    if let Some(value) = value {
        new_tab_repo::set_spotify_disconnected(db, value)?;
    }
    new_tab_repo::spotify_disconnected(db)
}
fn endpoints(fixture: &Option<String>) -> super::Endpoints {
    let mut e = super::Endpoints {
        authorize: "https://accounts.spotify.com/authorize".into(),
        token: "https://accounts.spotify.com/api/token".into(),
        ..Default::default()
    };
    if let Some(base) = fixture {
        e.authorize = format!("{base}/spotify/oauth");
        e.token = format!("{base}/spotify/token");
        e.fixture = Some(format!("{base}/spotify"));
    }
    e
}
fn api_endpoint(fixture: &Option<String>) -> String {
    fixture
        .as_ref()
        .map(|b| format!("{b}/spotify/v1"))
        .unwrap_or_else(|| "https://api.spotify.com/v1".into())
}
pub fn initialize(app: &AppHandle, settings: &Settings, fixture: Option<String>) {
    if private(app) {
        return;
    }
    let service = app.state::<Widgets>();
    let mut s = service.spotify.runtime.lock().unwrap();
    s.settings = settings.clone();
    s.fixture = fixture;
    s.isolated = app.config().identifier == "org.spark.browser.native-tests";
    let epoch = s.epoch;
    let restore = !s.isolated && !blocked(app, None).unwrap_or(true);
    drop(s);
    if restore {
        let app = app.clone();
        std::thread::spawn(move || {
            let result = platform::read(&app);
            let service = app.state::<Widgets>();
            let mut s = service.spotify.runtime.lock().unwrap();
            if s.epoch != epoch {
                return;
            }
            match result {
                Ok(Some(raw)) => {
                    if let Ok(c) = serde_json::from_str::<Credential>(&raw) {
                        if c.client_id == s.settings.spotify_client_id && !c.refresh.is_empty() {
                            s.credential = Some(c);
                            s.view.connected = true;
                        }
                    }
                }
                Err(e) => s.view.error = Some(e),
                _ => {}
            }
            drop(s);
            broadcast(&app);
        });
    }
}
pub fn snapshot(app: &AppHandle) -> View {
    if private(app) {
        return View::default();
    }
    let service = app.state::<Widgets>();
    let s = service.spotify.runtime.lock().unwrap();
    if s.settings.spotify_enabled {
        s.view.clone()
    } else {
        View {
            connected: s.view.connected,
            ..Default::default()
        }
    }
}
fn cancel(s: &mut Runtime) {
    s.epoch += 1;
    if let Some((_, flag)) = s.connection.take() {
        flag.store(true, Ordering::SeqCst);
    }
    s.view.connecting = false;
    s.view.busy = false;
}
pub fn revoke(app: &AppHandle, owner: &str) {
    let service = app.state::<Widgets>();
    let mut s = service.spotify.runtime.lock().unwrap();
    if s.connection
        .as_ref()
        .is_some_and(|(label, _)| label == owner)
    {
        cancel(&mut s);
    }
}
pub fn configure(app: &AppHandle, settings: &Settings) {
    let service = app.state::<Widgets>();
    let mut s = service.spotify.runtime.lock().unwrap();
    if s.settings.spotify_client_id != settings.spotify_client_id {
        cancel(&mut s);
        s.credential = None;
        s.access = None;
        s.view = View::default();
        if let Err(e) = blocked(app, Some(true)) {
            s.view.error = Some(e);
        }
    } else if s.settings.spotify_enabled != settings.spotify_enabled {
        cancel(&mut s);
        s.view.data = None;
        s.view.error = None;
    }
    s.settings = settings.clone();
    s.due = 0;
}
fn save(app: &AppHandle, s: &Runtime, c: &Credential) -> Result<(), String> {
    if s.isolated {
        return Ok(());
    }
    platform::write(
        app,
        &serde_json::to_string(c).map_err(|_| "secure_storage")?,
    )
}
pub async fn connect(app: &AppHandle, owner: String) -> Result<(), String> {
    if private(app) {
        return Err("cancelled".into());
    }
    let service = app.state::<Widgets>();
    let (epoch, client_id, fixture, flag) = {
        let mut s = service.spotify.runtime.lock().unwrap();
        if !s.settings.spotify_enabled
            || s.view.busy
            || s.view.connecting
            || (s.isolated && s.fixture.is_none())
        {
            return Err("cancelled".into());
        }
        if s.settings.spotify_client_id.is_empty() {
            return Err("spotify_configure".into());
        }
        if now() < s.retry_until {
            return Err("rate_limited".into());
        }
        cancel(&mut s);
        s.view.connecting = true;
        s.view.error = None;
        let flag = Arc::new(AtomicBool::new(false));
        s.connection = Some((owner, flag.clone()));
        (
            s.epoch,
            s.settings.spotify_client_id.clone(),
            s.fixture.clone(),
            flag,
        )
    };
    broadcast(app);
    let worker_id = client_id.clone();
    let worker_flag = flag.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        oauth::connect_scoped(
            &oauth::Config {
                client_id: worker_id,
                client_secret: String::new(),
            },
            &endpoints(&fixture),
            worker_flag,
            SCOPE,
            platform::open,
        )
    })
    .await
    .map_err(|_| "oauth_error".to_owned())
    .and_then(|r| r);
    let mut s = service.spotify.runtime.lock().unwrap();
    if s.epoch != epoch || flag.load(Ordering::SeqCst) {
        return Err("cancelled".into());
    }
    s.view.connecting = false;
    s.connection = None;
    let result = result.and_then(|grant| {
        let c = Credential {
            client_id,
            refresh: grant.refresh.ok_or("reconnect")?,
        };
        save(app, &s, &c)?;
        blocked(app, Some(false))?;
        s.credential = Some(c);
        s.access = Some(grant.access);
        s.view.connected = true;
        s.due = 0;
        Ok(())
    });
    let result = result.map_err(api::Failure::from);
    if let Err(e) = &result {
        s.view.error = Some(e.code.clone());
        if e.code == "rate_limited" {
            s.retry_until = now().saturating_add(e.retry);
        }
    }
    drop(s);
    broadcast(app);
    schedule(app, None);
    result.map_err(|e| e.code)
}
pub fn disconnect(app: &AppHandle) -> Result<(), String> {
    if private(app) {
        return Err("cancelled".into());
    }
    let service = app.state::<Widgets>();
    let mut s = service.spotify.runtime.lock().unwrap();
    cancel(&mut s);
    s.credential = None;
    s.access = None;
    s.view = View::default();
    // Persist the tombstone first: a failed deletion must never restore the account.
    let blocked = blocked(app, Some(true));
    let deleted = if s.isolated {
        Ok(())
    } else {
        platform::delete(app)
    };
    let result = blocked.and(deleted);
    if let Err(e) = &result {
        s.view.error = Some(e.clone());
    }
    drop(s);
    broadcast(app);
    result.map(|_| ())
}
pub fn schedule(app: &AppHandle, action: Option<&str>) {
    if private(app) || !super::visible(app) {
        return;
    }
    let service = app.state::<Widgets>();
    let mut s = service.spotify.runtime.lock().unwrap();
    if !s.settings.spotify_enabled
        || s.view.busy
        || s.view.connecting
        || s.credential.is_none()
        || now() < s.retry_until
        || s.view.error.as_deref() == Some("reconnect")
        || (action.is_none() && now() < s.due)
    {
        return;
    }
    if let Some(action) = action {
        let allowed = s.view.data.as_ref().is_some_and(|p| match action {
            "spotify_play" => p.can_play,
            "spotify_pause" => p.can_pause,
            "spotify_next" => p.can_next,
            "spotify_previous" => p.can_previous,
            _ => false,
        });
        if !allowed {
            return;
        }
    }
    s.view.busy = true;
    let (epoch, mut credential, access, fixture, previous) = (
        s.epoch,
        s.credential.clone().unwrap(),
        s.access.clone(),
        s.fixture.clone(),
        s.view.data.clone(),
    );
    let action = action.map(str::to_owned);
    let app = app.clone();
    drop(s);
    broadcast(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let result = (|| {
            let access = match access.filter(|a| a.expires > now()) {
                Some(a) => a,
                None => {
                    let grant = oauth::grant(
                        &endpoints(&fixture).token,
                        SCOPE,
                        &[
                            ("client_id", &credential.client_id),
                            ("refresh_token", &credential.refresh),
                            ("grant_type", "refresh_token"),
                        ],
                    )?;
                    if let Some(refresh) = grant.refresh {
                        credential.refresh = refresh;
                    }
                    grant.access
                }
            };
            // A disable/disconnect during refresh prevents a subsequent control request.
            {
                let service = app.state::<Widgets>();
                let mut s = service.spotify.runtime.lock().unwrap();
                if s.epoch != epoch {
                    return Err("cancelled".to_owned().into());
                }
                if s.credential
                    .as_ref()
                    .is_none_or(|old| old.refresh != credential.refresh)
                {
                    save(&app, &s, &credential)?;
                }
                s.credential = Some(credential.clone());
                s.access = Some(access.clone());
            }
            let endpoint = api_endpoint(&fixture);
            if let Some(action) = action {
                api::control(&endpoint, &access.token, &action)?;
            }
            let data = api::playback(&endpoint, &access.token, previous.as_ref())?;
            Ok::<_, api::Failure>((access, data))
        })();
        let service = app.state::<Widgets>();
        let mut s = service.spotify.runtime.lock().unwrap();
        if s.epoch != epoch {
            return;
        }
        s.view.busy = false;
        match result {
            Ok((access, data)) => {
                s.access = Some(access);
                s.credential = Some(credential);
                s.view.data = data;
                s.view.error = None;
                s.due = now().saturating_add(10);
                s.retry_until = 0;
            }
            Err(e) => {
                if e.code == "reconnect" {
                    s.access = None;
                    s.view.data = None;
                }
                s.view.error = Some(e.code);
                s.retry_until = now().saturating_add(e.retry);
                s.due = s.retry_until;
            }
        }
        drop(s);
        broadcast(&app);
    });
}
