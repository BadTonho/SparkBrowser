//! Windows: a job object with a hard processor cap for the web engine's processes, and the developer protocol
//! for the pages' network speed and measurement.
use super::{Limits, RECEIVED};
use crate::sync::Mutex;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    OnceLock,
};
use tauri::{AppHandle, Manager, Webview};
use webview2_com::{
    CallDevToolsProtocolMethodCompletedHandler, CoTaskMemPWSTR,
    DevToolsProtocolEventReceivedEventHandler,
    Microsoft::Web::WebView2::Win32::{ICoreWebView2, ICoreWebView2DevToolsProtocolEventReceiver},
};
use windows::core::{Interface, HSTRING};
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectCpuRateControlInformation,
    SetInformationJobObject, JOBOBJECT_CPU_RATE_CONTROL_INFORMATION,
    JOBOBJECT_CPU_RATE_CONTROL_INFORMATION_0, JOB_OBJECT_CPU_RATE_CONTROL_ENABLE,
    JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
};
use windows::Win32::System::Threading::{
    GetCurrentProcessId, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SET_QUOTA, PROCESS_TERMINATE,
};

pub const SUPPORTS_CPU_LIMIT: bool = true;
pub const CPU_LIMIT_IS_PRIORITY: bool = false;
pub const SUPPORTS_NETWORK_LIMIT: bool = true;
pub const SUPPORTS_NETWORK_MONITOR: bool = true;

/// The job. It is created on first use and never closed: closing it would not release the cap on a process
/// that is still running, and the processes belong to it until they exit.
static JOB: OnceLock<Mutex<isize>> = OnceLock::new();
/// The cap currently set, to avoid reassigning on every pass when nothing changed.
static GUARD: AtomicBool = AtomicBool::new(false);

fn job() -> Result<HANDLE, String> {
    let slot = JOB.get_or_init(|| {
        Mutex::new(unsafe { CreateJobObjectW(None, None) }.map_or(0, |handle| handle.0 as isize))
    });
    let raw = *slot.lock().map_err(|e| e.to_string())?;
    if raw == 0 {
        return Err("resources.job_unavailable".into());
    }
    Ok(HANDLE(raw as *mut c_void))
}

/// Every process that descends from this one: the web engine's browser, renderer, GPU and utility processes.
fn descendants() -> Vec<u32> {
    let mut parents: HashMap<u32, u32> = HashMap::new();
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return Vec::new();
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                parents.insert(entry.th32ProcessID, entry.th32ParentProcessID);
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    let own = unsafe { GetCurrentProcessId() };
    let mut family = HashSet::from([own]);
    // Parents can be listed after their children, so repeat until nothing new is found.
    loop {
        let before = family.len();
        for (pid, parent) in &parents {
            if family.contains(parent) {
                family.insert(*pid);
            }
        }
        if family.len() == before {
            break;
        }
    }
    family.remove(&own);
    family.into_iter().collect()
}

fn set_cap(percent: u8) -> Result<(), String> {
    let job = job()?;
    let info = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION {
        ControlFlags: JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
        Anonymous: JOBOBJECT_CPU_RATE_CONTROL_INFORMATION_0 {
            // The rate is in hundredths of a percent; 100% is "no cap".
            CpuRate: if percent == 0 {
                10_000
            } else {
                u32::from(percent) * 100
            },
        },
    };
    unsafe {
        SetInformationJobObject(
            job,
            JobObjectCpuRateControlInformation,
            &info as *const _ as *const c_void,
            std::mem::size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32,
        )
    }
    .map_err(|error| error.to_string())
}

/// Puts the engine's processes in the job. Processes the engine starts later join it by themselves, because
/// their parent is in it; this only has to catch the ones that exist (and new engines of isolated workspaces).
fn assign_all() -> Result<usize, String> {
    let job = job()?;
    let mut assigned = 0;
    for pid in descendants() {
        unsafe {
            let Ok(process) = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, false, pid) else {
                continue;
            };
            if AssignProcessToJobObject(job, process).is_ok() {
                assigned += 1;
            }
            let _ = CloseHandle(process);
        }
    }
    Ok(assigned)
}

fn file_time_seconds(time: FILETIME) -> f64 {
    ((u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)) as f64 / 10_000_000.0
}

/// Processor time used so far by the engine's processes (and nothing of the interface).
pub fn cpu_seconds() -> Option<f64> {
    let mut total = 0.0;
    for pid in descendants() {
        unsafe {
            let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
                continue;
            };
            let (mut created, mut exited, mut kernel, mut user) = (
                FILETIME::default(),
                FILETIME::default(),
                FILETIME::default(),
                FILETIME::default(),
            );
            if GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user).is_ok() {
                total += file_time_seconds(kernel) + file_time_seconds(user);
            }
            let _ = CloseHandle(process);
        }
    }
    Some(total)
}

