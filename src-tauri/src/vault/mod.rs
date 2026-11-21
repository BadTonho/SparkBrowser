//! Native password vault: encrypted local storage of logins, managed from Settings.
//! Not independently audited yet; see `docs/en/security/account-containers-vault-review.md`.
pub mod clipboard;
pub mod crypto;
pub mod csv;
#[cfg(debug_assertions)]
pub mod dev;
pub mod hello;
pub mod model;
pub mod ops;
pub mod service;
