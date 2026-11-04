//! Restart only the current executable, after the protected shutdown checkpoint.
//! The child waits for the previous process so the existing profile lock remains exclusive.
use tauri::AppHandle;

const PARENT: &str = "--veloce-runtime-restart-parent";

pub(super) fn launch(app: &AppHandle) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|_| "runtime.restart_failed")?;
    let mut args = Vec::new();
    let mut original = std::env::args_os().skip(1);
    while let Some(arg) = original.next() {
        if arg == PARENT {
            original.next();
        } else {
            args.push(arg);
        }
    }
    std::process::Command::new(executable)
        .args(args)
        .arg(PARENT)
        .arg(std::process::id().to_string())
        .spawn()
        .map_err(|_| "runtime.restart_failed")?;
    app.exit(0);
    Ok(())
}

pub fn wait_for_parent() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let Some(index) = args.iter().position(|arg| arg == PARENT) else {
        return Ok(());
    };
    let pid = args
        .get(index + 1)
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|pid| *pid > 0 && *pid != std::process::id())
        .ok_or("runtime.restart_failed")?;
    #[cfg(windows)]
    {
        use std::ffi::c_void;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
            fn WaitForSingleObject(handle: *mut c_void, milliseconds: u32) -> u32;
            fn CloseHandle(handle: *mut c_void) -> i32;
            fn GetLastError() -> u32;
        }
        unsafe {
            let handle = OpenProcess(0x00100000, 0, pid);
            if handle.is_null() {
                return if GetLastError() == 87 {
                    Ok(())
                } else {
                    Err("runtime.restart_failed".into())
                };
            }
            let result = WaitForSingleObject(handle, 15_000);
            CloseHandle(handle);
            if result != 0 {
                return Err("runtime.restart_failed".into());
            }
        }
    }
    #[cfg(not(windows))]
    let _ = pid;
    Ok(())
}
