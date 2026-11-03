use tauri::{AppHandle, Manager};

pub fn credential_key(app: &AppHandle) -> String {
    use sha2::{Digest, Sha256};
    let path = app.path().app_config_dir().unwrap_or_default();
    format!(
        "{}/gmail/{:x}",
        app.config().identifier,
        Sha256::digest(path.to_string_lossy().as_bytes())
    )
}
#[cfg(windows)]
pub fn read(key: &str) -> Result<Option<String>, String> {
    use windows::{core::PCWSTR, Win32::Security::Credentials::*};
    let name: Vec<u16> = key.encode_utf16().chain(Some(0)).collect();
    let mut ptr = std::ptr::null_mut();
    unsafe {
        if let Err(error) = CredReadW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None, &mut ptr) {
            return if error.code().0 as u32 == 0x80070490 {
                Ok(None)
            } else {
                Err("secure_storage".into())
            };
        }
        let data =
            std::slice::from_raw_parts((*ptr).CredentialBlob, (*ptr).CredentialBlobSize as usize)
                .to_vec();
        CredFree(ptr.cast());
        String::from_utf8(data)
            .map(Some)
            .map_err(|_| "secure_storage".into())
    }
}
#[cfg(windows)]
pub fn write(key: &str, value: &str) -> Result<(), String> {
    use windows::{core::PWSTR, Win32::Security::Credentials::*};
    let mut name: Vec<u16> = key.encode_utf16().chain(Some(0)).collect();
    let data = value.as_bytes();
    if data.len() > 2500 {
        return Err("secure_storage".into());
    }
    let credential = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(name.as_mut_ptr()),
        CredentialBlobSize: data.len() as u32,
        CredentialBlob: data.as_ptr() as *mut u8,
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        ..Default::default()
    };
    unsafe { CredWriteW(&credential, 0).map_err(|_| "secure_storage".into()) }
}
#[cfg(windows)]
pub fn delete(key: &str) -> Result<(), String> {
    use windows::{core::PCWSTR, Win32::Security::Credentials::*};
    let name: Vec<u16> = key.encode_utf16().chain(Some(0)).collect();
    unsafe {
        CredDeleteW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None)
            .or_else(|e| {
                if e.code().0 as u32 == 0x80070490 {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .map_err(|_| "secure_storage".into())
    }
}
#[cfg(not(windows))]
pub fn read(_key: &str) -> Result<Option<String>, String> {
    Ok(None)
}
#[cfg(not(windows))]
pub fn write(_key: &str, _value: &str) -> Result<(), String> {
    Err("secure_storage".into())
}
#[cfg(not(windows))]
pub fn delete(_key: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
pub async fn locate(app: &AppHandle) -> Result<(f64, f64), String> {
    use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator};
    let (send, receive) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || {
        let _ = send.send(Geolocator::RequestAccessAsync());
    })
    .map_err(|_| "location_error")?;
    let operation = receive
        .await
        .map_err(|_| "location_error")?
        .map_err(|_| "location_error")?;
    let status = operation.await.map_err(|_| "location_error")?;
    if status != GeolocationAccessStatus::Allowed {
        return Err("location_denied".into());
    }
    let locator = Geolocator::new().map_err(|_| "location_error")?;
    let operation = locator
        .GetGeopositionAsyncWithAgeAndTimeout(
            windows::Foundation::TimeSpan {
                Duration: 300_000_000,
            },
            windows::Foundation::TimeSpan {
                Duration: 100_000_000,
            },
        )
        .map_err(|_| "location_error")?;
    let position = operation.await.map_err(|_| "location_error")?;
    let coordinate = position.Coordinate().map_err(|_| "location_error")?;
    let latitude = coordinate.Latitude().map_err(|_| "location_error")?;
    let longitude = coordinate.Longitude().map_err(|_| "location_error")?;
    Ok((
        (latitude * 100.0).round() / 100.0,
        (longitude * 100.0).round() / 100.0,
    ))
}
#[cfg(not(windows))]
pub async fn locate(_app: &AppHandle) -> Result<(f64, f64), String> {
    Err("location_unsupported".into())
}

pub fn open_authorization(url: &url::Url) -> Result<(), String> {
    // Never authenticate inside a content WebView. Edge also avoids routing an
    // OS-default browser association back into Veloce itself.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let path = [
            std::env::var_os("ProgramFiles(x86)"),
            std::env::var_os("ProgramFiles"),
        ]
        .into_iter()
        .flatten()
        .map(|p| std::path::PathBuf::from(p).join("Microsoft/Edge/Application/msedge.exe"))
        .find(|p| p.is_file())
        .ok_or("external_browser")?;
        std::process::Command::new(path)
            .arg(url.as_str())
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|_| "external_browser")?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = url;
        Err("secure_storage".into())
    }
}
