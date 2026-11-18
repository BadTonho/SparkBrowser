//! Call stacks of every thread of the process, taken when the main thread stops answering.
//!
//! A freeze has no error message, but the stacks show exactly where each thread is waiting (a lock,
//! a channel, a system call), which is what is needed to find a deadlock. Windows on x86-64 only;
//! elsewhere the freeze entry keeps its trail of recent actions.
//!
//! Threads are suspended one at a time and only long enough to read their registers and walk their
//! stack into a pre-allocated buffer, so nothing allocates while a thread is stopped. Names are
//! looked up after the thread runs again. With the program database (`.pdb`) next to the
//! executable (development builds), frames show function names; without it (a released build)
//! they show the module and an offset, which can be resolved later with that build's `.pdb`.

#[cfg(all(windows, target_arch = "x86_64"))]
mod imp {
    use std::ffi::c_void;
    use std::fmt::Write;
    use std::sync::Once;
    use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};
    use std::time::Duration;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::Diagnostics::Debug::{
        AddrModeFlat, CONTEXT, CONTEXT_FULL_AMD64, GetThreadContext, IMAGEHLP_MODULE64,
        STACKFRAME64, SYMBOL_INFO, StackWalk64, SymFromAddr, SymFunctionTableAccess64,
        SymGetModuleBase64, SymGetModuleInfo64, SymInitializeW, SymSetOptions,
    };
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows::Win32::System::Threading::{
        GetCurrentProcess, GetCurrentProcessId, GetCurrentThreadId, OpenThread, ResumeThread,
        SuspendThread, THREAD_GET_CONTEXT, THREAD_QUERY_INFORMATION, THREAD_SUSPEND_RESUME,
    };
    use windows::core::PCWSTR;

    const MAX_FRAMES: usize = 48;
    const MAX_THREADS: usize = 64;
    const NAME_BYTES: usize = 256;
    const SYMOPT_UNDNAME: u32 = 0x2;
    const IMAGE_FILE_MACHINE_AMD64: u32 = 0x8664;

    static MAIN_THREAD: AtomicU32 = AtomicU32::new(0);
    /// The thread being read right now, so a stuck capture can still let it run again.
    static SUSPENDED: AtomicIsize = AtomicIsize::new(0);

    // `StackWalk64` wants plain system-ABI callbacks; the bindings expose Rust functions.
    unsafe extern "system" fn table_access(process: HANDLE, address: u64) -> *mut c_void {
        unsafe { SymFunctionTableAccess64(process, address) }
    }
    unsafe extern "system" fn module_base(process: HANDLE, address: u64) -> u64 {
        unsafe { SymGetModuleBase64(process, address) }
    }

    pub fn remember_main_thread() {
        MAIN_THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
    }

    pub fn capture() -> String {
        let (send, receive) = std::sync::mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("error-log-stacks".into())
            .spawn(move || {
                let _ = send.send(capture_all());
            });
        if spawned.is_err() {
            return "Could not start the stack capture.".into();
        }
        match receive.recv_timeout(Duration::from_secs(6)) {
            Ok(text) => text,
            Err(_) => {
                // The capture is stuck while a thread is stopped: let that thread run again.
                let handle = SUSPENDED.swap(0, Ordering::SeqCst);
                if handle != 0 {
                    unsafe { ResumeThread(HANDLE(handle as *mut c_void)) };
                }
                "The stack capture did not finish in time.".into()
            }
        }
    }

    fn capture_all() -> String {
        let process = unsafe { GetCurrentProcess() };
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            SymSetOptions(SYMOPT_UNDNAME);
            let directory: Vec<u16> = std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|d| d.as_os_str().to_owned()))
                .map(|d| {
                    use std::os::windows::ffi::OsStrExt;
                    d.encode_wide().chain(Some(0)).collect()
                })
                .unwrap_or_default();
            let path = if directory.is_empty() {
                PCWSTR::null()
            } else {
                PCWSTR(directory.as_ptr())
            };
            let _ = SymInitializeW(process, path, true);
        });
        let own = unsafe { GetCurrentThreadId() };
        let mut stacks: Vec<(u32, Vec<u64>, &str)> = Vec::new();
        for id in thread_ids().into_iter().take(MAX_THREADS) {
            if id != own {
                let (frames, note) = walk(process, id);
                stacks.push((id, frames, note));
            }
        }
        let main = MAIN_THREAD.load(Ordering::Relaxed);
        let mut text = String::new();
        for (id, frames, note) in stacks {
            let _ = writeln!(
                text,
                "thread {id}{}:",
                if id == main { " (main)" } else { "" }
            );
            if frames.is_empty() {
                let _ = writeln!(text, "  (no stack: {note})");
            }
            for address in frames {
                let _ = writeln!(text, "  {}", symbol(process, address));
            }
        }
        text
    }

    fn thread_ids() -> Vec<u32> {
        let mut ids = Vec::new();
        unsafe {
            let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) else {
                return ids;
            };
            let pid = GetCurrentProcessId();
            let mut entry = THREADENTRY32 {
                dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
                ..Default::default()
            };
            if Thread32First(snapshot, &mut entry).is_ok() {
                loop {
                    if entry.th32OwnerProcessID == pid {
                        ids.push(entry.th32ThreadID);
                    }
                    if Thread32Next(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
        }
        ids
    }

    /// Return addresses of one thread, innermost first. Nothing allocates while it is suspended.
    fn walk(process: HANDLE, id: u32) -> (Vec<u64>, &'static str) {
        let mut note = "";
        let mut frames = Vec::with_capacity(MAX_FRAMES);
        unsafe {
            let Ok(thread) = OpenThread(
                THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION,
                false,
                id,
            ) else {
                return (frames, "the thread could not be opened");
            };
            SUSPENDED.store(thread.0 as isize, Ordering::SeqCst);
            if SuspendThread(thread) != u32::MAX {
                let mut context = CONTEXT {
                    ContextFlags: CONTEXT_FULL_AMD64,
                    ..Default::default()
                };
                if GetThreadContext(thread, &mut context).is_err() {
                    note = "its registers could not be read";
                } else {
                    let mut frame = STACKFRAME64::default();
                    frame.AddrPC.Offset = context.Rip;
                    frame.AddrPC.Mode = AddrModeFlat;
                    frame.AddrFrame.Offset = context.Rbp;
                    frame.AddrFrame.Mode = AddrModeFlat;
                    frame.AddrStack.Offset = context.Rsp;
                    frame.AddrStack.Mode = AddrModeFlat;
                    while frames.len() < MAX_FRAMES
                        && StackWalk64(
                            IMAGE_FILE_MACHINE_AMD64,
                            process,
                            thread,
                            &mut frame,
                            &mut context as *mut CONTEXT as *mut c_void,
                            None,
                            Some(table_access),
                            Some(module_base),
                            None,
                        )
                        .as_bool()
                    {
                        frames.push(frame.AddrPC.Offset);
                        if frame.AddrPC.Offset == 0 {
                            break;
                        }
                    }
                    if frames.is_empty() {
                        note = "the stack could not be walked";
                    }
                }
                ResumeThread(thread);
            } else {
                note = "the thread could not be paused";
            }
            SUSPENDED.store(0, Ordering::SeqCst);
            let _ = CloseHandle(thread);
        }
        (frames, note)
    }

    fn symbol(process: HANDLE, address: u64) -> String {
        unsafe {
            // SYMBOL_INFO is followed by the name; u64 storage keeps it aligned.
            let size = std::mem::size_of::<SYMBOL_INFO>() + NAME_BYTES;
            let mut buffer = vec![0u64; size.div_ceil(8)];
            let info = buffer.as_mut_ptr() as *mut SYMBOL_INFO;
            (*info).SizeOfStruct = std::mem::size_of::<SYMBOL_INFO>() as u32;
            (*info).MaxNameLen = NAME_BYTES as u32 - 1;
            let mut displacement = 0u64;
            if SymFromAddr(process, address, Some(&mut displacement), info).is_ok() {
                let name = std::slice::from_raw_parts(
                    (*info).Name.as_ptr() as *const u8,
                    (*info).NameLen as usize,
                );
                return format!("{}+0x{displacement:x}", String::from_utf8_lossy(name));
            }
            let mut module = IMAGEHLP_MODULE64 {
                SizeOfStruct: std::mem::size_of::<IMAGEHLP_MODULE64>() as u32,
                ..Default::default()
            };
            if SymGetModuleInfo64(process, address, &mut module).is_ok() {
                let name: Vec<u8> = module
                    .ModuleName
                    .iter()
                    .take_while(|c| **c != 0)
                    .map(|c| *c as u8)
                    .collect();
                return format!(
                    "{}+0x{:x}",
                    String::from_utf8_lossy(&name),
                    address.saturating_sub(module.BaseOfImage)
                );
            }
            format!("0x{address:x}")
        }
    }
}

#[cfg(all(windows, target_arch = "x86_64"))]
pub use imp::{capture, remember_main_thread};

#[cfg(not(all(windows, target_arch = "x86_64")))]
pub fn remember_main_thread() {}

#[cfg(not(all(windows, target_arch = "x86_64")))]
pub fn capture() -> String {
    "Call stacks are only captured on Windows (x86-64); see the trail above.".into()
}

#[cfg(all(test, windows, target_arch = "x86_64"))]
mod tests {
    #[test]
    fn a_waiting_thread_shows_up_with_a_stack() {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        let worker = std::thread::spawn(move || {
            ready_tx.send(()).unwrap();
            let _ = stop_rx.recv();
        });
        ready_rx.recv().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let text = super::capture();
        stop_tx.send(()).unwrap();
        worker.join().unwrap();
        assert!(text.contains("thread "), "{text}");
        assert!(
            text.lines()
                .filter(|line| line.starts_with("  ") && !line.contains("no stack"))
                .count()
                > 3,
            "{text}"
        );
        assert!(!text.contains("no stack"), "{text}");
    }
}
