//! Per-tab renderer memory measurements for WebView2.

#[cfg(windows)]
use webview2_com::{
    CoTaskMemPWSTR, GetProcessExtendedInfosCompletedHandler, Microsoft::Web::WebView2::Win32::*,
};
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

#[cfg(windows)]
fn complete_once(
    completion: &std::sync::Arc<
        std::sync::Mutex<Option<tokio::sync::oneshot::Sender<Option<u64>>>>,
    >,
    value: Option<u64>,
) {
    if let Ok(mut slot) = completion.lock() {
        if let Some(sender) = slot.take() {
            let _ = sender.send(value);
        }
    }
}

#[cfg(windows)]
pub fn current_webview_source(view: &tauri::Webview) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let result = view.with_webview(move |platform| {
        let source = unsafe {
            platform.controller().CoreWebView2().and_then(|core| {
                let mut raw = Default::default();
                core.Source(&mut raw)?;
                Ok(CoTaskMemPWSTR::from(raw).to_string())
            })
        };
        let _ = tx.send(source.ok().filter(|value| !value.is_empty()));
    });
    if result.is_err() {
        return None;
    }
    rx.recv_timeout(std::time::Duration::from_millis(150))
        .ok()
        .flatten()
}

#[cfg(not(windows))]
pub fn current_webview_source(_view: &tauri::Webview) -> Option<String> {
    None
}

/// Measures the renderer for this WebView by matching its main-frame URL to
/// WebView2's per-process frame list. When one renderer hosts multiple tabs,
/// its working set is apportioned across those resident tabs.
#[cfg(windows)]
pub async fn measure_webview_memory(
    view: &tauri::Webview,
    target_source: String,
    resident_sources: Vec<String>,
) -> Option<u64> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let completion = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));
    let callback_completion = completion.clone();
    let result = view.with_webview(move |platform| unsafe {
        let core = match platform.controller().CoreWebView2() {
            Ok(core) => core,
            Err(_) => {
                complete_once(&callback_completion, None);
                return;
            }
        };
        let environment = match core
            .cast::<ICoreWebView2_2>()
            .and_then(|core| core.Environment())
            .and_then(|environment| environment.cast::<ICoreWebView2Environment13>())
        {
            Ok(environment) => environment,
            Err(_) => {
                complete_once(&callback_completion, None);
                return;
            }
        };
        let callback_completion = callback_completion.clone();
        let launch_completion = callback_completion.clone();
        let handler =
            GetProcessExtendedInfosCompletedHandler::create(Box::new(move |status, processes| {
                if status.is_err() {
                    complete_once(&callback_completion, None);
                    return Ok(());
                }
                let Some(processes) = processes else {
                    complete_once(&callback_completion, None);
                    return Ok(());
                };
                let mut count = 0u32;
                if processes.Count(&mut count).is_err() {
                    complete_once(&callback_completion, None);
                    return Ok(());
                }
                let mut measurement = None;
                for index in 0..count {
                    let Ok(process) = processes.GetValueAtIndex(index) else {
                        continue;
                    };
                    let Ok(info) = process.ProcessInfo() else {
                        continue;
                    };
                    let mut kind = COREWEBVIEW2_PROCESS_KIND_BROWSER;
                    let mut pid = 0i32;
                    if info.Kind(&mut kind).is_err()
                        || kind != COREWEBVIEW2_PROCESS_KIND_RENDERER
                        || info.ProcessId(&mut pid).is_err()
                        || pid <= 0
                    {
                        continue;
                    }
                    let Ok(frames) = process.AssociatedFrameInfos() else {
                        continue;
                    };
                    let Ok(iterator) = frames.GetIterator() else {
                        continue;
                    };
                    let mut main_sources = Vec::new();
                    loop {
                        let mut has_current = windows_core::BOOL(0);
                        if iterator.HasCurrent(&mut has_current).is_err() || !has_current.as_bool()
                        {
                            break;
                        }
                        if let Ok(frame) = iterator.GetCurrent() {
                            if let Ok(frame2) = frame.cast::<ICoreWebView2FrameInfo2>() {
                                let mut frame_kind = COREWEBVIEW2_FRAME_KIND_UNKNOWN;
                                if frame2.FrameKind(&mut frame_kind).is_ok()
                                    && frame_kind == COREWEBVIEW2_FRAME_KIND_MAIN_FRAME
                                {
                                    let mut raw_source = Default::default();
                                    if frame.Source(&mut raw_source).is_ok() {
                                        let source = CoTaskMemPWSTR::from(raw_source).to_string();
                                        if !source.is_empty() {
                                            main_sources.push(source);
                                        }
                                    }
                                }
                            }
                        }
                        let mut moved = windows_core::BOOL(0);
                        if iterator.MoveNext(&mut moved).is_err() || !moved.as_bool() {
                            break;
                        }
                    }
                    let Some(tab_count) = associated_resident_tab_count(
                        &target_source,
                        &resident_sources,
                        &main_sources,
                    ) else {
                        continue;
                    };
                    measurement = get_process_working_set_bytes(pid as u32)
                        .map(|bytes| bytes / tab_count as u64);
                    break;
                }
                complete_once(&callback_completion, measurement);
                Ok(())
            }));
        if environment.GetProcessExtendedInfos(&handler).is_err() {
            complete_once(&launch_completion, None);
        }
    });
    if result.is_err() {
        complete_once(&completion, None);
    }
    tokio::time::timeout(std::time::Duration::from_millis(500), rx)
        .await
        .ok()?
        .ok()?
}

#[cfg(not(windows))]
pub async fn measure_webview_memory(
    _view: &tauri::Webview,
    _target_source: String,
    _resident_sources: Vec<String>,
) -> Option<u64> {
    None
}

#[cfg(any(windows, test))]
fn associated_resident_tab_count(
    target: &str,
    resident_sources: &[String],
    main_frame_sources: &[String],
) -> Option<usize> {
    if !main_frame_sources.iter().any(|source| source == target) {
        return None;
    }
    Some(
        resident_sources
            .iter()
            .filter(|source| main_frame_sources.iter().any(|frame| frame == *source))
            .count()
            .max(1),
    )
}

#[cfg(test)]
mod tests {
    use super::associated_resident_tab_count;

    #[test]
    fn memory_lookup_only_matches_frames_for_the_requested_tab() {
        let resident = vec![
            "https://video.example/watch".into(),
            "spark://newtab".into(),
        ];
        let video_frames = vec!["https://video.example/watch".into()];
        assert_eq!(
            associated_resident_tab_count("https://video.example/watch", &resident, &video_frames),
            Some(1)
        );
        assert_eq!(
            associated_resident_tab_count("spark://newtab", &resident, &video_frames),
            None
        );
    }

    #[test]
    fn shared_renderer_memory_is_split_between_its_resident_tabs() {
        let same_site_tabs = vec![
            "https://video.example/watch/one".into(),
            "https://video.example/watch/two".into(),
            "https://other.example/".into(),
        ];
        let shared_renderer_frames = vec![
            "https://video.example/watch/one".into(),
            "https://video.example/watch/two".into(),
        ];
        assert_eq!(
            associated_resident_tab_count(
                "https://video.example/watch/one",
                &same_site_tabs,
                &shared_renderer_frames,
            ),
            Some(2)
        );
    }
}
