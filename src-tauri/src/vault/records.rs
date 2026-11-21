//! Payment cards and contact details kept in the vault, for autofill. They are encrypted like
//! logins, with the same data key. A card never stores its security code (CVV/CVC): the person
//! types it, as in other browsers. The list only ever shows the last four digits of a card; the full
//! number leaves the backend after the person proves who they are.
use super::crypto::{self, CryptoError};
use super::model::valid_id;
use super::service::VaultService;
use crate::database::vault_repo as repo;
use crate::error_log::{self, Level};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use zeroize::Zeroizing;

pub const KIND_CARD: &str = "card";
pub const KIND_CONTACT: &str = "contact";
pub const MAX_RECORDS_PER_KIND: usize = 200;
const MAX_TEXT: usize = 200;
const MAX_LONG_TEXT: usize = 500;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Card {
    pub nickname: String,
    pub holder: String,
    /// Digits only, 12 to 19, passing the Luhn check.
    pub number: String,
    pub exp_month: u8,
    pub exp_year: u16,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Contact {
    pub name: String,
    pub organization: String,
    pub email: String,
    pub phone: String,
    pub street: String,
    pub street2: String,
    pub city: String,
    pub region: String,
    pub postal_code: String,
    pub country: String,
}

/// What the interface sends to save, and gets back when it opens one.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Record {
    Card(Card),
    Contact(Contact),
}

/// A row of the list: enough to recognise it, never the full card number.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RecordSummary {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub subtitle: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RecordListing {
    pub records: Vec<RecordSummary>,
    pub unreadable: usize,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RecordRevealed {
    pub id: String,
    #[serde(flatten)]
    pub record: Record,
    pub updated_at: i64,
}

pub fn luhn(number: &str) -> bool {
    let mut sum = 0;
    for (index, digit) in number.bytes().rev().enumerate() {
        let mut value = u32::from(digit.wrapping_sub(b'0'));
        if index % 2 == 1 {
            value *= 2;
            if value > 9 {
                value -= 9;
            }
        }
        sum += value;
    }
    sum % 10 == 0
}

/// Removes the spaces and dashes people type between groups of digits.
pub fn clean_number(raw: &str) -> String {
    raw.chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect()
}

pub fn brand(number: &str) -> &'static str {
    let two: u32 = number.get(..2).and_then(|n| n.parse().ok()).unwrap_or(0);
    let four: u32 = number.get(..4).and_then(|n| n.parse().ok()).unwrap_or(0);
    match number.as_bytes().first() {
        Some(b'4') => "Visa",
        _ if (51..=55).contains(&two) || (2221..=2720).contains(&four) => "Mastercard",
        _ if two == 34 || two == 37 => "American Express",
        _ if two == 36 || two == 38 || (3000..=3059).contains(&four) => "Diners Club",
        _ if four == 6011 || two == 65 => "Discover",
        _ if (3528..=3589).contains(&four) => "JCB",
        _ => "Card",
    }
}

impl Card {
    pub fn validate(&mut self) -> Result<(), String> {
        self.number = clean_number(&self.number);
        let digits = self.number.len();
        if !(12..=19).contains(&digits)
            || !self.number.bytes().all(|b| b.is_ascii_digit())
            || !luhn(&self.number)
            || !(1..=12).contains(&self.exp_month)
            || !(2000..=2100).contains(&self.exp_year)
            || self.nickname.chars().count() > MAX_TEXT
            || self.holder.chars().count() > MAX_TEXT
        {
            return Err("vault.invalid_record".into());
        }
        Ok(())
    }

    fn summary(&self, id: &str, updated_at: i64) -> RecordSummary {
        let last4 = &self.number[self.number.len().saturating_sub(4)..];
        let name = if self.nickname.trim().is_empty() {
            brand(&self.number)
        } else {
            self.nickname.trim()
        };
        RecordSummary {
            id: id.into(),
            kind: KIND_CARD.into(),
            title: format!("{name} •••• {last4}"),
            subtitle: format!(
                "{} {:02}/{:02}",
                self.holder.trim(),
                self.exp_month,
                self.exp_year % 100
            )
            .trim()
            .to_string(),
            updated_at,
        }
    }
}

impl Contact {
    fn fields(&self) -> [&String; 10] {
        [
            &self.name,
            &self.organization,
            &self.email,
            &self.phone,
            &self.street,
            &self.street2,
            &self.city,
            &self.region,
            &self.postal_code,
            &self.country,
        ]
    }

    pub fn validate(&mut self) -> Result<(), String> {
        let fields = self.fields();
        let empty = fields.iter().all(|field| field.trim().is_empty());
        let long = fields
            .iter()
            .any(|field| field.chars().count() > MAX_LONG_TEXT);
        let email_ok = self.email.trim().is_empty()
            || (self.email.contains('@') && !self.email.contains(char::is_whitespace));
        if empty || long || !email_ok {
            return Err("vault.invalid_record".into());
        }
        Ok(())
    }

