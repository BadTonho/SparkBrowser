//! `Mutex` and `RwLock` that never stay poisoned.
//!
//! A `std::sync` lock is poisoned when a thread panics while holding it, and from then on every
//! `lock()` fails. Code that answered that failure with `let Ok(..) = lock() else { return }` made a
//! feature stop working silently until Spark restarted, and `.expect()` repeated the panic on
//! every later call. These locks recover instead: the panic is already in the error log, so the
//! state it left is kept, the poison flag is cleared and the recovery is logged as a `Warning`
//! with the place in the code that found it.
//!
//! `lock()`, `read()` and `write()` keep the `Result` signature of `std::sync`, so existing callers
//! compile unchanged; the error is never returned. `crate::error_log` keeps the plain `std::sync`
//! locks, because the recovery itself writes to the log.
use std::{fmt, panic::Location, sync};

pub use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

/// Never returned. It exists so callers written for `std::sync` still compile.
#[derive(Debug)]
pub struct Poisoned(());

impl fmt::Display for Poisoned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("lock poisoned")
    }
}

impl std::error::Error for Poisoned {}

#[track_caller]
fn report_recovery() {
    let place = Location::caller();
    crate::error_log::report(
        crate::error_log::Level::Warning,
        "native",
        &format!(
            "Recovered a lock left poisoned by a panic, at {}:{}",
            place.file(),
            place.line()
        ),
        "",
    );
}

#[derive(Debug, Default)]
pub struct Mutex<T: ?Sized>(sync::Mutex<T>);

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self(sync::Mutex::new(value))
    }

    pub fn into_inner(self) -> T {
        self.0
            .into_inner()
            .unwrap_or_else(sync::PoisonError::into_inner)
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Locks the mutex, recovering it when a panic poisoned it.
    #[track_caller]
    pub fn guard(&self) -> MutexGuard<'_, T> {
        match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                report_recovery();
                self.0.clear_poison();
                poisoned.into_inner()
            }
        }
    }

    /// Same as [`Mutex::guard`]; always `Ok`.
    #[track_caller]
    pub fn lock(&self) -> Result<MutexGuard<'_, T>, Poisoned> {
        Ok(self.guard())
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.0
            .get_mut()
            .unwrap_or_else(sync::PoisonError::into_inner)
    }

    #[cfg(test)]
    fn is_poisoned(&self) -> bool {
        self.0.is_poisoned()
    }
}

#[derive(Debug, Default)]
pub struct RwLock<T: ?Sized>(sync::RwLock<T>);

impl<T> RwLock<T> {
    pub const fn new(value: T) -> Self {
        Self(sync::RwLock::new(value))
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Always `Ok`; a poisoned lock is recovered and logged.
    #[track_caller]
    pub fn read(&self) -> Result<RwLockReadGuard<'_, T>, Poisoned> {
        Ok(match self.0.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                report_recovery();
                self.0.clear_poison();
                poisoned.into_inner()
            }
        })
    }

    /// Always `Ok`; a poisoned lock is recovered and logged.
    #[track_caller]
    pub fn write(&self) -> Result<RwLockWriteGuard<'_, T>, Poisoned> {
        Ok(match self.0.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                report_recovery();
                self.0.clear_poison();
                poisoned.into_inner()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn a_mutex_poisoned_by_a_panic_is_recovered_with_its_state_and_cleared() {
        let mutex = Arc::new(Mutex::new(1));
        let poisoner = mutex.clone();
        let outcome = std::thread::spawn(move || {
            let mut guard = poisoner.guard();
            *guard = 2;
            panic!("poison the lock");
        })
        .join();
        assert!(outcome.is_err());
        assert!(mutex.is_poisoned());
        assert_eq!(
            *mutex.lock().unwrap(),
            2,
            "the state left by the panic is kept"
        );
        assert!(!mutex.is_poisoned(), "later callers lock normally again");
        *mutex.guard() = 3;
        assert_eq!(*mutex.guard(), 3);
    }

    #[test]
    fn a_poisoned_rwlock_still_reads_and_writes() {
        let lock = Arc::new(RwLock::new(1));
        let poisoner = lock.clone();
        let outcome = std::thread::spawn(move || {
            let mut guard = poisoner.write().unwrap();
            *guard = 2;
            panic!("poison the lock");
        })
        .join();
        assert!(outcome.is_err());
        assert_eq!(*lock.read().unwrap(), 2);
        *lock.write().unwrap() = 3;
        assert_eq!(*lock.read().unwrap(), 3);
    }
}
