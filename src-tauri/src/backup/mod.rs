mod credentials;
pub use credentials::migrate_legacy_secrets;
pub mod bookmark_export;
pub mod crypto;
pub mod encryption;
pub mod files;
pub mod gdrive_webview;
pub mod local;
pub mod merge;
pub mod migrate;
pub mod model;
pub mod service;
pub mod sync;
pub mod validation;

#[cfg(test)]
mod integrity_tests;
#[cfg(test)]
mod legacy_tests;
#[cfg(test)]
mod regression_tests;
#[cfg(test)]
mod selective_tests;
#[cfg(test)]
mod tests;