    fn summary(&self, id: &str, updated_at: i64) -> RecordSummary {
        let title = [&self.name, &self.organization, &self.email]
            .into_iter()
            .find(|text| !text.trim().is_empty())
            .cloned()
            .unwrap_or_default();
        let subtitle = [&self.email, &self.phone, &self.city]
            .into_iter()
            .filter(|text| !text.trim().is_empty() && **text != title)
            .cloned()
            .collect::<Vec<_>>()
            .join(" · ");
        RecordSummary {
            id: id.into(),
            kind: KIND_CONTACT.into(),
            title,
            subtitle,
            updated_at,
        }
    }
}

impl Record {
    pub fn kind(&self) -> &'static str {
        match self {
            Record::Card(_) => KIND_CARD,
            Record::Contact(_) => KIND_CONTACT,
        }
    }

    pub fn validate(&mut self) -> Result<(), String> {
        match self {
            Record::Card(card) => card.validate(),
            Record::Contact(contact) => contact.validate(),
        }
    }

    pub fn summary(&self, id: &str, updated_at: i64) -> RecordSummary {
        match self {
            Record::Card(card) => card.summary(id, updated_at),
            Record::Contact(contact) => contact.summary(id, updated_at),
        }
    }
}

fn fail(error: CryptoError, place: &str) -> String {
    error_log::report(
        Level::Error,
        "vault",
        &format!("{place} failed: {}", error.code()),
        "",
    );
    error.code().into()
}

