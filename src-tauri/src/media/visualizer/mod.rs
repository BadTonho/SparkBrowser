//! One process-scoped audio analyzer shared by visible, authenticated new-tab documents.
#[cfg(any(target_os = "linux", test))]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod linux;
#[cfg(any(target_os = "linux", test))]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod pulse;
mod spectrum;
#[cfg(windows)]
mod windows;
use crate::{database::new_tab_repo::AudioBackground, window::new_tab::audio::Consumer};
#[cfg(target_os = "linux")]
use linux as platform;
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
#[cfg(windows)]
use windows as platform;

#[derive(Default)]
struct State {
    preferences: AudioBackground,
    consumers: HashMap<String, Consumer>,
    generation: u64,
    sequence: u64,
}
#[derive(Default)]
pub struct Visualizer {
    state: Mutex<State>,
}
#[derive(Serialize)]
struct Frame {
    generation: u64,
    sequence: u64,
    status: &'static str,
    bands: [f32; spectrum::BANDS],
}
fn private(app: &AppHandle) -> bool {
    app.try_state::<crate::private_session::PrivateRole>()
        .is_some_and(|r| r.is_private())
}
pub fn configure(app: &AppHandle, preferences: &AudioBackground) {
    let service = app.state::<Visualizer>();
    if let Ok(mut s) = service.state.lock() {
        let preferences = if private(app) {
            AudioBackground::default()
        } else {
            preferences.clone()
        };
        if s.preferences != preferences {
            s.preferences = preferences;
            s.generation += 1;
        }
    };
}
pub(crate) fn subscribe(app: &AppHandle, consumer: Consumer) {
    if private(app) || !consumer.visible(app) {
        return;
    }
    if let Ok(mut s) = app.state::<Visualizer>().state.lock() {
        s.consumers.insert(consumer.label.clone(), consumer);
    }
}
pub fn unsubscribe(app: &AppHandle, label: &str) {
    if let Ok(mut s) = app.state::<Visualizer>().state.lock() {
        s.consumers.remove(label);
    }
}
pub fn start(app: &AppHandle) {
    if private(app) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || run(app));
}
fn audible(app: &AppHandle) -> bool {
    app.state::<super::MediaService>()
        .snapshot(app)
        .is_ok_and(|s| s.tabs.iter().any(|t| t.audible && !t.muted && t.volume > 0))
}
#[cfg(any(windows, target_os = "linux"))]
fn run(app: AppHandle) {
    #[cfg(windows)]
    let apartment = platform::Apartment::enter();
    let mut backend: Option<platform::Backend> = None;
    let mut roots = Vec::new();
    let mut next_probe = Instant::now();
    let mut previous_idle = None;
    loop {
        if app.state::<crate::shutdown::ShutdownService>().blocked() {
            // A protected-close review can be cancelled. Pause rather than permanently
            // terminating the worker, and release capture while shutdown is pending.
            drop(backend.take());
            roots.clear();
            next_probe = Instant::now();
            std::thread::sleep(Duration::from_millis(200));
            continue;
        }
        let service = app.state::<Visualizer>();
        // Only a copy is taken under the lock. Asking whether a window is visible is answered by the
        // main thread, and the main thread may be waiting for this very lock (a new tab replacing an
        // old one unsubscribes from here); holding the lock across that question deadlocks the app.
        let (enabled, snapshot, generation) = {
            let state = service.state.lock().unwrap();
            (
                state.preferences.enabled,
                state.consumers.values().cloned().collect::<Vec<_>>(),
                state.generation,
            )
        };
        let stale: Vec<(String, String)> = snapshot
            .iter()
            .filter(|c| !c.current(&app))
            .map(|c| (c.label.clone(), c.token.clone()))
            .collect();
        if !stale.is_empty() {
            let mut state = service.state.lock().unwrap();
            for (label, token) in &stale {
                // A newer subscription under the same label is not stale.
                if state
                    .consumers
                    .get(label)
                    .is_some_and(|c| &c.token == token)
                {
                    state.consumers.remove(label);
                }
            }
        }
        let consumers: Vec<_> = snapshot
            .into_iter()
            .filter(|c| !stale.iter().any(|(label, _)| label == &c.label) && c.visible(&app))
            .collect();
        let active = enabled && !consumers.is_empty() && audible(&app);
        let (status, bands) = if !active {
            backend = None;
            roots.clear();
            next_probe = Instant::now();
            ("idle", [0.0; spectrum::BANDS])
        } else {
            if Instant::now() >= next_probe {
                let found = platform::roots(&app);
                if found != roots || backend.is_none() {
                    drop(backend.take());
                    #[cfg(windows)]
                    let opened = if apartment.is_ok() {
                        platform::Backend::open(&found)
                    } else {
                        Err("audio_unavailable")
                    };
                    #[cfg(target_os = "linux")]
                    let opened = platform::Backend::open(&found);
                    backend = opened.ok();
                    roots = found;
                }
                next_probe = Instant::now() + Duration::from_secs(2);
            }
            match backend.as_mut().map(platform::Backend::read) {
                Some(Ok(bands)) => ("ready", bands),
                _ => {
                    backend = None;
                    ("audio_unavailable", [0.0; spectrum::BANDS])
                }
            }
        };
        // Disabled consumers receive one clear frame, never the stale spectrum.
        let deliver = active || previous_idle != Some((generation, consumers.len()));
        if deliver {
            let mut state = service.state.lock().unwrap();
            if state.generation == generation {
                state.sequence += 1;
                let frame = Frame {
                    generation,
                    sequence: state.sequence,
                    status,
                    bands,
                };
                drop(state);
                for consumer in &consumers {
                    consumer.deliver(&app, &frame);
                }
            }
        }
        previous_idle = if active {
            None
        } else {
            Some((generation, consumers.len()))
        };
        std::thread::sleep(Duration::from_millis(if active { 50 } else { 200 }));
    }
}
#[cfg(not(any(windows, target_os = "linux")))]
fn run(_app: AppHandle) {}
