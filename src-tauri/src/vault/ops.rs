//! Vault operations on a database connection. The Tauri commands are thin wrappers around these, so
//! the rules (locking, the brake on wrong guesses, validation, what is logged) are tested without an app.
//!
//! Nothing here logs a password, a recovery key, an entry or a derived key; logs say what failed and where.
use super::crypto::{self, CryptoError, Header, Key};
use super::model::{
    valid_id, EntryData, Revealed, Status, Summary, AUTO_LOCK_CHOICES, DEFAULT_AUTO_LOCK_MINUTES,
    MAX_ENTRIES,
};
use super::service::VaultService;
use crate::database::vault_repo as repo;
use crate::error_log::{self, Level};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashSet;
use std::time::Instant;
use zeroize::{Zeroize, Zeroizing};

#[derive(Debug, Serialize, PartialEq)]
pub struct Listing {
    pub entries: Vec<Summary>,
    /// Entries that could not be decrypted (damaged data); the rest still open.
    pub unreadable: usize,
}

/// What an import did with the rows of a file.
#[derive(Debug, Serialize, PartialEq)]
pub struct ImportResult {
    pub imported: usize,
    /// Already in the vault (same address, user name and password).
    pub duplicates: usize,
    /// Rows left out: unreadable, or the vault is full.
    pub skipped: usize,
}

fn fail(error: CryptoError, place: &str) -> String {
    let level = match error {
        CryptoError::Corrupt | CryptoError::Unavailable => Level::Error,
        _ => Level::Warning,
    };
    error_log::report(
        level,
        "vault",
        &format!("{place} failed: {}", error.code()),
        "",
    );
    error.code().into()
}

fn header(conn: &Connection) -> Result<Header, String> {
    repo::load_header(conn)?.ok_or_else(|| "vault.not_initialized".to_string())
}

fn brake(service: &VaultService, now: Instant) -> Result<(), String> {
    service
        .check_attempt(now)
        .map_err(|seconds| format!("vault.throttled:{seconds}"))
}

pub fn status(conn: &Connection, service: &VaultService, now: Instant) -> Result<Status, String> {
    let initialized = repo::load_header(conn)?.is_some();
    let header = repo::load_header(conn)?;
    Ok(Status {
        initialized,
        has_password: header.as_ref().is_some_and(|h| h.password.is_some()),
        has_hello: header.as_ref().is_some_and(|h| h.hello.is_some()),
        // Whether this computer offers Windows Hello is the caller's to say (it needs the operating system).
        hello_available: false,
        require_hello: initialized && repo::require_hello(conn)?,
        unlocked: initialized && service.is_unlocked(now),
        entry_count: if initialized { repo::count(conn)? } else { 0 },
        auto_lock_minutes: repo::auto_lock_minutes(conn)?.unwrap_or(DEFAULT_AUTO_LOCK_MINUTES),
        retry_after_seconds: service.retry_after_seconds(now),
    })
}

/// Creates the vault and leaves it unlocked. Returns the recovery key, which is never stored.
/// It opens with the master password, with Windows Hello (`hello_secret`) or with both.
pub fn create(
    conn: &Connection,
    service: &VaultService,
    master_password: Option<&str>,
    hello_secret: Option<&[u8]>,
    now: Instant,
) -> Result<Zeroizing<String>, String> {
    if repo::load_header(conn)?.is_some() {
        return Err("vault.already_initialized".into());
    }
    let (header, key, recovery) =
        crypto::create(master_password, hello_secret).map_err(|e| fail(e, "creating the vault"))?;
    repo::create(conn, &header, DEFAULT_AUTO_LOCK_MINUTES)?;
    service.set_auto_lock_minutes(DEFAULT_AUTO_LOCK_MINUTES);
    service.unlock(key, now);
    error_log::report(Level::Info, "vault", "the password vault was created", "");
    Ok(Zeroizing::new(recovery))
}

fn unlock_with(
    conn: &Connection,
    service: &VaultService,
    now: Instant,
    how: &str,
    open: impl FnOnce(&Header) -> Result<Key, CryptoError>,
) -> Result<(), String> {
    brake(service, now)?;
    let header = header(conn)?;
    match open(&header) {
        Ok(key) => {
            service.set_auto_lock_minutes(
                repo::auto_lock_minutes(conn)?.unwrap_or(DEFAULT_AUTO_LOCK_MINUTES),
            );
            service.unlock(key, now);
            Ok(())
        }
        Err(error) => {
            if error == CryptoError::WrongSecret {
                service.failure(now);
            }
            Err(fail(error, &format!("unlocking the vault with {how}")))
        }
    }
}

pub fn unlock(
    conn: &Connection,
    service: &VaultService,
    password: &str,
    now: Instant,
) -> Result<(), String> {
    unlock_with(conn, service, now, "the password", |h| {
        crypto::unlock_with_password(h, password)
    })
}

