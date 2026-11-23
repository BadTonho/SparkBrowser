//! Turning the app lock on and off and checking what is presented to it. Everything works on a
//! connection and takes the time as an argument, so it is tested without a window.
use super::verifier::{self, Verifier};
use super::{Config, Method};
use crate::database::app_lock_repo as repo;
use crate::error_log::{self, Level};
use crate::vault::ops as vault;
use serde::Serialize;

/// What the person presents to prove it is them.
pub enum Proof<'a> {
    /// The lock's own password.
    Password(&'a str),
    /// The secret Windows Hello produced after PIN, fingerprint or face.
    Hello(&'a [u8]),
    /// The master password of the profile's password vault.
    Vault(&'a str),
}

/// The first method of a lock that is still off.
pub enum First<'a> {
    Password(&'a str),
    Hello(&'a [u8]),
    Vault(&'a str),
}

/// What the interface may know about the lock.
#[derive(Serialize, Debug, PartialEq)]
pub struct Status {
    pub enabled: bool,
    pub password: bool,
    pub hello: bool,
    pub vault: bool,
    /// The vault of this profile has a master password, so it could be one of the methods.
    pub vault_available: bool,
    /// Seconds to wait before another attempt is accepted (0 when none).
    pub retry_after_seconds: u64,
}

fn enabled_config(conn: &rusqlite::Connection) -> Result<Config, String> {
    repo::load(conn)?.ok_or_else(|| "lock.not_enabled".to_string())
}

pub fn is_enabled(conn: &rusqlite::Connection) -> Result<bool, String> {
    Ok(repo::load(conn)?.is_some())
}

/// Seconds left before another attempt is accepted, 0 when there is no wait.
pub fn retry_after(conn: &rusqlite::Connection, now: i64) -> Result<u64, String> {
    let (_, blocked_until) = repo::brake(conn)?;
    Ok(u64::try_from(blocked_until - now).unwrap_or(0))
}

pub fn status(conn: &rusqlite::Connection, now: i64) -> Result<Status, String> {
    let config = repo::load(conn)?;
    let vault_available = vault::has_master_password(conn)?;
    Ok(Status {
        enabled: config.is_some(),
        password: config.as_ref().is_some_and(|c| c.password.is_some()),
        hello: config.as_ref().is_some_and(|c| c.hello.is_some()),
        vault: config.as_ref().is_some_and(|c| c.vault),
        vault_available,
        retry_after_seconds: retry_after(conn, now)?,
    })
}

fn throttled(seconds: u64) -> String {
    format!("lock.throttled:{seconds}")
}

fn record_failure(conn: &rusqlite::Connection, now: i64) -> Result<(), String> {
    let (failures, _) = repo::brake(conn)?;
    let failures = failures.saturating_add(1);
    let blocked_until = crate::brake::delay_after(failures).map_or(0, |delay| {
        now.saturating_add(i64::try_from(delay).unwrap_or(0))
    });
    repo::set_brake(conn, failures, blocked_until)
}

/// Checks what was presented. A wrong secret counts toward the brake, which survives a restart; a
/// right one clears it.
pub fn verify(conn: &rusqlite::Connection, proof: &Proof<'_>, now: i64) -> Result<Method, String> {
    let config = enabled_config(conn)?;
    let wait = retry_after(conn, now)?;
    if wait > 0 {
        return Err(throttled(wait));
    }
    let (method, accepted) = match proof {
        Proof::Password(password) => {
            let verifier = config.password.as_ref().ok_or("lock.method_unavailable")?;
            (Method::Password, verifier.matches(password)?)
        }
        Proof::Hello(secret) => {
            let stored = config.hello.as_deref().ok_or("lock.method_unavailable")?;
            (Method::Hello, verifier::hello_matches(stored, secret))
        }
        Proof::Vault(password) => {
            if !config.vault || !vault::has_master_password(conn)? {
                return Err("lock.method_unavailable".into());
            }
            (
                Method::Vault,
                vault::matches_master_password(conn, password)?,
            )
        }
    };
    if accepted {
        repo::set_brake(conn, 0, 0)?;
        Ok(method)
    } else {
        record_failure(conn, now)?;
        error_log::report(
            Level::Warning,
            "app-lock",
            "a wrong secret was presented to the app lock",
            "",
        );
        match retry_after(conn, now)? {
            0 => Err("lock.wrong_secret".into()),
            seconds => Err(format!("lock.wrong_secret_wait:{seconds}")),
        }
    }
}

/// Turns the lock on with its first method. Only while the lock is off.
pub fn enable(conn: &rusqlite::Connection, first: First<'_>) -> Result<(), String> {
    if repo::load(conn)?.is_some() {
        return Err("lock.already_on".into());
    }
    let mut config = Config::new();
    match first {
        First::Password(password) => config.password = Some(Verifier::new(password)?),
        First::Hello(secret) => config.hello = Some(verifier::hello_fingerprint(secret)),
        First::Vault(password) => {
            use_vault_password(conn, password)?;
            config.vault = true;
        }
    }
    repo::save(conn, &config)
}

/// The vault's master password may only be one of the ways in when it is the right one.
fn use_vault_password(conn: &rusqlite::Connection, password: &str) -> Result<(), String> {
    if !vault::has_master_password(conn)? {
        return Err("lock.vault_without_password".into());
    }
    if vault::matches_master_password(conn, password)? {
        Ok(())
    } else {
        Err("lock.wrong_secret".into())
    }
}

/// Sets (or changes) the lock's own password, after proving who is asking with a current method.
pub fn set_password(
    conn: &rusqlite::Connection,
    proof: &Proof<'_>,
    new_password: &str,
    now: i64,
) -> Result<(), String> {
    verify(conn, proof, now)?;
    let verifier = Verifier::new(new_password)?;
    let mut config = enabled_config(conn)?;
    config.password = Some(verifier);
    repo::save(conn, &config)
}

/// Adds Windows Hello as a way in (the secret comes from enrolling the key pair).
pub fn add_hello(
    conn: &rusqlite::Connection,
    proof: &Proof<'_>,
    secret: &[u8],
    now: i64,
) -> Result<(), String> {
    verify(conn, proof, now)?;
    let mut config = enabled_config(conn)?;
    config.hello = Some(verifier::hello_fingerprint(secret));
    repo::save(conn, &config)
}

/// Adds the vault's master password as a way in.
pub fn add_vault(
    conn: &rusqlite::Connection,
    proof: &Proof<'_>,
    vault_password: &str,
    now: i64,
) -> Result<(), String> {
    verify(conn, proof, now)?;
    use_vault_password(conn, vault_password)?;
    let mut config = enabled_config(conn)?;
    config.vault = true;
    repo::save(conn, &config)
}

/// Turns one way in off. The last one cannot be removed this way (turn the lock off instead).
pub fn remove_method(
    conn: &rusqlite::Connection,
    proof: &Proof<'_>,
    method: Method,
    now: i64,
) -> Result<(), String> {
    verify(conn, proof, now)?;
    let mut config = enabled_config(conn)?;
    match method {
        Method::Password => config.password = None,
        Method::Hello => config.hello = None,
        Method::Vault => config.vault = false,
    }
    if config.method_count() == 0 {
        return Err("lock.last_method".into());
    }
    repo::save(conn, &config)
}

/// Turns the lock off.
pub fn disable(conn: &rusqlite::Connection, proof: &Proof<'_>, now: i64) -> Result<(), String> {
    verify(conn, proof, now)?;
    repo::clear(conn)
}

/// The lock can only be opened with the vault's master password, so that password must stay.
/// Vault changes that would remove it (turning the master password off, erasing the vault) ask first.
pub fn depends_only_on_vault(conn: &rusqlite::Connection) -> Result<bool, String> {
    Ok(repo::load(conn)?
        .is_some_and(|config| config.vault && config.password.is_none() && config.hello.is_none()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::crypto;
    use rusqlite::Connection;

    const PASSWORD: &str = "correct horse battery";
    const OTHER: &str = "another long password";

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::database::vault_repo::migrate(&conn).unwrap();
        repo::migrate(&conn).unwrap();
        conn
    }

    fn with_vault(conn: &Connection, master: Option<&str>) {
        // Windows Hello secrets are long signatures; a short stand-in is refused by the vault.
        let (header, _, _) = crypto::create(master, Some(&[7u8; 256])).unwrap();
        crate::database::vault_repo::create(conn, &header, 5).unwrap();
    }

    #[test]
    fn a_profile_starts_without_a_lock_and_nothing_can_be_verified() {
        let conn = db();
        assert!(!is_enabled(&conn).unwrap());
        assert!(!status(&conn, 0).unwrap().enabled);
        assert_eq!(
            verify(&conn, &Proof::Password(PASSWORD), 0).unwrap_err(),
            "lock.not_enabled"
        );
    }

    #[test]
    fn the_own_password_opens_and_a_wrong_one_does_not() {
        let conn = db();
        enable(&conn, First::Password(PASSWORD)).unwrap();
        assert_eq!(
            verify(&conn, &Proof::Password(PASSWORD), 100).unwrap(),
            Method::Password
        );
        assert_eq!(
            verify(&conn, &Proof::Password(OTHER), 100).unwrap_err(),
            "lock.wrong_secret"
        );
        let status = status(&conn, 100).unwrap();
        assert!(status.enabled && status.password && !status.hello && !status.vault);
        assert_eq!(
            enable(&conn, First::Password(OTHER)).unwrap_err(),
            "lock.already_on"
        );
    }

    #[test]
    fn wrong_guesses_slow_down_survive_a_restart_and_a_right_one_clears_them() {
        let conn = db();
        enable(&conn, First::Password(PASSWORD)).unwrap();
        let now = 1_000;
        for _ in 0..2 {
            assert_eq!(
                verify(&conn, &Proof::Password(OTHER), now).unwrap_err(),
                "lock.wrong_secret"
            );
        }
        // The third mistake starts the wait.
        let third = verify(&conn, &Proof::Password(OTHER), now).unwrap_err();
        assert!(third.starts_with("lock.wrong_secret_wait:"), "{third}");
        // While waiting even the right password is refused, and the wait is on disk.
        assert!(verify(&conn, &Proof::Password(PASSWORD), now)
            .unwrap_err()
            .starts_with("lock.throttled:"));
        assert!(retry_after(&conn, now).unwrap() > 0);
        assert_eq!(retry_after(&conn, now + 120).unwrap(), 0);
        assert_eq!(
            verify(&conn, &Proof::Password(PASSWORD), now + 120).unwrap(),
            Method::Password
        );
        assert_eq!(
            repo::brake(&conn).unwrap(),
            (0, 0),
            "a right secret clears the brake"
        );
    }

    #[test]
    fn windows_hello_is_checked_by_the_hash_of_its_signature() {
        let conn = db();
        enable(&conn, First::Hello(b"signature")).unwrap();
        assert_eq!(
            verify(&conn, &Proof::Hello(b"signature"), 0).unwrap(),
            Method::Hello
        );
        assert_eq!(
            verify(&conn, &Proof::Hello(b"forged"), 0).unwrap_err(),
            "lock.wrong_secret"
        );
        assert_eq!(
            verify(&conn, &Proof::Password(PASSWORD), 0).unwrap_err(),
            "lock.method_unavailable"
        );
        let stored = serde_json::to_string(&repo::load(&conn).unwrap().unwrap()).unwrap();
        assert!(!stored.contains("signature"));
    }

    #[test]
    fn the_vault_master_password_opens_without_opening_the_vault() {
        let conn = db();
        with_vault(&conn, Some("vault master pw"));
        enable(&conn, First::Vault("vault master pw")).unwrap();
        assert_eq!(
            verify(&conn, &Proof::Vault("vault master pw"), 0).unwrap(),
            Method::Vault
        );
        assert_eq!(
            verify(&conn, &Proof::Vault("not the vault pw"), 0).unwrap_err(),
            "lock.wrong_secret"
        );
        assert!(depends_only_on_vault(&conn).unwrap());
    }

    #[test]
    fn the_vault_method_needs_a_vault_with_a_master_password_and_the_right_one() {
        let conn = db();
        assert_eq!(
            enable(&conn, First::Vault("x")).unwrap_err(),
            "lock.vault_without_password",
            "no vault at all"
        );
        with_vault(&conn, None);
        assert!(
            !status(&conn, 0).unwrap().vault_available,
            "a Hello-only vault has no master password"
        );
        assert_eq!(
            enable(&conn, First::Vault("x")).unwrap_err(),
            "lock.vault_without_password"
        );
        let other = db();
        with_vault(&other, Some("vault master pw"));
        assert!(status(&other, 0).unwrap().vault_available);
        assert_eq!(
            enable(&other, First::Vault("wrong")).unwrap_err(),
            "lock.wrong_secret"
        );
        assert!(!is_enabled(&other).unwrap(), "nothing was turned on");
    }

    #[test]
    fn changes_need_a_current_method_and_the_last_method_stays() {
        let conn = db();
        enable(&conn, First::Password(PASSWORD)).unwrap();
        // Adding a way in, and changing the password, ask who is asking.
        assert_eq!(
            add_hello(&conn, &Proof::Password(OTHER), b"sig", 0).unwrap_err(),
            "lock.wrong_secret"
        );
        add_hello(&conn, &Proof::Password(PASSWORD), b"sig", 0).unwrap();
        set_password(&conn, &Proof::Hello(b"sig"), OTHER, 0).unwrap();
        assert!(verify(&conn, &Proof::Password(OTHER), 0).is_ok());
        assert!(verify(&conn, &Proof::Password(PASSWORD), 0).is_err());
        // One way in can go while another remains; the last one cannot.
        remove_method(&conn, &Proof::Password(OTHER), Method::Hello, 0).unwrap();
        assert_eq!(
            remove_method(&conn, &Proof::Password(OTHER), Method::Password, 0).unwrap_err(),
            "lock.last_method"
        );
        assert!(is_enabled(&conn).unwrap());
        // Turning the lock off is a different step and needs a current method too.
        assert!(disable(&conn, &Proof::Password(PASSWORD), 0).is_err());
        disable(&conn, &Proof::Password(OTHER), 0).unwrap();
        assert!(!is_enabled(&conn).unwrap());
    }

    #[test]
    fn a_short_password_is_refused_and_leaves_the_lock_off() {
        let conn = db();
        assert_eq!(
            enable(&conn, First::Password("short")).unwrap_err(),
            "lock.password_invalid"
        );
        assert!(!is_enabled(&conn).unwrap());
    }

    #[test]
    fn a_lock_with_several_methods_does_not_depend_only_on_the_vault() {
        let conn = db();
        with_vault(&conn, Some("vault master pw"));
        enable(&conn, First::Vault("vault master pw")).unwrap();
        add_hello(&conn, &Proof::Vault("vault master pw"), b"sig", 0).unwrap();
        assert!(!depends_only_on_vault(&conn).unwrap());
    }
}
