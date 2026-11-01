use crate::state::browser_state::BrowserState;
use std::sync::mpsc::{self, Sender};
use tauri::{AppHandle, Emitter, Manager};

enum Write {
    Session,
    Visit { title: String, url: String },
    Flush(Sender<Result<(), String>>),
}

/// One ordered worker keeps SQLite I/O out of native WebView callbacks.
pub struct NavigationWriter(Sender<Write>);

impl NavigationWriter {
    pub fn start(app: AppHandle) -> Result<Self, String> {
        let (send, receive) = mpsc::channel();
        std::thread::Builder::new()
            .name("navigation-writer".into())
            .spawn(move || {
                let mut last_error = None;
                for write in receive {
                    match write {
                        Write::Session => {
                            let state = app.state::<BrowserState>();
                            let _operation = state.tab_operations.blocking_lock();
                            if let Err(error) = state.persist_session() {
                                last_error = Some(error);
                                let _ = app.emit("browser-error", "Could not persist the session");
                            }
                        }
                        Write::Visit { title, url } => {
                            let state = app.state::<BrowserState>();
                            // Snapshot only after serialized tab operations have finished.
                            let _operation = state.tab_operations.blocking_lock();
                            let result = state
                                .record_history_visit(&title, &url)
                                .and_then(|()| state.persist_session());
                            if let Err(error) = result {
                                last_error = Some(error);
                                let _ =
                                    app.emit("browser-error", "Could not persist navigation data");
                            }
                        }
                        Write::Flush(reply) => {
                            let _ = reply.send(last_error.take().map_or(Ok(()), Err));
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self(send))
    }

    pub fn visit(&self, title: String, url: String) -> Result<(), String> {
        self.0
            .send(Write::Visit { title, url })
            .map_err(|e| e.to_string())
    }

    pub fn session(&self) -> Result<(), String> {
        self.0.send(Write::Session).map_err(|e| e.to_string())
    }

    /// Call off the UI thread; acknowledges all earlier writes and their failures.
    pub fn flush(&self) -> Result<(), String> {
        let (send, receive) = mpsc::channel();
        self.0.send(Write::Flush(send)).map_err(|e| e.to_string())?;
        receive.recv().map_err(|e| e.to_string())?
    }
}
