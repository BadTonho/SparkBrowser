//! Tab memory measurement for active WebViews on Windows.

#[cfg(windows)]
use webview2_com::Microsoft::Web::WebView2::Win32::*;
#[cfg(windows)]
use windows_core::Interface;

#[cfg(windows)]
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
    fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    fn K32GetProcessMemoryInfo(
        process: *mut std::ffi::c_void,
        counters: *mut ProcessMemoryCounters,
        cb: u32,
    ) -> i32;
}

#[cfg(windows)]
pub fn get_process_working_set_bytes(pid: u32) -> Option<u64> {
    if pid == 0 {
        return None;
    }
    unsafe {
        let handle = OpenProcess(0x0400 | 0x0010, 0, pid);
        if handle.is_null() {
            let handle_lim = OpenProcess(0x1000, 0, pid);
            if handle_lim.is_null() {
                return None;
            }
            let mut counters: ProcessMemoryCounters = std::mem::zeroed();
            counters.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;
            let success = K32GetProcessMemoryInfo(handle_lim, &mut counters, counters.cb);
            CloseHandle(handle_lim);
            return if success != 0 {
                Some(counters.working_set_size as u64)
            } else {
                None
            };
        }
        let mut counters: ProcessMemoryCounters = std::mem::zeroed();
        counters.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;
        let success = K32GetProcessMemoryInfo(handle, &mut counters, counters.cb);
        CloseHandle(handle);
        if success != 0 {
            Some(counters.working_set_size as u64)
        } else {
            None
        }
    }
}

/// Measures memory for a specific webview on Windows.
#[cfg(windows)]
pub fn measure_webview_memory(view: &tauri::Webview, resident_count: usize) -> Option<u64> {
    let (tx, rx) = std::sync::mpsc::channel();
    let res = view.with_webview(move |platform| {
        let mut target_pid = 0u32;
        let mut is_renderer = false;
        if let Ok(core) = unsafe { platform.controller().CoreWebView2() } {
            if let Ok(core2) = core.cast::<ICoreWebView2_2>() {
                if let Ok(env) = unsafe { core2.Environment() } {
                    if let Ok(env8) = env.cast::<ICoreWebView2Environment8>() {
                        if let Ok(col) = unsafe { env8.GetProcessInfos() } {
                            let mut count = 0u32;
                            if unsafe { col.Count(&mut count) }.is_ok() {
                                for i in 0..count {
                                    if let Ok(info) = unsafe { col.GetValueAtIndex(i) } {
                                        let mut kind = COREWEBVIEW2_PROCESS_KIND_BROWSER;
                                        let mut p: i32 = 0;
                                        if unsafe { info.Kind(&mut kind) }.is_ok()
                                            && unsafe { info.ProcessId(&mut p) }.is_ok()
                                        {
                                            if kind == COREWEBVIEW2_PROCESS_KIND_RENDERER && p > 0 {
                                                target_pid = p as u32;
                                                is_renderer = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if target_pid == 0 {
                let mut browser_pid = 0u32;
                if unsafe { core.BrowserProcessId(&mut browser_pid) }.is_ok() {
                    target_pid = browser_pid;
                    is_renderer = false;
                }
            }
        }
        let _ = tx.send((target_pid, is_renderer));
    });

    if res.is_err() {
        return None;
    }

    let (target_pid, is_renderer) = rx
        .recv_timeout(std::time::Duration::from_millis(150))
        .unwrap_or((0, false));

    if target_pid > 0 {
        if let Some(bytes) = get_process_working_set_bytes(target_pid) {
            if is_renderer {
                Some(bytes)
            } else {
                Some(bytes / resident_count.max(1) as u64)
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(not(windows))]
pub fn measure_webview_memory(_view: &tauri::Webview, _resident_count: usize) -> Option<u64> {
    None
}
