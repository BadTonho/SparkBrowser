//! Native password vault: encrypted local storage of logins, managed from Settings.
//! Not independently audited yet; see `docs/en/security/account-containers-vault-review.md`.
pub mod crypto;
pub mod model;
pub mod ops;
pub mod service;
