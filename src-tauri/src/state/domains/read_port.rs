use crate::sync::{Mutex, MutexGuard};
use std::ops::Deref;

/// Borrowed query port: callers cannot recover or mutate the underlying mutex.
pub(crate) struct ReadPort<'a, T>(&'a Mutex<T>);
pub(crate) struct ReadGuard<'a, T>(MutexGuard<'a, T>);

impl<'a, T> ReadPort<'a, T> {
    pub(super) fn new(value: &'a Mutex<T>) -> Self {
        Self(value)
    }
    pub(crate) fn lock(&self) -> Result<ReadGuard<'a, T>, String> {
        self.0.lock().map(ReadGuard).map_err(|e| e.to_string())
    }
}
impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
