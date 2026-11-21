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
use std::time::Instant;
use zeroize::{Zeroize, Zeroizing};

#[derive(Debug, Serialize, PartialEq)]
pub struct Listing {
    pub entries: Vec<Summary>,
    /// Entries that could not be decrypted (damaged data); the rest still open.
    pub unreadable: usize,
}

fn fail(error: CryptoError, place: &str) -> String {
    let level = match error {
        CryptoError::Corrupt | CryptoError::Unavailable => Level::Error,
        _ => Level::Warning,
    };
    error_log::report(level, "vault", &format!("{place} failed: {}", error.code()), "");
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
    Ok(Status {
        initialized,
        unlocked: initialized && service.is_unlocked(now),
        entry_count: if initialized { repo::count(conn)? } else { 0 },
        auto_lock_minutes: repo::auto_lock_minutes(conn)?.unwrap_or(DEFAULT_AUTO_LOCK_MINUTES),
        retry_after_seconds: service.retry_after_seconds(now),
    })
}

/// Creates the vault and leaves it unlocked. Returns the recovery key, which is never stored.
pub fn create(
    conn: &Connection,
    service: &VaultService,
    master_password: &str,
    now: Instant,
) -> Result<Zeroizing<String>, String> {
    if repo::load_header(conn)?.is_some() {
        return Err("vault.already_initialized".into());
    }
    let (header, key, recovery) =
        crypto::create(master_password).map_err(|e| fail(e, "creating the vault"))?;
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
            service.set_auto_lock_minutes(repo::auto_lock_minutes(conn)?.unwrap_or(DEFAULT_AUTO_LOCK_MINUTES));
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

pub fn unlock(conn: &Connection, service: &VaultService, password: &str, now: Instant) -> Result<(), String> {
    unlock_with(conn, service, now, "the password", |h| crypto::unlock_with_password(h, password))
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
    Ok(Listing { entries, unreadable })
}

pub fn reveal(conn: &Connection, service: &VaultService, id: &str, now: Instant) -> Result<Revealed, String> {
    let key = service.key(now)?;
    if !valid_id(id) {
        return Err("vault.not_found".into());
    }
    let row = repo::row(conn, id)?.ok_or("vault.not_found")?;
    let plain = crypto::open_entry(&key, &row.id, &row.blob).map_err(|e| fail(e, "reading a vault entry"))?;
    let data: EntryData = serde_json::from_slice(&plain).map_err(|_| fail(CryptoError::Corrupt, "reading a vault entry"))?;
    Ok(Revealed { id: row.id, data, updated_at: row.updated_at })
}

/// Only the password of one entry, for copying it from the backend (it never reaches the interface).
pub fn password(conn: &Connection, service: &VaultService, id: &str, now: Instant) -> Result<Zeroizing<String>, String> {
    let mut revealed = reveal(conn, service, id, now)?;
    let password = Zeroizing::new(std::mem::take(&mut revealed.data.password));
    revealed.data.notes.zeroize();
    Ok(password)
}

fn new_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| fail(CryptoError::Unavailable, "creating an entry id"))?;
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
    let blob = crypto::seal_entry(&key, &id, &plain).map_err(|e| fail(e, "saving a vault entry"))?;
    repo::put(conn, &id, &blob, timestamp)?;
    Ok(data.summary(&id, timestamp))
}

pub fn delete(conn: &Connection, service: &VaultService, id: &str, now: Instant) -> Result<(), String> {
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
    error_log::report(Level::Info, "vault", "the vault master password was changed", "");
    Ok(())
}

