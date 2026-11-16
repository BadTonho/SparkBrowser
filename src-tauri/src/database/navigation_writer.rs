use crate::state::browser_state::BrowserState;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Sender, SyncSender},
    Arc, Mutex,
};
use tauri::{AppHandle, Manager};

enum Write {
    Session,
    Visit { title: String, url: String },
    Flush(Sender<Result<(), String>>),
}

/// One ordered worker keeps SQLite I/O out of native WebView callbacks.
pub struct NavigationWriter {
    send: SyncSender<Write>,
    session_queued: Arc<AtomicBool>,
    paused: Mutex<bool>,
}

impl NavigationWriter {
    pub fn start(app: AppHandle) -> Result<Self, String> {
        let (send, receive) = mpsc::sync_channel(256);
        let session_queued = Arc::new(AtomicBool::new(false));
        let queued_for_worker = session_queued.clone();
        std::thread::Builder::new()
            .name("navigation-writer".into())
            .spawn(move || {
                let mut last_error = None;
                for write in receive {
                    match write {
                        Write::Session => {
                            queued_for_worker.store(false, Ordering::Release);
                            let state = app.state::<BrowserState>();
                            let _operation = state.session().blocking_operation();
                            if let Err(error) = state.persist_session() {
                                last_error = Some(error);
                                crate::error_log::ignored(crate::security::emit_shell(
                                    &app,
                                    "browser-error",
                                    "Could not persist the session",
                                ));
                            }
                        }
                        Write::Visit { title, url } => {
                            let state = app.state::<BrowserState>();
                            // Snapshot only after serialized tab operations have finished.
                            let _operation = state.session().blocking_operation();
                            let result = state
                                .record_history_visit(&title, &url)
                                .and_then(|()| state.persist_session());
                            if let Err(error) = result {
                                last_error = Some(error);
                                crate::error_log::ignored(crate::security::emit_shell(
                                    &app,
                                    "browser-error",
                                    "Could not persist navigation data",
                                ));
                            } else {
                                crate::window::new_tab::invalidate(&app);
                            }
                        }
                        Write::Flush(reply) => {
                            crate::error_log::ignored(
                                reply.send(last_error.take().map_or(Ok(()), Err)),
                            );
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            send,
            session_queued,
            paused: Mutex::new(false),
        })
    }

    pub fn visit(&self, title: String, url: String) -> Result<(), String> {
        let paused = self.paused.lock().map_err(|e| e.to_string())?;
        if *paused {
            return Ok(());
        }
        self.send
            .try_send(Write::Visit { title, url })
            .map_err(|e| e.to_string())
    }

    pub fn session(&self) -> Result<(), String> {
        let paused = self.paused.lock().map_err(|e| e.to_string())?;
        if *paused {
            return Ok(());
        }
        if self.session_queued.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        if let Err(error) = self.send.try_send(Write::Session) {
            self.session_queued.store(false, Ordering::Release);
            return Err(error.to_string());
        }
        Ok(())
    }

    /// Call off the UI thread; acknowledges all earlier writes and their failures.
    pub fn flush(&self) -> Result<(), String> {
        let (send, receive) = mpsc::channel();
        self.send
            .send(Write::Flush(send))
            .map_err(|e| e.to_string())?;
        receive.recv().map_err(|e| e.to_string())?
    }

    /// Stop accepting navigation writes before draining the queue for an installer exit.
    pub fn freeze(&self) -> Result<(), String> {
        *self.paused.lock().map_err(|e| e.to_string())? = true;
        if let Err(error) = self.flush() {
            self.resume();
            return Err(error);
        }
        Ok(())
    }
    pub fn resume(&self) {
        if let Ok(mut paused) = self.paused.lock() {
            *paused = false;
        }
        crate::error_log::ignored(self.session());
    }
}
