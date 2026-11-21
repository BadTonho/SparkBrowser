pub mod manager;
pub mod native;
#[cfg(test)]
mod tests;
pub mod transfer;

pub mod files;
pub mod location;
pub mod model;
#[cfg(not(windows))]
mod native_fallback;
#[cfg(target_os = "linux")]
mod native_linux;
#[cfg(windows)]
mod native_windows;
pub mod resumable;
pub mod visual;
