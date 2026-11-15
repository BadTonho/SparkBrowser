use super::{creation_guard, Panel, Popover};
use std::time::Duration;
use tauri::{AppHandle, Manager};

const RETAIN_AFTER_USE: Duration = Duration::from_secs(30);

pub(super) fn schedule<P: Panel>(app: &AppHandle) {
    let service = app.state::<Popover<P>>();
    let generation = {
        let Ok(mut inner) = service.0.lock() else {
            return;
        };
        if inner.session.is_some() || inner.release_pending {
            return;
        }
        inner.idle_generation += 1;
        inner.release_pending = true;
        inner.idle_generation
    };

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(RETAIN_AFTER_USE).await;
        let _ = tauri::async_runtime::spawn_blocking(move || release::<P>(&app, generation)).await;
    });
}

fn release<P: Panel>(app: &AppHandle, generation: u64) {
    // Keep creation serialized with native close, including when a click arrives
    // after the timer has already claimed the idle view.
    let Ok(_creating) = creation_guard() else {
        return;
    };
    let service = app.state::<Popover<P>>();
    {
        let Ok(mut inner) = service.0.lock() else {
            return;
        };
        if !inner.release_pending || inner.idle_generation != generation || inner.session.is_some() {
            return;
        }
        inner.release_pending = false;
    }

    let (finished, wait) = std::sync::mpsc::channel();
    let main_app = app.clone();
    if app
        .run_on_main_thread(move || {
            if let Some(view) = main_app.get_webview(P::LABEL) {
                if let Err(error) = view.close() {
                    eprintln!("Could not release {}: {error}", P::LABEL);
                } else {
                    #[cfg(target_os = "linux")]
                    crate::window::linux_layout::remove_child(&main_app, P::LABEL);
                }
            }
            let _ = finished.send(());
        })
        .is_ok()
    {
        let _ = wait.recv();
    }
}
