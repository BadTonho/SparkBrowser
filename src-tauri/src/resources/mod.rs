//! Limits and live use of the browser's resources: the processor and the internet.
//!
//! Memory is limited by hibernating tabs, which the interface does with the commands it already has.
//!
//! - **Processor**: on Windows the processes of the web engine are put in a job with a hard cap on the share of
//!   the processor they may use; the browser's own interface is outside it, so it stays responsive. On Linux
//!   the web processes get a lower priority, which is a weaker control (it only matters when the machine is busy).
//! - **Internet**: on Windows the pages' network conditions are limited through the engine's developer
//!   protocol (`Network.emulateNetworkConditions`); on Linux the engine has no such control, so only the
//!   browser's own downloads are limited.
//!
//! The numbers shown in the panel come from [`usage`]. They are measured only while the panel is open.
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};
use std::time::Instant;
use tauri::AppHandle;

pub mod memory_level;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as platform;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(not(any(windows, target_os = "linux")))]
mod unsupported;
#[cfg(not(any(windows, target_os = "linux")))]
use unsupported as platform;

/// The limits the user chose. `0` means no limit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Share of the processor the web engine may use, in percent (1 to 99).
    pub cpu_percent: u8,
    /// Speed of the pages' network in kilobits per second, for downloading and uploading.
    pub network_kbps: u32,
}

impl Limits {
    pub fn valid(self) -> bool {
        self.cpu_percent < 100 && self.network_kbps <= 1_000_000
    }
}

/// Live numbers for the panel.
#[derive(Debug, Default, Serialize)]
pub struct Usage {
    /// Share of the whole processor the browser's processes used since the last reading, 0 to 100.
    pub cpu_percent: f32,
    /// Bytes per second received by the pages since the last reading; `None` when this system cannot measure it.
    pub download_bytes_per_second: Option<u64>,
    pub supports_cpu_limit: bool,
    /// The processor limit lowers the priority of the engine instead of capping it (Linux).
    pub cpu_limit_is_priority: bool,
    pub supports_network_limit: bool,
}

pub struct Resources {
    limits: Mutex<Limits>,
    monitoring: AtomicBool,
    last: Mutex<Option<Reading>>,
}

struct Reading {
    at: Instant,
    cpu_seconds: f64,
    received: u64,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            limits: Mutex::new(Limits::default()),
            monitoring: AtomicBool::new(false),
            last: Mutex::new(None),
        }
    }
}

/// Bytes the pages have received since the monitor started (the monitor adds to it).
pub(crate) static RECEIVED: AtomicU64 = AtomicU64::new(0);
/// Speed limit of the browser's own downloads, in bytes per second; `0` is none.
pub static DOWNLOAD_LIMIT: AtomicU64 = AtomicU64::new(0);

impl Resources {
    pub fn limits(&self) -> Limits {
        self.limits.lock().map(|limits| *limits).unwrap_or_default()
    }

    pub fn monitoring(&self) -> bool {
        self.monitoring.load(Ordering::Relaxed)
    }
}

/// Applies the limits now and keeps them for the pages that open later.
pub fn configure(app: &AppHandle, limits: Limits) -> Result<(), String> {
    if !limits.valid() {
        return Err("resources.invalid_limits".into());
    }
    use tauri::Manager;
    let state = app.state::<Resources>();
    *state.limits.lock().map_err(|e| e.to_string())? = limits;
    DOWNLOAD_LIMIT.store(u64::from(limits.network_kbps) * 1000 / 8, Ordering::Relaxed);
    platform::apply(app, limits, state.monitoring())
}

/// Starts or stops measuring the network. Measuring is only needed while the panel is open.
pub fn monitor(app: &AppHandle, on: bool) -> Result<(), String> {
    use tauri::Manager;
    let state = app.state::<Resources>();
    state.monitoring.store(on, Ordering::Relaxed);
    if on {
        if let Ok(mut last) = state.last.lock() {
            *last = None;
        }
    }
    platform::apply(app, state.limits(), on)
}

/// A page that was just created gets the limits and the monitor, like the ones that were already open.
pub fn webview_created(app: &AppHandle, view: &tauri::Webview) {
    use tauri::Manager;
    let state = app.state::<Resources>();
    platform::attach(view, state.limits(), state.monitoring());
}

/// Reads the live numbers. The first reading after the monitor starts has no earlier one to compare with
/// and reports zero.
pub fn usage(app: &AppHandle) -> Usage {
    use tauri::Manager;
    let state = app.state::<Resources>();
    let now = Instant::now();
    let cpu_seconds = platform::cpu_seconds();
    let received = RECEIVED.load(Ordering::Relaxed);
    let mut usage = Usage {
        supports_cpu_limit: platform::SUPPORTS_CPU_LIMIT,
        cpu_limit_is_priority: platform::CPU_LIMIT_IS_PRIORITY,
        supports_network_limit: platform::SUPPORTS_NETWORK_LIMIT,
        download_bytes_per_second: platform::SUPPORTS_NETWORK_MONITOR.then_some(0),
        ..Usage::default()
    };
    if let Ok(mut last) = state.last.lock() {
        if let (Some(before), Some(cpu)) = (last.as_ref(), cpu_seconds) {
            let elapsed = now.duration_since(before.at).as_secs_f64();
            if elapsed > 0.2 {
                let cores = std::thread::available_parallelism().map_or(1, |n| n.get()) as f64;
                usage.cpu_percent = (((cpu - before.cpu_seconds) / (elapsed * cores)) * 100.0)
                    .clamp(0.0, 100.0) as f32;
                if platform::SUPPORTS_NETWORK_MONITOR {
                    usage.download_bytes_per_second =
                        Some((received.saturating_sub(before.received) as f64 / elapsed) as u64);
                }
            }
        }
        if let Some(cpu) = cpu_seconds {
            *last = Some(Reading {
                at: now,
                cpu_seconds: cpu,
                received,
            });
        }
    }
    usage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_sensible_limits_are_accepted() {
        assert!(Limits::default().valid());
        assert!(Limits {
            cpu_percent: 50,
            network_kbps: 5_000
        }
        .valid());
        assert!(
            !Limits {
                cpu_percent: 100,
                network_kbps: 0
            }
            .valid(),
            "100% is no limit: use 0"
        );
        assert!(!Limits {
            cpu_percent: 0,
            network_kbps: 2_000_000
        }
        .valid());
    }

    #[test]
    fn a_network_limit_becomes_bytes_per_second_for_downloads() {
        DOWNLOAD_LIMIT.store(u64::from(8_000u32) * 1000 / 8, Ordering::Relaxed);
        assert_eq!(DOWNLOAD_LIMIT.load(Ordering::Relaxed), 1_000_000);
        DOWNLOAD_LIMIT.store(0, Ordering::Relaxed);
    }
}
