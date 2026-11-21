//! Test helpers for the password vault. Compiled only into debug builds (`npm run tauri dev`, tests):
//! a release build has none of this code, so it cannot be reached there.
//!
//! They add made-up logins (all marked in their notes) so the list can be tried with data, and remove
//! only those again, never anything the person saved.
use super::crypto;
use super::model::EntryData;
use super::ops::{self, ImportResult};
use super::service::VaultService;
use crate::database::vault_repo as repo;
use crate::error_log::{self, Level};
use rusqlite::Connection;
use std::time::Instant;

/// Every test entry starts its notes with this, which is how they are told apart from real ones.
pub const MARKER: &str = "[dados de teste]";

fn sample(title: &str, url: &str, username: &str, password: &str, note: &str) -> EntryData {
    EntryData {
        title: title.into(),
        url: url.into(),
        username: username.into(),
        password: password.into(),
        notes: format!("{MARKER} {note}").trim_end().to_string(),
    }
}

/// Made-up logins that exercise the list: several accounts on one site, a `www.` address, a login
/// without an address, a long title, notes with line breaks.
pub fn samples() -> Vec<EntryData> {
    vec![
        sample(
            "Crunchyroll",
            "https://www.crunchyroll.com/login",
            "otaku@exemplo.test",
            "teste-crunchyroll-01",
            "",
        ),
        sample(
            "Discord",
            "https://discord.com/login",
            "amigo@exemplo.test",
            "teste-discord-02",
            "conta principal",
        ),
        sample(
            "Qt",
            "https://login.qt.io",
            "dev@exemplo.test",
            "teste-qt-03",
            "",
        ),
        sample(
            "Reserva",
            "https://reserva.ink",
            "leitor@exemplo.test",
            "teste-reserva-04",
            "",
        ),
        sample(
            "Roblox",
            "https://www.roblox.com/home",
            "jogador",
            "teste-roblox-05",
            "",
        ),
        sample(
            "Supabase",
            "https://supabase.com/dashboard",
            "dev@exemplo.test",
            "teste-supabase-06",
            "projeto de desenvolvimento",
        ),
        sample(
            "Supabase",
            "https://supabase.com/dashboard",
            "prod@exemplo.test",
            "teste-supabase-07",
            "projeto de producao",
        ),
        sample(
            "Supabase",
            "https://supabase.com/dashboard",
            "teste@exemplo.test",
            "teste-supabase-08",
            "",
        ),
        sample(
            "GitHub",
            "https://github.com/login",
            "pessoal",
            "teste-github-09",
            "",
        ),
        sample(
            "GitHub",
            "https://github.com/login",
            "trabalho",
            "teste-github-10",
            "linha 1\nlinha 2",
        ),
        sample(
            "Wi-Fi de casa",
            "",
            "roteador",
            "teste-wifi-11",
            "um login sem endereco de site",
        ),
        sample(
            "Banco com um nome bem comprido para ver como a lista quebra a linha",
            "https://internetbanking.um-endereco-bem-comprido-para-testar.exemplo.test/entrar",
            "cliente-com-um-nome-de-usuario-tambem-comprido@exemplo.test",
            "teste-banco-12",
            "",
        ),
    ]
}

/// Adds the made-up logins. Ones already there are not added again, so pressing it twice is harmless.
pub fn seed(
    conn: &Connection,
    service: &VaultService,
    timestamp: i64,
    now: Instant,
) -> Result<ImportResult, String> {
    ops::import_entries(conn, service, samples(), timestamp, now)
}

/// Removes the made-up logins (and only them). Returns how many were removed.
pub fn remove(conn: &Connection, service: &VaultService, now: Instant) -> Result<usize, String> {
    let key = service.key(now)?;
    let mut removed = 0;
    for row in repo::rows(conn)? {
        let is_test = crypto::open_entry(&key, &row.id, &row.blob)
            .ok()
            .and_then(|plain| serde_json::from_slice::<EntryData>(&plain).ok())
            .is_some_and(|data| data.notes.starts_with(MARKER));
        if is_test && repo::delete(conn, &row.id)? {
            removed += 1;
        }
    }
    error_log::report(
        Level::Info,
        "vault",
        &format!("{removed} test passwords were removed from the vault"),
        "",
    );
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logins_are_added_once_and_removed_without_touching_real_ones() {
        let conn = Connection::open_in_memory().unwrap();
        repo::migrate(&conn).unwrap();
        let (service, now) = (VaultService::default(), Instant::now());
        ops::create(&conn, &service, Some("correct horse battery"), None, now).unwrap();
        let real = EntryData {
            title: "Real".into(),
            password: "pw".into(),
            ..EntryData::default()
        };
        ops::save(&conn, &service, None, &real, 1, now).unwrap();

        let first = seed(&conn, &service, 2, now).unwrap();
        assert_eq!(
            first,
            ImportResult {
                imported: samples().len(),
                duplicates: 0,
                skipped: 0
            }
        );
        let again = seed(&conn, &service, 3, now).unwrap();
        assert_eq!(
            again,
            ImportResult {
                imported: 0,
                duplicates: samples().len(),
                skipped: 0
            }
        );
        assert_eq!(
            ops::list(&conn, &service, now).unwrap().entries.len(),
            samples().len() + 1
        );

        assert_eq!(remove(&conn, &service, now).unwrap(), samples().len());
        let left = ops::list(&conn, &service, now).unwrap().entries;
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].title, "Real");

        service.lock();
        assert_eq!(seed(&conn, &service, 4, now).unwrap_err(), "vault.locked");
        assert_eq!(remove(&conn, &service, now).unwrap_err(), "vault.locked");
    }

    #[test]
    fn every_sample_is_valid_and_marked() {
        for entry in samples() {
            assert!(entry.validate().is_ok(), "{}", entry.title);
            assert!(entry.notes.starts_with(MARKER));
            assert!(entry.password.starts_with("teste-"));
        }
    }
}