fn new_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|_| fail(CryptoError::Unavailable, "creating a record id"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn valid_kind(kind: &str) -> bool {
    kind == KIND_CARD || kind == KIND_CONTACT
}

fn open(key: &crypto::Key, row: &repo::RecordRow) -> Result<Record, String> {
    let plain = crypto::open_entry(key, &row.id, &row.blob)
        .map_err(|e| fail(e, "reading a vault record"))?;
    let record: Record = serde_json::from_slice(&plain)
        .map_err(|_| fail(CryptoError::Corrupt, "reading a vault record"))?;
    if record.kind() != row.kind {
        return Err(fail(CryptoError::Corrupt, "reading a vault record"));
    }
    Ok(record)
}

pub fn list(
    conn: &Connection,
    service: &VaultService,
    kind: &str,
    now: Instant,
) -> Result<RecordListing, String> {
    let key = service.key(now)?;
    if !valid_kind(kind) {
        return Err("vault.invalid_record".into());
    }
    let mut records = Vec::new();
    let mut unreadable = 0;
    for row in repo::records(conn, kind)? {
        match open(&key, &row) {
            Ok(record) => records.push(record.summary(&row.id, row.updated_at)),
            Err(_) => unreadable += 1,
        }
    }
    records.sort_by_key(|record| record.title.to_lowercase());
    Ok(RecordListing {
        records,
        unreadable,
    })
}

/// Opens one record in full. For a card this is the step that shows the whole number, so the caller
/// must have confirmed the person first (`ops::confirm_reveal`); contacts need only an open vault.
pub fn reveal(
    conn: &Connection,
    service: &VaultService,
    id: &str,
    now: Instant,
) -> Result<RecordRevealed, String> {
    let key = service.key(now)?;
    if !valid_id(id) {
        return Err("vault.not_found".into());
    }
    let row = repo::record(conn, id)?.ok_or("vault.not_found")?;
    let record = open(&key, &row)?;
    Ok(RecordRevealed {
        id: row.id,
        record,
        updated_at: row.updated_at,
    })
}

/// The kind of a stored record, so the command can decide whether to ask for proof first.
pub fn kind_of(conn: &Connection, id: &str) -> Result<String, String> {
    if !valid_id(id) {
        return Err("vault.not_found".into());
    }
    Ok(repo::record(conn, id)?.ok_or("vault.not_found")?.kind)
}

/// Adds a record (`id` is None) or replaces one of the same kind.
pub fn save(
    conn: &Connection,
    service: &VaultService,
    id: Option<&str>,
    mut record: Record,
    timestamp: i64,
    now: Instant,
) -> Result<RecordSummary, String> {
    let key = service.key(now)?;
    record.validate()?;
    let kind = record.kind();
    let id = match id {
        Some(id) => {
            if !valid_id(id) || repo::record(conn, id)?.is_none_or(|row| row.kind != kind) {
                return Err("vault.not_found".into());
            }
            id.to_string()
        }
        None => {
            if repo::record_count(conn, kind)? >= MAX_RECORDS_PER_KIND {
                return Err("vault.full".into());
            }
            new_id()?
        }
    };
    let plain = Zeroizing::new(serde_json::to_vec(&record).map_err(|e| e.to_string())?);
    let blob =
        crypto::seal_entry(&key, &id, &plain).map_err(|e| fail(e, "saving a vault record"))?;
    repo::put_record(conn, &id, kind, &blob, timestamp)?;
    Ok(record.summary(&id, timestamp))
}

pub fn delete(
    conn: &Connection,
    service: &VaultService,
    id: &str,
    now: Instant,
) -> Result<(), String> {
    service.key(now)?;
    if !valid_id(id) || !repo::delete_record(conn, id)? {
        return Err("vault.not_found".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Connection, VaultService, Instant) {
        let conn = Connection::open_in_memory().unwrap();
        repo::migrate(&conn).unwrap();
        let service = VaultService::default();
        let now = Instant::now();
        service.unlock(
            crypto::create(Some("correct horse battery"), None)
                .unwrap()
                .1,
            now,
        );
        (conn, service, now)
    }

    fn card(number: &str) -> Record {
        Record::Card(Card {
            nickname: "Main".into(),
            holder: "Ana Souza".into(),
            number: number.into(),
            exp_month: 7,
            exp_year: 2031,
        })
    }

    #[test]
    fn card_numbers_must_pass_the_luhn_check_and_spaces_are_ignored() {
        assert!(luhn("4242424242424242"));
        assert!(!luhn("4242424242424241"));
        let (conn, service, now) = setup();
        assert!(save(&conn, &service, None, card("4242 4242 4242 4242"), 1, now).is_ok());
        for bad in ["4242424242424241", "1234", "abcd424242424242", ""] {
            assert!(
                save(&conn, &service, None, card(bad), 1, now).is_err(),
                "{bad}"
            );
        }
        let mut late = card("4242424242424242");
        if let Record::Card(c) = &mut late {
            c.exp_month = 13;
        }
        assert!(save(&conn, &service, None, late, 1, now).is_err());
    }

    #[test]
    fn the_list_shows_only_the_last_four_digits_and_the_full_number_needs_a_reveal() {
        let (conn, service, now) = setup();
        let summary = save(&conn, &service, None, card("4242424242424242"), 5, now).unwrap();
        assert_eq!(summary.title, "Main •••• 4242");
        assert_eq!(summary.subtitle, "Ana Souza 07/31");
        let json = serde_json::to_string(&list(&conn, &service, KIND_CARD, now).unwrap()).unwrap();
        assert!(!json.contains("42424242"));
        let revealed = reveal(&conn, &service, &summary.id, now).unwrap();
        assert!(matches!(revealed.record, Record::Card(c) if c.number == "4242424242424242"));
    }

    #[test]
    fn records_are_encrypted_and_need_an_open_vault() {
        let (conn, service, now) = setup();
        let summary = save(&conn, &service, None, card("4242424242424242"), 1, now).unwrap();
        let row = repo::record(&conn, &summary.id).unwrap().unwrap();
        assert!(!String::from_utf8_lossy(&row.blob).contains("4242"));
        service.lock();
        assert_eq!(
            list(&conn, &service, KIND_CARD, now).unwrap_err(),
            "vault.locked"
        );
        assert!(reveal(&conn, &service, &summary.id, now).is_err());
        assert!(delete(&conn, &service, &summary.id, now).is_err());
    }

    #[test]
    fn contacts_need_something_in_them_and_a_plausible_email() {
        let (conn, service, now) = setup();
        let empty = Record::Contact(Contact::default());
        assert!(save(&conn, &service, None, empty, 1, now).is_err());
        let bad = Record::Contact(Contact {
            email: "not an email".into(),
            ..Contact::default()
        });
        assert!(save(&conn, &service, None, bad, 1, now).is_err());
        let ok = Record::Contact(Contact {
            name: "Ana".into(),
            email: "ana@example.test".into(),
            city: "Recife".into(),
            ..Contact::default()
        });
        let summary = save(&conn, &service, None, ok, 1, now).unwrap();
        assert_eq!(summary.title, "Ana");
        assert_eq!(summary.subtitle, "ana@example.test · Recife");
    }

    #[test]
    fn a_record_keeps_its_kind_and_can_be_edited_and_deleted() {
        let (conn, service, now) = setup();
        let summary = save(&conn, &service, None, card("4242424242424242"), 1, now).unwrap();
        let other_kind = Record::Contact(Contact {
            name: "Ana".into(),
            ..Contact::default()
        });
        assert!(save(&conn, &service, Some(&summary.id), other_kind, 2, now).is_err());
        let edited = save(
            &conn,
            &service,
            Some(&summary.id),
            card("5555555555554444"),
            2,
            now,
        )
        .unwrap();
        assert_eq!(edited.id, summary.id);
        assert_eq!(kind_of(&conn, &summary.id).unwrap(), KIND_CARD);
        assert_eq!(
            list(&conn, &service, KIND_CONTACT, now)
                .unwrap()
                .records
                .len(),
            0
        );
        delete(&conn, &service, &summary.id, now).unwrap();
        assert_eq!(
            delete(&conn, &service, &summary.id, now).unwrap_err(),
            "vault.not_found"
        );
        assert!(save(
            &conn,
            &service,
            Some("not-an-id"),
            card("4242424242424242"),
            1,
            now
        )
        .is_err());
    }

    #[test]
    fn card_brands_are_recognised() {
        assert_eq!(brand("4242424242424242"), "Visa");
        assert_eq!(brand("5555555555554444"), "Mastercard");
        assert_eq!(brand("378282246310005"), "American Express");
    }
}
