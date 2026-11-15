//! Spotify credentials never enter SQLite, backups, or renderer storage.
use tauri::AppHandle;

fn key(app: &AppHandle) -> String {
    super::super::platform::credential_key(app).replacen("/gmail/", "/spotify/", 1)
}

#[cfg(windows)]
pub fn read(app: &AppHandle) -> Result<Option<String>, String> {
    super::super::platform::read(&key(app))
}
#[cfg(windows)]
pub fn write(app: &AppHandle, value: &str) -> Result<(), String> {
    super::super::platform::write(&key(app), value)
}
#[cfg(windows)]
pub fn delete(app: &AppHandle) -> Result<(), String> {
    super::super::platform::delete(&key(app))
}
#[cfg(windows)]
pub fn open(url: &url::Url) -> Result<(), String> {
    super::super::platform::open_authorization(url)
}

#[cfg(target_os = "linux")]
fn command(app: &AppHandle, operation: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new("secret-tool");
    cmd.arg(operation);
    if operation == "store" {
        cmd.arg("--label=Spark Spotify");
    }
    cmd.args([
        "application",
        "org.spark.browser.spotify",
        "profile",
        &key(app),
    ]);
    cmd
}
#[cfg(target_os = "linux")]
pub fn read(app: &AppHandle) -> Result<Option<String>, String> {
    let output = command(app, "lookup")
        .output()
        .map_err(|_| "secure_storage")?;
    if output.status.success() {
        let value = String::from_utf8(output.stdout).map_err(|_| "secure_storage")?;
        Ok(Some(value.trim_end().to_owned()))
    } else if output.status.code() == Some(1) && output.stderr.is_empty() {
        Ok(None)
    } else {
        Err("secure_storage".into())
    }
}
#[cfg(target_os = "linux")]
pub fn write(app: &AppHandle, value: &str) -> Result<(), String> {
    use std::{io::Write, process::Stdio};
    let mut child = command(app, "store")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "secure_storage")?;
    let written = child
        .stdin
        .take()
        .ok_or("secure_storage")?
        .write_all(value.as_bytes());
    let status = child.wait().map_err(|_| "secure_storage")?;
    written.map_err(|_| "secure_storage")?;
    if status.success() {
        Ok(())
    } else {
        Err("secure_storage".into())
    }
}
#[cfg(target_os = "linux")]
pub fn delete(app: &AppHandle) -> Result<(), String> {
    let output = command(app, "clear")
        .output()
        .map_err(|_| "secure_storage")?;
    if output.status.success() || (output.status.code() == Some(1) && output.stderr.is_empty()) {
        Ok(())
    } else {
        Err("secure_storage".into())
    }
}
#[cfg(target_os = "linux")]
pub fn open(url: &url::Url) -> Result<(), String> {
    // Explicit external browsers avoid sending OAuth back into Spark when it is the default.
    for binary in [
        "firefox",
        "chromium",
        "chromium-browser",
        "google-chrome",
        "microsoft-edge",
    ] {
        match std::process::Command::new(binary).arg(url.as_str()).spawn() {
            Ok(mut child) => {
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
                return Ok(());
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return Err("external_browser".into()),
        }
    }
    Err("external_browser".into())
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn read(_: &AppHandle) -> Result<Option<String>, String> {
    Err("secure_storage".into())
}
#[cfg(not(any(windows, target_os = "linux")))]
pub fn write(_: &AppHandle, _: &str) -> Result<(), String> {
    Err("secure_storage".into())
}
#[cfg(not(any(windows, target_os = "linux")))]
pub fn delete(_: &AppHandle) -> Result<(), String> {
    Err("secure_storage".into())
}
#[cfg(not(any(windows, target_os = "linux")))]
pub fn open(_: &url::Url) -> Result<(), String> {
    Err("external_browser".into())
}
