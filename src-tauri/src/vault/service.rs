//! The unlocked state of the vault: the data key (memory only), the idle timer and the brake on
//! wrong guesses. The key is never written anywhere; closing the browser locks the vault.
use super::crypto::Key;
use super::model::DEFAULT_AUTO_LOCK_MINUTES;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

const MAX_DELAY_SECONDS: u64 = 60;
/// Wrong guesses allowed before every further attempt has to wait.
const FREE_ATTEMPTS: u32 = 3;

struct Inner {
    key: Option<Key>,
    last_activity: Instant,
    auto_lock: Duration,
    failures: u32,
    blocked_until: Option<Instant>,
}

#[derive(Clone)]
pub struct VaultService(Arc<Mutex<Inner>>);

impl Default for VaultService {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Inner {
            key: None,
            last_activity: Instant::now(),
            auto_lock: minutes(DEFAULT_AUTO_LOCK_MINUTES),
            failures: 0,
            blocked_until: None,
        })))
    }
}

fn minutes(value: u32) -> Duration {
    Duration::from_secs(u64::from(value) * 60)
}

impl VaultService {
    fn inner(&self) -> MutexGuard<'_, Inner> {
        // A panic elsewhere must not leave the vault stuck open or unusable.
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Locks the vault if it has been idle for longer than the configured time.
    fn expire(inner: &mut Inner, now: Instant) -> bool {
        let idle = now.saturating_duration_since(inner.last_activity);
        if inner.key.is_some() && idle >= inner.auto_lock {
            inner.key = None;
            return true;
        }
        false
    }

    pub fn unlock(&self, key: Key, now: Instant) {
        let mut inner = self.inner();
        inner.key = Some(key);
        inner.last_activity = now;
        inner.failures = 0;
        inner.blocked_until = None;
    }

    /// Returns true when the vault was open.
    pub fn lock(&self) -> bool {
        self.inner().key.take().is_some()
    }

    /// The key for an operation, which also counts as activity. Err when locked (or just timed out).
    pub fn key(&self, now: Instant) -> Result<Key, String> {
        let mut inner = self.inner();
        Self::expire(&mut inner, now);
        match &inner.key {
            Some(key) => {
                let key = key.clone();
                inner.last_activity = now;
                Ok(key)
            }
            None => Err("vault.locked".into()),
        }
    }

    pub fn is_unlocked(&self, now: Instant) -> bool {
        let mut inner = self.inner();
        Self::expire(&mut inner, now);
        inner.key.is_some()
    }

    /// Called by the background timer. Returns true when this call locked the vault.
    pub fn lock_if_idle(&self, now: Instant) -> bool {
        Self::expire(&mut self.inner(), now)
    }

    pub fn set_auto_lock_minutes(&self, value: u32) {
        self.inner().auto_lock = minutes(value);
    }

    /// Err(seconds to wait) while the brake on wrong guesses is on.
    pub fn check_attempt(&self, now: Instant) -> Result<(), u64> {
        match self.inner().blocked_until {
            Some(until) if until > now => Err((until - now).as_secs() + 1),
            _ => Ok(()),
        }
    }

    pub fn retry_after_seconds(&self, now: Instant) -> u64 {
        self.check_attempt(now).err().unwrap_or(0)
    }

    pub fn failure(&self, now: Instant) {
        let mut inner = self.inner();
        inner.failures = inner.failures.saturating_add(1);
        if inner.failures >= FREE_ATTEMPTS {
            let exponent = (inner.failures - FREE_ATTEMPTS).min(6);
            let delay = (1u64 << exponent).min(MAX_DELAY_SECONDS);
            inner.blocked_until = Some(now + Duration::from_secs(delay));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::crypto;

    fn key() -> Key {
        crypto::create(Some("correct horse battery"), None)
            .unwrap()
            .1
    }

    #[test]
    fn it_starts_locked_and_locks_on_request() {
        let service = VaultService::default();
        let now = Instant::now();
        assert!(!service.is_unlocked(now));
        assert_eq!(service.key(now).err().as_deref(), Some("vault.locked"));
        service.unlock(key(), now);
        assert!(service.is_unlocked(now));
        assert!(service.key(now).is_ok());
        assert!(service.lock());
        assert!(!service.lock(), "already locked");
        assert!(service.key(now).is_err());
    }

    #[test]
    fn the_vault_locks_itself_after_the_idle_time_and_activity_postpones_it() {
        let service = VaultService::default();
        let start = Instant::now();
        service.set_auto_lock_minutes(1);
        service.unlock(key(), start);
        // Using the vault resets the clock.
        assert!(service.key(start + Duration::from_secs(50)).is_ok());
        assert!(!service.lock_if_idle(start + Duration::from_secs(100)));
        assert!(service.is_unlocked(start + Duration::from_secs(100)));
        // 60 seconds without use locks it, both when asked and by the timer.
        assert!(service.key(start + Duration::from_secs(120)).is_err());
        assert!(!service.is_unlocked(start + Duration::from_secs(120)));
        service.unlock(key(), start);
        assert!(service.lock_if_idle(start + Duration::from_secs(61)));
        assert!(
            !service.lock_if_idle(start + Duration::from_secs(62)),
            "only reports the change once"
        );
    }

    #[test]
    fn wrong_guesses_slow_down_and_a_success_clears_the_brake() {
        let service = VaultService::default();
        let now = Instant::now();
        service.failure(now);
        service.failure(now);
        assert!(
            service.check_attempt(now).is_ok(),
            "a few honest mistakes cost nothing"
        );
        service.failure(now);
        assert!(service.check_attempt(now).is_err());
        assert!(service.check_attempt(now + Duration::from_secs(2)).is_ok());
        for _ in 0..30 {
            service.failure(now);
        }
        let wait = service.check_attempt(now).unwrap_err();
        assert!(wait <= MAX_DELAY_SECONDS + 1, "the delay is capped");
        service.unlock(key(), now);
        assert!(service.check_attempt(now).is_ok());
        assert_eq!(service.retry_after_seconds(now), 0);
    }
}