/// Keeps the cap on engines that start later (a new isolated workspace has a browser process of its own).
fn keep_capped() {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    crate::error_log::ignored(
        std::thread::Builder::new()
            .name("resources-cpu-guard".into())
            .spawn(|| loop {
                std::thread::sleep(std::time::Duration::from_secs(5));
                if GUARD.load(Ordering::Relaxed) {
                    crate::error_log::ignored(assign_all());
                }
            }),
    );
}

pub fn apply(app: &AppHandle, limits: Limits, monitoring: bool) -> Result<(), String> {
    set_cap(limits.cpu_percent)?;
    GUARD.store(limits.cpu_percent > 0, Ordering::Relaxed);
    if limits.cpu_percent > 0 {
        assign_all()?;
        keep_capped();
    }
    for (label, view) in app.webviews() {
        if label.starts_with(crate::window::webview_manager::CONTENT_WEBVIEW_PREFIX) {
            attach(&view, limits, monitoring);
        }
    }
    Ok(())
}

// ---- the network ----

/// Pages already listened to, by the address of their engine object.
fn listened() -> &'static Mutex<HashSet<usize>> {
    static LISTENED: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();
    LISTENED.get_or_init(Default::default)
}

fn call(core: &ICoreWebView2, method: &str, parameters: &str) {
    let name = method.to_owned();
    let handler = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |result, _| {
        if let Err(error) = result {
            crate::error_log::warn(&format!("Developer protocol call {name} failed: {error}"));
        }
        Ok(())
    }));
    if let Err(error) = unsafe {
        core.CallDevToolsProtocolMethod(
            &HSTRING::from(method),
            &HSTRING::from(parameters),
            &handler,
        )
    } {
        crate::error_log::warn(&format!("Could not call {method}: {error}"));
    }
}

/// Sets the network speed of one page and starts or stops counting what it receives.
pub fn attach(view: &Webview, limits: Limits, monitoring: bool) {
    let needed = limits.network_kbps > 0 || monitoring;
    crate::error_log::ignored(view.with_webview(move |platform| unsafe {
        let Ok(core) = platform.controller().CoreWebView2() else {
            return;
        };
        let key = core.as_raw() as usize;
        if needed {
            call(&core, "Network.enable", "{}");
            if monitoring {
                let first = listened().lock().map(|mut set| set.insert(key)).unwrap_or(false);
                if first {
                    listen_for_data(&core);
                }
            }
        }
        // `-1` leaves a direction unlimited.
        let bytes = if limits.network_kbps > 0 {
            (u64::from(limits.network_kbps) * 1000 / 8).to_string()
        } else {
            "-1".into()
        };
        call(
            &core,
            "Network.emulateNetworkConditions",
            &format!(
                "{{\"offline\":false,\"latency\":0,\"downloadThroughput\":{bytes},\"uploadThroughput\":{bytes}}}"
            ),
        );
        if !needed {
            call(&core, "Network.disable", "{}");
        }
    }));
}

fn listen_for_data(core: &ICoreWebView2) {
    unsafe {
        let Ok(receiver) =
            core.GetDevToolsProtocolEventReceiver(&HSTRING::from("Network.dataReceived"))
        else {
            return;
        };
        let receiver: ICoreWebView2DevToolsProtocolEventReceiver = receiver;
        let mut token = 0;
        let handler =
            DevToolsProtocolEventReceivedEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    let mut text = windows::core::PWSTR::null();
                    if args.ParameterObjectAsJson(&mut text).is_ok() {
                        let json = CoTaskMemPWSTR::from(text).to_string();
                        if let Some(bytes) = encoded_length(&json) {
                            RECEIVED.fetch_add(bytes, Ordering::Relaxed);
                        }
                    }
                }
                Ok(())
            }));
        if let Err(error) = receiver.add_DevToolsProtocolEventReceived(&handler, &mut token) {
            crate::error_log::warn(&format!("Could not listen to the network: {error}"));
        }
    }
}

/// `encodedDataLength` of a `Network.dataReceived` event: the bytes that crossed the wire for that chunk.
fn encoded_length(json: &str) -> Option<u64> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    value.get("encodedDataLength")?.as_u64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wire_bytes_of_a_chunk_are_read_from_the_event() {
        assert_eq!(
            encoded_length(r#"{"requestId":"1","dataLength":900,"encodedDataLength":412}"#),
            Some(412)
        );
        assert_eq!(encoded_length(r#"{"requestId":"1"}"#), None);
        assert_eq!(encoded_length("not json"), None);
    }

    #[test]
    fn the_family_of_this_process_never_includes_itself() {
        assert!(!descendants().contains(&unsafe { GetCurrentProcessId() }));
    }
}
