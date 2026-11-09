//! Device-local WebDAV credentials. SQLite contains no new credential values.
use rusqlite::Connection;
use sha2::{Digest, Sha256};

#[derive(Default)]
pub struct Credentials {
    #[cfg(test)]
    values: std::sync::Mutex<std::collections::HashMap<String, String>>,
    #[cfg(test)]
    pub unavailable: std::sync::atomic::AtomicBool,
}

pub fn key(conn: &Connection) -> String {
    format!(
        "org.veloce.browser/webdav/{:x}",
        Sha256::digest(conn.path().unwrap_or("memory").as_bytes())
    )
}

impl Credentials {
    pub fn read(&self, key: &str) -> Result<Option<String>, String> {
        #[cfg(test)]
        {
            self.available()?;
            Ok(self.values.lock().unwrap().get(key).cloned())
        }
        #[cfg(not(test))]
        {
            os::read(key)
        }
    }
    pub fn write(&self, key: &str, value: &str) -> Result<(), String> {
        #[cfg(test)]
        {
            self.available()?;
            self.values.lock().unwrap().insert(key.into(), value.into());
            Ok(())
        }
        #[cfg(not(test))]
        {
            os::write(key, value)
        }
    }
    pub fn delete(&self, key: &str) -> Result<(), String> {
        #[cfg(test)]
        {
            self.available()?;
            self.values.lock().unwrap().remove(key);
            Ok(())
        }
        #[cfg(not(test))]
        {
            os::delete(key)
        }
    }
    #[cfg(test)]
    fn available(&self) -> Result<(), String> {
        if self.unavailable.load(std::sync::atomic::Ordering::Relaxed) {
            Err("backup.sync_secure_storage_failed".into())
        } else {
            Ok(())
        }
    }
}

#[cfg(all(not(test), windows))]
mod os {
    use windows::{
        core::{PCWSTR, PWSTR},
        Win32::Security::Credentials::*,
    };
    const ERROR: &str = "backup.sync_secure_storage_failed";
    pub fn read(key: &str) -> Result<Option<String>, String> {
        let name: Vec<u16> = key.encode_utf16().chain(Some(0)).collect();
        let mut ptr = std::ptr::null_mut();
        unsafe {
            if let Err(error) = CredReadW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None, &mut ptr)
            {
                return if error.code().0 as u32 == 0x80070490 {
                    Ok(None)
                } else {
                    Err(ERROR.into())
                };
            }
            let bytes = if (*ptr).CredentialBlobSize == 0 {
                Vec::new()
            } else {
                std::slice::from_raw_parts(
                    (*ptr).CredentialBlob,
                    (*ptr).CredentialBlobSize as usize,
                )
                .to_vec()
            };
            CredFree(ptr.cast());
            String::from_utf8(bytes).map(Some).map_err(|_| ERROR.into())
        }
    }
    pub fn write(key: &str, value: &str) -> Result<(), String> {
        if value.len() > 5120 {
            return Err(ERROR.into());
        }
        let mut name: Vec<u16> = key.encode_utf16().chain(Some(0)).collect();
        let credential = CREDENTIALW {
            Type: CRED_TYPE_GENERIC,
            TargetName: PWSTR(name.as_mut_ptr()),
            CredentialBlobSize: value.len() as u32,
            CredentialBlob: value.as_ptr() as *mut u8,
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            ..Default::default()
        };
        unsafe { CredWriteW(&credential, 0).map_err(|_| ERROR.into()) }
    }
    pub fn delete(key: &str) -> Result<(), String> {
        let name: Vec<u16> = key.encode_utf16().chain(Some(0)).collect();
        unsafe {
            match CredDeleteW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None) {
                Ok(()) => Ok(()),
                Err(error) if error.code().0 as u32 == 0x80070490 => Ok(()),
                Err(_) => Err(ERROR.into()),
            }
        }
    }
}

#[cfg(all(not(test), target_os = "linux"))]
mod os {
    use std::{
        io::Write,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    const ERROR: &str = "backup.sync_secure_storage_failed";
    // libsecret's CLI talks to the user's Secret Service. Never invoke a shell
    // or put credential values in command-line arguments, logs or files.
    fn run(args: &[&str], input: Option<&str>) -> Result<std::process::Output, String> {
        let mut child = Command::new("secret-tool")
            .args(args)
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| ERROR)?;
        if let Some(value) = input {
            let mut stdin = child.stdin.take().ok_or(ERROR)?;
            if stdin.write_all(value.as_bytes()).is_err() {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ERROR.into());
            }
        }
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => return child.wait_with_output().map_err(|_| ERROR.into()),
                Ok(None) if started.elapsed() < Duration::from_secs(15) => {
                    std::thread::sleep(Duration::from_millis(25))
                }
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ERROR.into());
                }
            }
        }
    }
    pub fn read(key: &str) -> Result<Option<String>, String> {
        let result = run(
            &[
                "lookup",
                "application",
                "org.veloce.browser",
                "profile",
                key,
            ],
            None,
        )?;
        if result.status.code() == Some(1) {
            return Ok(None);
        }
        if !result.status.success() {
            return Err(ERROR.into());
        }
        // Store base64 so secret-tool's terminating newline cannot alter passwords.
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(result.stdout.trim_ascii())
            .map_err(|_| ERROR)?;
        String::from_utf8(bytes).map(Some).map_err(|_| ERROR.into())
    }
    pub fn write(key: &str, value: &str) -> Result<(), String> {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(value);
        let result = run(
            &[
                "store",
                "--label=Spark WebDAV",
                "application",
                "org.veloce.browser",
                "profile",
                key,
            ],
            Some(&encoded),
        )?;
        if result.status.success() {
            Ok(())
        } else {
            Err(ERROR.into())
        }
    }
    pub fn delete(key: &str) -> Result<(), String> {
        let result = run(
            &["clear", "application", "org.veloce.browser", "profile", key],
            None,
        )?;
        if result.status.success() || result.status.code() == Some(1) {
            Ok(())
        } else {
            Err(ERROR.into())
        }
    }
}

#[cfg(all(not(test), not(any(windows, target_os = "linux"))))]
mod os {
    pub fn read(_: &str) -> Result<Option<String>, String> {
        Err("backup.sync_secure_storage_failed".into())
    }
    pub fn write(_: &str, _: &str) -> Result<(), String> {
        Err("backup.sync_secure_storage_failed".into())
    }
    pub fn delete(_: &str) -> Result<(), String> {
        Err("backup.sync_secure_storage_failed".into())
    }
}