/// Opens the vault with the secret Windows Hello produced. The operating system already made the person
/// prove who they are, so a failure here is not a wrong guess and does not count toward the brake.
pub fn unlock_with_hello(
    conn: &Connection,
    service: &VaultService,
    secret: &[u8],
    now: Instant,
) -> Result<(), String> {
    let header = header(conn)?;
    match crypto::unlock_with_hello(&header, secret) {
        Ok(key) => {
            service.set_auto_lock_minutes(
                repo::auto_lock_minutes(conn)?.unwrap_or(DEFAULT_AUTO_LOCK_MINUTES),
            );
            service.unlock(key, now);
            Ok(())
        }
        Err(error) => Err(hello_failure(
            error,
            "unlocking the vault with Windows Hello",
        )),
    }
}

/// A Windows Hello secret that does not open the vault is not a wrong guess (Windows controls who can
/// produce it), so it is reported as its own failure and never feeds the brake.
fn hello_failure(error: CryptoError, place: &str) -> String {
    let error = if error == CryptoError::WrongSecret {
        CryptoError::Unavailable
    } else {
        error
    };
    error_log::report(
        Level::Warning,
        "vault",
        &format!("{place} failed: the Windows Hello key no longer opens it"),
        error.code(),
    );
    match error {
        CryptoError::Unavailable => "vault.hello_failed".to_string(),
        other => other.code().to_string(),
    }
}

/// True when Windows Hello must be confirmed again before a password is shown, copied or exported.
pub fn requires_hello(conn: &Connection) -> Result<bool, String> {
    repo::require_hello(conn)
}

/// Checks a Windows Hello secret against the vault without opening it.
pub fn confirm_hello(conn: &Connection, secret: &[u8]) -> Result<(), String> {
    crypto::unlock_with_hello(&header(conn)?, secret)
        .map(|_| ())
        .map_err(|e| hello_failure(e, "confirming Windows Hello"))
}

/// Refuses the action when the vault asks for Windows Hello each time and no valid proof came with it.
pub fn confirm_if_required(conn: &Connection, proof: Option<&[u8]>) -> Result<(), String> {
    if !repo::require_hello(conn)? {
        return Ok(());
    }
    match proof {
        Some(secret) => confirm_hello(conn, secret),
        None => Err("vault.hello_required".into()),
    }
}

/// Turns "ask for Windows Hello each time" on or off. It needs Windows Hello to be one of the ways to
/// unlock, and changing it while it is on needs a fresh Windows Hello confirmation.
pub fn set_require_hello(
    conn: &Connection,
    service: &VaultService,
    on: bool,
    proof: Option<&[u8]>,
    now: Instant,
) -> Result<(), String> {
    service.key(now)?;
    if on && header(conn)?.hello.is_none() {
        return Err("vault.method_unavailable".into());
    }
    confirm_if_required(conn, proof)?;
    repo::set_require_hello(conn, on)?;
    error_log::report(
        Level::Info,
        "vault",
        if on {
            "Windows Hello is now asked for every password shown or copied"
        } else {
            "Windows Hello is no longer asked for every password shown or copied"
        },
        "",
    );
    Ok(())
}

/// Checks the master password of an open vault (it counts toward the brake on wrong guesses).
pub fn verify_password(
    conn: &Connection,
    service: &VaultService,
    current: &str,
    now: Instant,
) -> Result<Header, String> {
    service.key(now)?;
    brake(service, now)?;
    let header = header(conn)?;
    if let Err(error) = crypto::unlock_with_password(&header, current) {
        if error == CryptoError::WrongSecret {
            service.failure(now);
        }
        return Err(fail(error, "confirming the current vault password"));
    }
    Ok(header)
}

pub fn unlock_with_recovery(
    conn: &Connection,
    service: &VaultService,
    recovery_key: &str,
    now: Instant,
) -> Result<(), String> {
    unlock_with(conn, service, now, "the recovery key", |h| {
        crypto::unlock_with_recovery(h, recovery_key)
    })
}

pub fn list(conn: &Connection, service: &VaultService, now: Instant) -> Result<Listing, String> {
    let key = service.key(now)?;
    let mut entries = Vec::new();
    let mut unreadable = 0;
    for row in repo::rows(conn)? {
        match crypto::open_entry(&key, &row.id, &row.blob)
            .ok()
            .and_then(|plain| serde_json::from_slice::<EntryData>(&plain).ok())
        {
            Some(data) => entries.push(data.summary(&row.id, row.updated_at)),
            None => {
                unreadable += 1;
                error_log::report(
                    Level::Error,
                    "vault",
                    "a vault entry could not be decrypted",
                    &format!("entry id {}", row.id),
                );
            }
        }
    }
    entries.sort_by_key(|entry| entry.title.to_lowercase());
    Ok(Listing {
        entries,
        unreadable,
    })
}

