//! Native password vault: encrypted local storage of logins, managed from Settings.
//! Not independently audited yet; see `docs/en/security/account-containers-vault-review.md`.
pub mod clipboard;
pub mod crypto;
pub mod csv;
pub mod hello;
pub mod model;
pub mod ops;
pub mod service;
