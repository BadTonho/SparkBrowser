//! Windows Hello does not exist outside Windows: the vault there opens with the master password.
use super::HelloError;
use zeroize::Zeroizing;

pub fn is_available() -> bool {
    false
}

pub fn enroll(_name: &str) -> Result<Zeroizing<Vec<u8>>, HelloError> {
    Err(HelloError::Unavailable)
}

pub fn secret(_name: &str) -> Result<Zeroizing<Vec<u8>>, HelloError> {
    Err(HelloError::Unavailable)
}

pub fn forget(_name: &str) -> Result<(), HelloError> {
    Err(HelloError::Unavailable)
}
