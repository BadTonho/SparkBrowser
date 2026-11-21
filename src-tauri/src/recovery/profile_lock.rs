use std::{
    fs::{File, OpenOptions, TryLockError},
    path::Path,
};

/// The file stays on disk; the OS releases its lock when the process exits or crashes.
pub struct ProfileLock {
    _file: File,
}
impl ProfileLock {
    pub fn acquire(directory: &Path) -> Result<Self, TryLockError> {
        std::fs::create_dir_all(directory).map_err(TryLockError::Error)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(directory.join("profile.lock"))
            .map_err(TryLockError::Error)?;
        file.try_lock()?;
        Ok(Self { _file: file })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profile_lock_is_exclusive_and_released_without_deleting_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let first = ProfileLock::acquire(dir.path()).unwrap();
        assert!(matches!(
            ProfileLock::acquire(dir.path()),
            Err(TryLockError::WouldBlock)
        ));
        drop(first);
        assert!(dir.path().join("profile.lock").exists());
        assert!(ProfileLock::acquire(dir.path()).is_ok());
    }
}
