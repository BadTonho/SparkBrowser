//! Opening Spark a second time hands over to the running one. The profile can only be used by one
//! process, so the new process asks the running one for a new window and exits.
//!
//! The running process listens on a loopback port that only it knows, together with a random token;
//! both are saved in `instance.json` inside the profile folder. A process that cannot find a live
//! listener there knows nobody is running (the file of a crashed run is simply ignored).
use crate::error_log::{self, Level};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::Duration,
};
use tauri::{AppHandle, Manager};

const FILE: &str = "instance.json";
const NEW_WINDOW: &str = "new-window";
const REPLY_OK: &str = "ok";
const IO_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Serialize, Deserialize)]
struct Endpoint {
    port: u16,
    token: String,
}

fn file(directory: &Path) -> PathBuf {
    directory.join(FILE)
}

fn new_token() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Asks the running Spark to open a new window. `Ok(true)` when it did, `Ok(false)` when there is
/// no running Spark to talk to.
pub fn ask_running_instance(directory: &Path) -> Result<bool, String> {
    let Ok(text) = std::fs::read_to_string(file(directory)) else {
        return Ok(false);
    };
    let endpoint: Endpoint = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, endpoint.port));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, IO_TIMEOUT) else {
        return Ok(false);
    };
    stream
        .set_read_timeout(Some(IO_TIMEOUT))
        .and_then(|()| stream.set_write_timeout(Some(IO_TIMEOUT)))
        .map_err(|e| e.to_string())?;
    writeln!(stream, "{}\n{NEW_WINDOW}", endpoint.token).map_err(|e| e.to_string())?;
    let mut reply = String::new();
    BufReader::new(stream)
        .read_line(&mut reply)
        .map_err(|e| e.to_string())?;
    Ok(reply.trim() == REPLY_OK)
}

/// Starts listening for other launches. Call once the first window exists.
pub fn serve(app: &AppHandle, directory: &Path) -> Result<(), String> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|e| e.to_string())?;
    let token = new_token()?;
    let endpoint = Endpoint {
        port: listener.local_addr().map_err(|e| e.to_string())?.port(),
        token: token.clone(),
    };
    std::fs::write(
        file(directory),
        serde_json::to_vec(&endpoint).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let app = app.clone();
    std::thread::Builder::new()
        .name("spark-instance-link".into())
        .spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => handle(&app, &token, stream),
                    Err(error) => error_log::report(
                        Level::Warning,
                        "instance-link",
                        "Spark could not accept a launch from a second process",
                        &error.to_string(),
                    ),
                }
            }
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn handle(app: &AppHandle, token: &str, mut stream: TcpStream) {
    if let Err(error) = stream
        .set_read_timeout(Some(IO_TIMEOUT))
        .and_then(|()| stream.set_write_timeout(Some(IO_TIMEOUT)))
    {
        error_log::report(
            Level::Warning,
            "instance-link",
            "Spark could not set up a launch from a second process",
            &error.to_string(),
        );
        return;
    }
    let mut lines = BufReader::new(&stream).lines();
    let received_token = lines.next().and_then(Result::ok).unwrap_or_default();
    let command = lines.next().and_then(Result::ok).unwrap_or_default();
    drop(lines);
    if received_token != token || command != NEW_WINDOW {
        error_log::report(
            Level::Warning,
            "instance-link",
            "Spark refused a launch request with a wrong token or command",
            "",
        );
        return;
    }
    if crate::app_lock::is_locked(app) {
        // A locked profile opens no window for anyone: the lock screen comes forward instead.
        if let Some(window) = app.get_webview_window("main") {
            error_log::ignored(window.unminimize());
            error_log::ignored(window.set_focus());
        }
        error_log::ignored(writeln!(stream, "{REPLY_OK}"));
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match crate::window::browser_windows::open(&app, crate::state::tab_model::MAIN_WINDOW_ID)
            .await
        {
            Ok(id) => {
                if let Some(window) = app.get_webview_window(&id) {
                    error_log::ignored(window.unminimize());
                    error_log::ignored(window.set_focus());
                }
            }
            Err(error) => error_log::report(
                Level::Error,
                "instance-link",
                "Spark could not open the window asked for by a second launch",
                &error,
            ),
        }
    });
    error_log::ignored(writeln!(stream, "{REPLY_OK}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_running_spark_there_is_nobody_to_ask() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!ask_running_instance(dir.path()).unwrap());
        // A file left by a crashed run points to a port nobody listens on.
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let stale = Endpoint {
            port,
            token: "x".into(),
        };
        std::fs::write(file(dir.path()), serde_json::to_vec(&stale).unwrap()).unwrap();
        assert!(!ask_running_instance(dir.path()).unwrap());
    }

    #[test]
    fn tokens_are_random_and_hex() {
        let (a, b) = (new_token().unwrap(), new_token().unwrap());
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
    }
}