pub fn set_auto_lock(conn: &Connection, service: &VaultService, minutes: u32, now: Instant) -> Result<(), String> {
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
    error_log::report(Level::Warning, "vault", "the password vault was erased and reset", "");
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
        let recovery = create(&conn, &service, PASSWORD, now).unwrap();
        assert_eq!(recovery.len(), 39);
        let s = status(&conn, &service, now).unwrap();
        assert!(s.initialized && s.unlocked && s.entry_count == 0);
        assert_eq!(create(&conn, &service, PASSWORD, now).unwrap_err(), "vault.already_initialized");
        let (other, other_service, _) = setup();
        assert_eq!(create(&other, &other_service, "short", now).unwrap_err(), "vault.password_invalid");
        assert!(!status(&other, &other_service, now).unwrap().initialized);
    }

    #[test]
    fn entries_are_encrypted_at_rest_listed_without_secrets_and_revealed_on_request() {
        let (conn, service, now) = setup();
        create(&conn, &service, PASSWORD, now).unwrap();
        let saved = save(&conn, &service, None, &entry("Bank", "hunter2-secret"), 10, now).unwrap();
        save(&conn, &service, None, &entry("alpha", "p"), 11, now).unwrap();
        // Nothing readable in the database.
        for row in repo::rows(&conn).unwrap() {
            let text = String::from_utf8_lossy(&row.blob).to_string();
            assert!(!text.contains("hunter2") && !text.contains("Bank") && !text.contains("example.test"));
        }
        let listing = list(&conn, &service, now).unwrap();
        assert_eq!(listing.entries.iter().map(|e| e.title.as_str()).collect::<Vec<_>>(), ["alpha", "Bank"]);
        assert!(!serde_json::to_string(&listing).unwrap().contains("hunter2"));
        let shown = reveal(&conn, &service, &saved.id, now).unwrap();
        assert_eq!(shown.data.password, "hunter2-secret");
        assert_eq!(shown.data.notes, "private note");
    }

    #[test]
    fn saving_can_replace_an_entry_and_deleting_removes_it() {
        let (conn, service, now) = setup();
        create(&conn, &service, PASSWORD, now).unwrap();
        let first = save(&conn, &service, None, &entry("Site", "one"), 1, now).unwrap();
        save(&conn, &service, Some(&first.id), &entry("Site", "two"), 2, now).unwrap();
        assert_eq!(list(&conn, &service, now).unwrap().entries.len(), 1);
        assert_eq!(reveal(&conn, &service, &first.id, now).unwrap().data.password, "two");
        assert_eq!(save(&conn, &service, Some(&"0".repeat(32)), &entry("x", "y"), 3, now).unwrap_err(), "vault.not_found");
        assert_eq!(save(&conn, &service, None, &entry("  ", "y"), 3, now).unwrap_err(), "vault.invalid_entry");
        delete(&conn, &service, &first.id, now).unwrap();
        assert_eq!(delete(&conn, &service, &first.id, now).unwrap_err(), "vault.not_found");
        assert_eq!(reveal(&conn, &service, "nonsense", now).unwrap_err(), "vault.not_found");
    }

    #[test]
    fn nothing_works_while_locked_and_unlocking_needs_the_right_secret() {
        let (conn, service, now) = setup();
        let recovery = create(&conn, &service, PASSWORD, now).unwrap();
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
        assert_eq!(unlock(&conn, &service, "wrong password", now).unwrap_err(), "vault.wrong_secret");
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
        create(&conn, &service, PASSWORD, now).unwrap();
        service.lock();
        for _ in 0..3 {
            assert_eq!(unlock(&conn, &service, "nope nope nope", now).unwrap_err(), "vault.wrong_secret");
        }
        let blocked = unlock(&conn, &service, PASSWORD, now).unwrap_err();
        assert!(blocked.starts_with("vault.throttled:"), "{blocked}");
        assert!(status(&conn, &service, now).unwrap().retry_after_seconds > 0);
        unlock(&conn, &service, PASSWORD, now + Duration::from_secs(5)).unwrap();
    }

    #[test]
    fn changing_the_master_password_needs_the_current_one_and_keeps_the_entries() {
        let (conn, service, now) = setup();
        let recovery = create(&conn, &service, PASSWORD, now).unwrap();
        let saved = save(&conn, &service, None, &entry("Site", "pw"), 1, now).unwrap();
        assert_eq!(change_password(&conn, &service, "not it at all", "a brand new password", now).unwrap_err(), "vault.wrong_secret");
        assert_eq!(change_password(&conn, &service, PASSWORD, "short", now).unwrap_err(), "vault.password_invalid");
        change_password(&conn, &service, PASSWORD, "a brand new password", now).unwrap();
        service.lock();
        assert!(unlock(&conn, &service, PASSWORD, now).is_err());
        unlock(&conn, &service, "a brand new password", now + Duration::from_secs(120)).unwrap();
        assert_eq!(reveal(&conn, &service, &saved.id, now + Duration::from_secs(120)).unwrap().data.password, "pw");
        service.lock();
        unlock_with_recovery(&conn, &service, &recovery, now + Duration::from_secs(120)).unwrap();
    }

    #[test]
    fn auto_lock_choices_are_validated_and_applied() {
        let (conn, service, now) = setup();
        create(&conn, &service, PASSWORD, now).unwrap();
        assert_eq!(set_auto_lock(&conn, &service, 7, now).unwrap_err(), "vault.invalid_setting");
        set_auto_lock(&conn, &service, 1, now).unwrap();
        assert_eq!(status(&conn, &service, now).unwrap().auto_lock_minutes, 1);
        assert!(!service.is_unlocked(now + Duration::from_secs(61)));
    }

    #[test]
    fn a_damaged_entry_is_reported_but_does_not_hide_the_others() {
        let (conn, service, now) = setup();
        create(&conn, &service, PASSWORD, now).unwrap();
        let good = save(&conn, &service, None, &entry("Good", "pw"), 1, now).unwrap();
        let bad = save(&conn, &service, None, &entry("Bad", "pw"), 2, now).unwrap();
        let mut row = repo::row(&conn, &bad.id).unwrap().unwrap();
        *row.blob.last_mut().unwrap() ^= 1;
        repo::put(&conn, &row.id, &row.blob, 2).unwrap();
        let listing = list(&conn, &service, now).unwrap();
        assert_eq!(listing.entries.len(), 1);
        assert_eq!(listing.entries[0].id, good.id);
        assert_eq!(listing.unreadable, 1);
        assert_eq!(reveal(&conn, &service, &bad.id, now).unwrap_err(), "vault.corrupt");
    }

    #[test]
    fn resetting_erases_everything_and_allows_a_new_vault() {
        let (conn, service, now) = setup();
        create(&conn, &service, PASSWORD, now).unwrap();
        save(&conn, &service, None, &entry("Site", "pw"), 1, now).unwrap();
        reset(&conn, &service).unwrap();
        let s = status(&conn, &service, now).unwrap();
        assert!(!s.initialized && !s.unlocked && s.entry_count == 0);
        create(&conn, &service, "a brand new password", now).unwrap();
        assert_eq!(list(&conn, &service, now).unwrap().entries.len(), 0);
    }
}