/// What the person used to prove who they are before a password is shown.
pub enum Proof<'a> {
    /// The master password, typed.
    Password(&'a str),
    /// The secret Windows Hello produced after PIN, fingerprint or face.
    Hello(&'a [u8]),
}

/// Checks a proof before a password is shown. When the vault asks for Windows Hello each time, a typed
/// master password is not enough. A wrong master password counts toward the brake.
pub fn confirm_reveal(
    conn: &Connection,
    service: &VaultService,
    proof: Proof<'_>,
    now: Instant,
) -> Result<(), String> {
    match proof {
        Proof::Password(password) => {
            if repo::require_hello(conn)? {
                return Err("vault.hello_required".into());
            }
            verify_password(conn, service, password, now).map(|_| ())
        }
        Proof::Hello(secret) => {
            service.key(now)?;
            confirm_hello(conn, secret)
        }
    }
}

/// Shows one entry in full, but only after the person proved who they are (every time, not once per unlock).
pub fn reveal_confirmed(
    conn: &Connection,
    service: &VaultService,
    id: &str,
    proof: Proof<'_>,
    now: Instant,
) -> Result<Revealed, String> {
    confirm_reveal(conn, service, proof, now)?;
    reveal(conn, service, id, now)
}

pub fn reveal(
    conn: &Connection,
    service: &VaultService,
    id: &str,
    now: Instant,
) -> Result<Revealed, String> {
    let key = service.key(now)?;
    if !valid_id(id) {
        return Err("vault.not_found".into());
    }
    let row = repo::row(conn, id)?.ok_or("vault.not_found")?;
    let plain = crypto::open_entry(&key, &row.id, &row.blob)
        .map_err(|e| fail(e, "reading a vault entry"))?;
    let data: EntryData = serde_json::from_slice(&plain)
        .map_err(|_| fail(CryptoError::Corrupt, "reading a vault entry"))?;
    Ok(Revealed {
        id: row.id,
        data,
        updated_at: row.updated_at,
    })
}

/// Only the password of one entry, for copying it from the backend (it never reaches the interface).
pub fn password(
    conn: &Connection,
    service: &VaultService,
    id: &str,
    now: Instant,
) -> Result<Zeroizing<String>, String> {
    let mut revealed = reveal(conn, service, id, now)?;
    let password = Zeroizing::new(std::mem::take(&mut revealed.data.password));
    revealed.data.notes.zeroize();
    Ok(password)
}

fn new_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|_| fail(CryptoError::Unavailable, "creating an entry id"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Adds an entry (`id` is None) or replaces one.
pub fn save(
    conn: &Connection,
    service: &VaultService,
    id: Option<&str>,
    data: &EntryData,
    timestamp: i64,
    now: Instant,
) -> Result<Summary, String> {
    let key = service.key(now)?;
    data.validate()?;
    let id = match id {
        Some(id) => {
            if !valid_id(id) || repo::row(conn, id)?.is_none() {
                return Err("vault.not_found".into());
            }
            id.to_string()
        }
        None => {
            if repo::count(conn)? >= MAX_ENTRIES {
                return Err("vault.full".into());
            }
            new_id()?
        }
    };
    let plain = Zeroizing::new(serde_json::to_vec(data).map_err(|e| e.to_string())?);
    let blob =
        crypto::seal_entry(&key, &id, &plain).map_err(|e| fail(e, "saving a vault entry"))?;
    repo::put(conn, &id, &blob, timestamp)?;
    Ok(data.summary(&id, timestamp))
}

pub fn delete(
    conn: &Connection,
    service: &VaultService,
    id: &str,
    now: Instant,
) -> Result<(), String> {
    service.key(now)?;
    if !valid_id(id) || !repo::delete(conn, id)? {
        return Err("vault.not_found".into());
    }
    Ok(())
}

/// Changes the master password. The current one is asked again even though the vault is open, and
/// counts toward the brake on wrong guesses. Entries and the recovery key are unaffected.
pub fn change_password(
    conn: &Connection,
    service: &VaultService,
    current: &str,
    new_password: &str,
    now: Instant,
) -> Result<(), String> {
    service.key(now)?;
    brake(service, now)?;
    let header = header(conn)?;
    let key = match crypto::unlock_with_password(&header, current) {
        Ok(key) => key,
        Err(error) => {
            if error == CryptoError::WrongSecret {
                service.failure(now);
            }
            return Err(fail(error, "confirming the current vault password"));
        }
    };
    let updated = crypto::change_password(&header, &key, new_password)
        .map_err(|e| fail(e, "changing the vault password"))?;
    repo::update_header(conn, &updated)?;
    error_log::report(
        Level::Info,
        "vault",
        "the vault master password was changed",
        "",
    );
    Ok(())
}

/// Turns Windows Hello on (or replaces its key) for an open vault. The master password is unaffected.
pub fn enable_hello(
    conn: &Connection,
    service: &VaultService,
    secret: &[u8],
    now: Instant,
) -> Result<(), String> {
    let key = service.key(now)?;
    let updated = crypto::set_hello(&header(conn)?, &key, secret)
        .map_err(|e| fail(e, "turning on Windows Hello"))?;
    repo::update_header(conn, &updated)?;
    error_log::report(
        Level::Info,
        "vault",
        "Windows Hello was turned on for the vault",
        "",
    );
    Ok(())
}

/// Turns Windows Hello off. Refused when it is the only way in besides the recovery key.
pub fn disable_hello(
    conn: &Connection,
    service: &VaultService,
    now: Instant,
) -> Result<(), String> {
    service.key(now)?;
    let updated =
        crypto::remove_hello(&header(conn)?).map_err(|e| fail(e, "turning off Windows Hello"))?;
    repo::update_header(conn, &updated)?;
    // Without Windows Hello there is nothing to ask for each time.
    repo::set_require_hello(conn, false)?;
    error_log::report(
        Level::Info,
        "vault",
        "Windows Hello was turned off for the vault",
        "",
    );
    Ok(())
}

/// Turns the master password on, for a vault that only opened with Windows Hello.
pub fn enable_password(
    conn: &Connection,
    service: &VaultService,
    new_password: &str,
    now: Instant,
) -> Result<(), String> {
    let key = service.key(now)?;
    let header = header(conn)?;
    if header.password.is_some() {
        return Err("vault.method_already_on".into());
    }
    let updated = crypto::change_password(&header, &key, new_password)
        .map_err(|e| fail(e, "turning on the vault master password"))?;
    repo::update_header(conn, &updated)?;
    error_log::report(
        Level::Info,
        "vault",
        "the vault master password was turned on",
        "",
    );
    Ok(())
}

/// Turns the master password off. The current one is asked again, and Windows Hello must remain.
pub fn disable_password(
    conn: &Connection,
    service: &VaultService,
    current: &str,
    now: Instant,
) -> Result<(), String> {
    let header = verify_password(conn, service, current, now)?;
    let updated = crypto::remove_password(&header)
        .map_err(|e| fail(e, "turning off the vault master password"))?;
    repo::update_header(conn, &updated)?;
    error_log::report(
        Level::Info,
        "vault",
        "the vault master password was turned off",
        "",
    );
    Ok(())
}

/// Adds the entries of an import. Ones already in the vault are not added twice, and the vault never
/// grows past its limit. All of it is stored or none of it.
pub fn import_entries(
    conn: &Connection,
    service: &VaultService,
    entries: Vec<EntryData>,
    timestamp: i64,
    now: Instant,
) -> Result<ImportResult, String> {
    let key = service.key(now)?;
    let identity = |entry: &EntryData| {
        (
            entry.url.to_lowercase(),
            entry.username.to_lowercase(),
            entry.password.clone(),
        )
    };
    let mut known: HashSet<(String, String, String)> = HashSet::new();
    for row in repo::rows(conn)? {
        if let Some(data) = crypto::open_entry(&key, &row.id, &row.blob)
            .ok()
            .and_then(|plain| serde_json::from_slice::<EntryData>(&plain).ok())
        {
            known.insert(identity(&data));
        }
    }
    let mut count = repo::count(conn)?;
    let mut result = ImportResult {
        imported: 0,
        duplicates: 0,
        skipped: 0,
    };
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    for entry in entries {
        if entry.validate().is_err() {
            result.skipped += 1;
        } else if !known.insert(identity(&entry)) {
            result.duplicates += 1;
        } else if count >= MAX_ENTRIES {
            result.skipped += 1;
        } else {
            let id = new_id()?;
            let plain = Zeroizing::new(serde_json::to_vec(&entry).map_err(|e| e.to_string())?);
            let blob = crypto::seal_entry(&key, &id, &plain)
                .map_err(|e| fail(e, "importing vault entries"))?;
            repo::put(&tx, &id, &blob, timestamp)?;
            count += 1;
            result.imported += 1;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    error_log::report(
        Level::Info,
        "vault",
        &format!(
            "{} passwords were imported from a file ({} already there, {} left out)",
            result.imported, result.duplicates, result.skipped
        ),
        "",
    );
    Ok(result)
}

/// Every entry in plain text, for an export the person confirmed. The second value counts entries that
/// could not be decrypted (damaged data), which are left out.
pub fn export_entries(
    conn: &Connection,
    service: &VaultService,
    now: Instant,
) -> Result<(Vec<EntryData>, usize), String> {
    let key = service.key(now)?;
    let (mut entries, mut unreadable) = (Vec::new(), 0);
    for row in repo::rows(conn)? {
        match crypto::open_entry(&key, &row.id, &row.blob)
            .ok()
            .and_then(|plain| serde_json::from_slice::<EntryData>(&plain).ok())
        {
            Some(data) => entries.push(data),
            None => {
                unreadable += 1;
                error_log::report(
                    Level::Error,
                    "vault",
                    "a vault entry could not be decrypted for the export",
                    &format!("entry id {}", row.id),
                );
            }
        }
    }
    entries.sort_by_key(|entry| entry.title.to_lowercase());
    Ok((entries, unreadable))
}

pub fn set_auto_lock(
    conn: &Connection,
    service: &VaultService,
    minutes: u32,
    now: Instant,
) -> Result<(), String> {
    service.key(now)?;
    if !AUTO_LOCK_CHOICES.contains(&minutes) {
        return Err("vault.invalid_setting".into());
    }
    repo::set_auto_lock_minutes(conn, minutes)?;
    service.set_auto_lock_minutes(minutes);
    Ok(())
}

/// Starts over after both secrets were lost: every entry is deleted (they cannot be read anyway).
pub fn reset(conn: &Connection, service: &VaultService) -> Result<(), String> {
    repo::wipe(conn)?;
    service.lock();
    error_log::report(
        Level::Warning,
        "vault",
        "the password vault was erased and reset",
        "",
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    const PASSWORD: &str = "correct horse battery";

    fn setup() -> (Connection, VaultService, Instant) {
        let conn = Connection::open_in_memory().unwrap();
        repo::migrate(&conn).unwrap();
        (conn, VaultService::default(), Instant::now())
    }

    fn entry(title: &str, password: &str) -> EntryData {
        EntryData {
            title: title.into(),
            url: "https://example.test".into(),
            username: "me@example.test".into(),
            password: password.into(),
            notes: "private note".into(),
        }
    }

    #[test]
    fn creating_unlocks_gives_a_recovery_key_and_cannot_be_repeated() {
        let (conn, service, now) = setup();
        assert!(!status(&conn, &service, now).unwrap().initialized);
        let recovery = create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        assert_eq!(recovery.len(), 39);
        let s = status(&conn, &service, now).unwrap();
        assert!(s.initialized && s.unlocked && s.entry_count == 0);
        assert_eq!(
            create(&conn, &service, Some(PASSWORD), None, now).unwrap_err(),
            "vault.already_initialized"
        );
        let (other, other_service, _) = setup();
        assert_eq!(
            create(&other, &other_service, Some("short"), None, now).unwrap_err(),
            "vault.password_invalid"
        );
        assert!(!status(&other, &other_service, now).unwrap().initialized);
    }

    #[test]
    fn entries_are_encrypted_at_rest_listed_without_secrets_and_revealed_on_request() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        let saved = save(
            &conn,
            &service,
            None,
            &entry("Bank", "hunter2-secret"),
            10,
            now,
        )
        .unwrap();
        save(&conn, &service, None, &entry("alpha", "p"), 11, now).unwrap();
        // Nothing readable in the database.
        for row in repo::rows(&conn).unwrap() {
            let text = String::from_utf8_lossy(&row.blob).to_string();
            assert!(
                !text.contains("hunter2")
                    && !text.contains("Bank")
                    && !text.contains("example.test")
            );
        }
        let listing = list(&conn, &service, now).unwrap();
        assert_eq!(
            listing
                .entries
                .iter()
                .map(|e| e.title.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "Bank"]
        );
        assert!(!serde_json::to_string(&listing).unwrap().contains("hunter2"));
        let shown = reveal(&conn, &service, &saved.id, now).unwrap();
        assert_eq!(shown.data.password, "hunter2-secret");
        assert_eq!(shown.data.notes, "private note");
    }

    #[test]
    fn saving_can_replace_an_entry_and_deleting_removes_it() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        let first = save(&conn, &service, None, &entry("Site", "one"), 1, now).unwrap();
        save(
            &conn,
            &service,
            Some(&first.id),
            &entry("Site", "two"),
            2,
            now,
        )
        .unwrap();
        assert_eq!(list(&conn, &service, now).unwrap().entries.len(), 1);
        assert_eq!(
            reveal(&conn, &service, &first.id, now)
                .unwrap()
                .data
                .password,
            "two"
        );
        assert_eq!(
            save(
                &conn,
                &service,
                Some(&"0".repeat(32)),
                &entry("x", "y"),
                3,
                now
            )
            .unwrap_err(),
            "vault.not_found"
        );
        assert_eq!(
            save(&conn, &service, None, &entry("  ", "y"), 3, now).unwrap_err(),
            "vault.invalid_entry"
        );
        delete(&conn, &service, &first.id, now).unwrap();
        assert_eq!(
            delete(&conn, &service, &first.id, now).unwrap_err(),
            "vault.not_found"
        );
        assert_eq!(
            reveal(&conn, &service, "nonsense", now).unwrap_err(),
            "vault.not_found"
        );
    }

    #[test]
    fn nothing_works_while_locked_and_unlocking_needs_the_right_secret() {
        let (conn, service, now) = setup();
        let recovery = create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        let saved = save(&conn, &service, None, &entry("Site", "pw"), 1, now).unwrap();
        service.lock();
        for result in [
            list(&conn, &service, now).map(|_| ()),
            reveal(&conn, &service, &saved.id, now).map(|_| ()),
            save(&conn, &service, None, &entry("x", "y"), 1, now).map(|_| ()),
            delete(&conn, &service, &saved.id, now),
            set_auto_lock(&conn, &service, 15, now),
            change_password(&conn, &service, PASSWORD, "another password", now),
        ] {
            assert_eq!(result.unwrap_err(), "vault.locked");
        }
        assert_eq!(
            unlock(&conn, &service, "wrong password", now).unwrap_err(),
            "vault.wrong_secret"
        );
        assert!(!status(&conn, &service, now).unwrap().unlocked);
        unlock(&conn, &service, PASSWORD, now).unwrap();
        assert_eq!(list(&conn, &service, now).unwrap().entries.len(), 1);
        service.lock();
        unlock_with_recovery(&conn, &service, &recovery, now).unwrap();
        assert!(status(&conn, &service, now).unwrap().unlocked);
    }

    #[test]
    fn repeated_wrong_guesses_are_slowed_down_even_for_the_right_password() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        service.lock();
        for _ in 0..3 {
            assert_eq!(
                unlock(&conn, &service, "nope nope nope", now).unwrap_err(),
                "vault.wrong_secret"
            );
        }
        let blocked = unlock(&conn, &service, PASSWORD, now).unwrap_err();
        assert!(blocked.starts_with("vault.throttled:"), "{blocked}");
        assert!(status(&conn, &service, now).unwrap().retry_after_seconds > 0);
        unlock(&conn, &service, PASSWORD, now + Duration::from_secs(5)).unwrap();
    }

    #[test]
    fn changing_the_master_password_needs_the_current_one_and_keeps_the_entries() {
        let (conn, service, now) = setup();
        let recovery = create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        let saved = save(&conn, &service, None, &entry("Site", "pw"), 1, now).unwrap();
        assert_eq!(
            change_password(
                &conn,
                &service,
                "not it at all",
                "a brand new password",
                now
            )
            .unwrap_err(),
            "vault.wrong_secret"
        );
        assert_eq!(
            change_password(&conn, &service, PASSWORD, "short", now).unwrap_err(),
            "vault.password_invalid"
        );
        change_password(&conn, &service, PASSWORD, "a brand new password", now).unwrap();
        service.lock();
        assert!(unlock(&conn, &service, PASSWORD, now).is_err());
        unlock(
            &conn,
            &service,
            "a brand new password",
            now + Duration::from_secs(120),
        )
        .unwrap();
        assert_eq!(
            reveal(&conn, &service, &saved.id, now + Duration::from_secs(120))
                .unwrap()
                .data
                .password,
            "pw"
        );
        service.lock();
        unlock_with_recovery(&conn, &service, &recovery, now + Duration::from_secs(120)).unwrap();
    }

    const HELLO: &[u8] =
        b"a signature produced by windows hello after the user proved who they are";

    #[test]
    fn a_vault_can_be_created_and_opened_with_windows_hello_alone() {
        let (conn, service, now) = setup();
        let recovery = create(&conn, &service, None, Some(HELLO), now).unwrap();
        let saved = save(&conn, &service, None, &entry("Site", "pw"), 1, now).unwrap();
        let s = status(&conn, &service, now).unwrap();
        assert!(s.has_hello && !s.has_password);
        service.lock();
        assert_eq!(
            unlock(&conn, &service, PASSWORD, now).unwrap_err(),
            "vault.method_unavailable"
        );
        unlock_with_hello(&conn, &service, HELLO, now).unwrap();
        assert_eq!(
            reveal(&conn, &service, &saved.id, now)
                .unwrap()
                .data
                .password,
            "pw"
        );
        service.lock();
        unlock_with_recovery(&conn, &service, &recovery, now).unwrap();
        assert_eq!(
            create(&conn, &service, None, None, now).unwrap_err(),
            "vault.already_initialized"
        );
        let (other, other_service, _) = setup();
        assert_eq!(
            create(&other, &other_service, None, None, now).unwrap_err(),
            "vault.last_method"
        );
    }

    #[test]
    fn a_failed_windows_hello_unlock_is_not_a_wrong_guess() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), Some(HELLO), now).unwrap();
        service.lock();
        for _ in 0..5 {
            assert_eq!(
                unlock_with_hello(
                    &conn,
                    &service,
                    b"another signature that is long enough",
                    now
                )
                .unwrap_err(),
                "vault.hello_failed"
            );
        }
        assert!(
            service.check_attempt(now).is_ok(),
            "no brake for Windows Hello failures"
        );
        unlock(&conn, &service, PASSWORD, now).unwrap();
    }

    #[test]
    fn the_methods_are_turned_on_and_off_independently_but_one_always_remains() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        let saved = save(&conn, &service, None, &entry("Site", "pw"), 1, now).unwrap();
        assert_eq!(
            disable_password(&conn, &service, PASSWORD, now).unwrap_err(),
            "vault.last_method"
        );
        disable_hello(&conn, &service, now).unwrap(); // already off: nothing to do, nothing lost
        enable_hello(&conn, &service, HELLO, now).unwrap();
        assert!(status(&conn, &service, now).unwrap().has_hello);
        assert_eq!(
            disable_password(&conn, &service, "not it at all", now).unwrap_err(),
            "vault.wrong_secret"
        );
        disable_password(&conn, &service, PASSWORD, now).unwrap();
        service.lock();
        assert!(unlock(&conn, &service, PASSWORD, now + Duration::from_secs(120)).is_err());
        unlock_with_hello(&conn, &service, HELLO, now).unwrap();
        assert_eq!(
            reveal(&conn, &service, &saved.id, now)
                .unwrap()
                .data
                .password,
            "pw"
        );
        assert_eq!(
            enable_password(&conn, &service, "short", now).unwrap_err(),
            "vault.password_invalid"
        );
        enable_password(&conn, &service, "a brand new password", now).unwrap();
        assert_eq!(
            enable_password(&conn, &service, "another password", now).unwrap_err(),
            "vault.method_already_on"
        );
        disable_hello(&conn, &service, now).unwrap();
        service.lock();
        assert!(unlock_with_hello(&conn, &service, HELLO, now).is_err());
        unlock(&conn, &service, "a brand new password", now).unwrap();
        enable_hello(&conn, &service, HELLO, now).unwrap();
        service.lock();
        for result in [
            enable_hello(&conn, &service, HELLO, now),
            disable_hello(&conn, &service, now),
            enable_password(&conn, &service, "a brand new password", now),
            disable_password(&conn, &service, "a brand new password", now),
        ] {
            assert_eq!(result.unwrap_err(), "vault.locked");
        }
    }

    #[test]
    fn showing_a_password_needs_the_master_password_or_windows_hello_every_time() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), Some(HELLO), now).unwrap();
        let saved = save(&conn, &service, None, &entry("Site", "pw"), 1, now).unwrap();
        // The vault is open, yet nothing is shown without a proof, and a wrong one is refused.
        let shown =
            reveal_confirmed(&conn, &service, &saved.id, Proof::Password(PASSWORD), now).unwrap();
        assert_eq!(shown.data.password, "pw");
        assert_eq!(
            reveal_confirmed(
                &conn,
                &service,
                &saved.id,
                Proof::Password("not it at all"),
                now
            )
            .unwrap_err(),
            "vault.wrong_secret"
        );
        assert_eq!(
            reveal_confirmed(
                &conn,
                &service,
                &saved.id,
                Proof::Hello(b"another signature that is long enough"),
                now
            )
            .unwrap_err(),
            "vault.hello_failed"
        );
        assert_eq!(
            reveal_confirmed(&conn, &service, &saved.id, Proof::Hello(HELLO), now)
                .unwrap()
                .data
                .password,
            "pw"
        );
        assert_eq!(
            reveal_confirmed(&conn, &service, "nonsense", Proof::Password(PASSWORD), now)
                .unwrap_err(),
            "vault.not_found"
        );
        // Wrong master passwords slow down, like at the unlock screen.
        for _ in 0..3 {
            let _ = reveal_confirmed(
                &conn,
                &service,
                &saved.id,
                Proof::Password("nope nope nope"),
                now,
            );
        }
        assert!(
            reveal_confirmed(&conn, &service, &saved.id, Proof::Password(PASSWORD), now)
                .unwrap_err()
                .starts_with("vault.throttled:")
        );
        // When Windows Hello is required each time, a typed password no longer works.
        let later = now + Duration::from_secs(120);
        set_require_hello(&conn, &service, true, None, later).unwrap();
        assert_eq!(
            reveal_confirmed(&conn, &service, &saved.id, Proof::Password(PASSWORD), later)
                .unwrap_err(),
            "vault.hello_required"
        );
        assert!(reveal_confirmed(&conn, &service, &saved.id, Proof::Hello(HELLO), later).is_ok());
        service.lock();
        assert_eq!(
            reveal_confirmed(&conn, &service, &saved.id, Proof::Hello(HELLO), later).unwrap_err(),
            "vault.locked"
        );
    }

    #[test]
    fn asking_for_windows_hello_each_time_guards_passwords_and_needs_windows_hello() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        assert_eq!(
            set_require_hello(&conn, &service, true, None, now).unwrap_err(),
            "vault.method_unavailable"
        );
        enable_hello(&conn, &service, HELLO, now).unwrap();
        assert!(confirm_if_required(&conn, None).is_ok(), "off by default");
        set_require_hello(&conn, &service, true, None, now).unwrap();
        assert!(status(&conn, &service, now).unwrap().require_hello);
        assert_eq!(
            confirm_if_required(&conn, None).unwrap_err(),
            "vault.hello_required"
        );
        assert_eq!(
            confirm_if_required(&conn, Some(b"another signature that is long enough")).unwrap_err(),
            "vault.hello_failed"
        );
        confirm_if_required(&conn, Some(HELLO)).unwrap();
        // Changing it while it is on needs the proof too.
        assert_eq!(
            set_require_hello(&conn, &service, false, None, now).unwrap_err(),
            "vault.hello_required"
        );
        set_require_hello(&conn, &service, false, Some(HELLO), now).unwrap();
        set_require_hello(&conn, &service, true, None, now).unwrap();
        // Turning Windows Hello off drops the setting with it, so passwords never get stuck.
        disable_hello(&conn, &service, now).unwrap();
        assert!(!status(&conn, &service, now).unwrap().require_hello);
        assert!(confirm_if_required(&conn, None).is_ok());
    }

    #[test]
    fn an_import_adds_new_entries_once_and_an_export_lists_everything() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        save(&conn, &service, None, &entry("Existing", "pw"), 1, now).unwrap();
        let mut same = entry("Another title", "pw");
        same.url = "HTTPS://EXAMPLE.TEST".into();
        same.username = "ME@example.test".into();
        let fresh = EntryData {
            title: "Fresh".into(),
            url: "https://fresh.test".into(),
            username: "u".into(),
            password: "p".into(),
            notes: String::new(),
        };
        let invalid = EntryData {
            title: " ".into(),
            ..fresh.clone()
        };
        let result = import_entries(
            &conn,
            &service,
            vec![same, fresh.clone(), fresh, invalid],
            5,
            now,
        )
        .unwrap();
        assert_eq!(
            result,
            ImportResult {
                imported: 1,
                duplicates: 2,
                skipped: 1
            }
        );
        assert_eq!(list(&conn, &service, now).unwrap().entries.len(), 2);
        let (all, unreadable) = export_entries(&conn, &service, now).unwrap();
        assert_eq!((all.len(), unreadable), (2, 0));
        assert_eq!(all[0].title, "Existing");
        service.lock();
        assert_eq!(
            import_entries(&conn, &service, vec![], 5, now).unwrap_err(),
            "vault.locked"
        );
        assert_eq!(
            export_entries(&conn, &service, now).unwrap_err(),
            "vault.locked"
        );
    }

    #[test]
    fn verifying_the_password_counts_wrong_guesses() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        verify_password(&conn, &service, PASSWORD, now).unwrap();
        for _ in 0..3 {
            assert_eq!(
                verify_password(&conn, &service, "nope nope nope", now).unwrap_err(),
                "vault.wrong_secret"
            );
        }
        assert!(verify_password(&conn, &service, PASSWORD, now)
            .unwrap_err()
            .starts_with("vault.throttled:"));
    }

    #[test]
    fn auto_lock_choices_are_validated_and_applied() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        assert_eq!(
            set_auto_lock(&conn, &service, 7, now).unwrap_err(),
            "vault.invalid_setting"
        );
        set_auto_lock(&conn, &service, 1, now).unwrap();
        assert_eq!(status(&conn, &service, now).unwrap().auto_lock_minutes, 1);
        assert!(!service.is_unlocked(now + Duration::from_secs(61)));
    }

    #[test]
    fn a_damaged_entry_is_reported_but_does_not_hide_the_others() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        let good = save(&conn, &service, None, &entry("Good", "pw"), 1, now).unwrap();
        let bad = save(&conn, &service, None, &entry("Bad", "pw"), 2, now).unwrap();
        let mut row = repo::row(&conn, &bad.id).unwrap().unwrap();
        *row.blob.last_mut().unwrap() ^= 1;
        repo::put(&conn, &row.id, &row.blob, 2).unwrap();
        let listing = list(&conn, &service, now).unwrap();
        assert_eq!(listing.entries.len(), 1);
        assert_eq!(listing.entries[0].id, good.id);
        assert_eq!(listing.unreadable, 1);
        assert_eq!(
            reveal(&conn, &service, &bad.id, now).unwrap_err(),
            "vault.corrupt"
        );
    }

    #[test]
    fn resetting_erases_everything_and_allows_a_new_vault() {
        let (conn, service, now) = setup();
        create(&conn, &service, Some(PASSWORD), None, now).unwrap();
        save(&conn, &service, None, &entry("Site", "pw"), 1, now).unwrap();
        reset(&conn, &service).unwrap();
        let s = status(&conn, &service, now).unwrap();
        assert!(!s.initialized && !s.unlocked && s.entry_count == 0);
        create(&conn, &service, Some("a brand new password"), None, now).unwrap();
        assert_eq!(list(&conn, &service, now).unwrap().entries.len(), 0);
    }
}
