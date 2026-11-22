//! "Save this?" after a form is submitted. The page script reports what was submitted; the browser
//! compares it with the vault, remembers what could be saved and shows a bar in the page (again on
//! the next page when the submit redirects). Nothing is saved until the person clicks Save.
//!
//! Only with an open vault, never in a private window (its copy of the vault is read-only), and a
//! question that was declined is not asked again for the same site until Spark restarts.
use super::{matching, page_of, send, AutofillService, Kind};
use crate::database::executor;
use crate::vault::{
    model::EntryData,
    ops,
    records::{self, Card, Contact, Record},
    service::VaultService,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Url};

/// How long a question waits for the page that follows the submit.
const KEEP: Duration = Duration::from_secs(90);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Save,
    Update,
}

#[derive(Clone)]
enum Payload {
    Login {
        username: String,
        password: String,
        update: Option<String>,
    },
    Card(Card),
    Contact(Contact),
}

#[derive(Clone)]
pub(super) struct PendingSave {
    payload: Payload,
    /// Address the form was on (`https://example.com`).
    site: String,
    subject: String,
    created: Instant,
}

impl PendingSave {
    fn kind(&self) -> Kind {
        match self.payload {
            Payload::Login { .. } => Kind::Login,
            Payload::Card(_) => Kind::Card,
            Payload::Contact(_) => Kind::Contact,
        }
    }
    fn mode(&self) -> Mode {
        match &self.payload {
            Payload::Login {
                update: Some(_), ..
            } => Mode::Update,
            _ => Mode::Save,
        }
    }
    fn message(&self) -> serde_json::Value {
        json!({
            "op": "prompt",
            "kind": self.kind().as_str(),
            "mode": if self.mode() == Mode::Update { "update" } else { "save" },
            "subject": self.subject,
        })
    }
    fn declined_key(&self) -> String {
        format!(
            "{}|{}|{}",
            self.kind().as_str(),
            matching::display_host(&self.site),
            self.subject
        )
    }
}

fn related(pending: &PendingSave, page: &Url) -> bool {
    matching::same_family(&pending.site, page)
}

fn contact_from(values: &HashMap<String, String>) -> Contact {
    let get = |key: &str| values.get(key).cloned().unwrap_or_default();
    let name = if values.contains_key("name") {
        get("name")
    } else {
        [get("given-name"), get("family-name")]
            .iter()
            .filter(|part| !part.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let street = if values.contains_key("address-line1") {
        get("address-line1")
    } else {
        get("street-address")
    };
    Contact {
        name,
        organization: get("organization"),
        email: get("email"),
        phone: get("tel"),
        street,
        street2: get("address-line2"),
        city: get("address-level2"),
        region: get("address-level1"),
        postal_code: get("postal-code"),
        country: get("country-name"),
    }
}

fn card_from(values: &HashMap<String, String>) -> Option<Card> {
    let get = |key: &str| values.get(key).cloned().unwrap_or_default();
    let mut card = Card {
        nickname: String::new(),
        holder: get("cc-name"),
        number: get("cc-number"),
        exp_month: get("cc-exp-month").parse().ok()?,
        exp_year: get("cc-exp-year").parse().ok()?,
    };
    card.validate().ok()?;
    Some(card)
}

/// Decides, against the vault, whether the submitted values are worth asking about.
fn decide(
    conn: &rusqlite::Connection,
    service: &VaultService,
    kind: Kind,
    values: &HashMap<String, String>,
    site: &str,
    page: &Url,
) -> Result<Option<PendingSave>, String> {
    let now = Instant::now();
    let created = Instant::now();
    match kind {
        Kind::Login => {
            let password = values.get("password").cloned().unwrap_or_default();
            let username = values.get("username").cloned().unwrap_or_default();
            if password.is_empty() {
                return Ok(None);
            }
            let mut update = None;
            for entry in ops::list(conn, service, now)?.entries {
                if matching::login_applies(&entry.url, page) && entry.username == username {
                    if ops::reveal(conn, service, &entry.id, now)?.data.password == password {
                        return Ok(None);
                    }
                    update = Some(entry.id);
                    break;
                }
            }
            Ok(Some(PendingSave {
                payload: Payload::Login {
                    username,
                    password,
                    update,
                },
                site: site.to_string(),
                subject: matching::display_host(site),
                created,
            }))
        }
        Kind::Card => {
            let Some(card) = card_from(values) else {
                return Ok(None);
            };
            let last4 = card.number[card.number.len() - 4..].to_string();
            for row in records::list(conn, service, records::KIND_CARD, now)?.records {
                if row.title.ends_with(&last4) {
                    if let Record::Card(saved) =
                        records::reveal(conn, service, &row.id, now)?.record
                    {
                        if saved.number == card.number {
                            return Ok(None);
                        }
                    }
                }
            }
            let subject = format!("{} •••• {last4}", records::brand(&card.number));
            Ok(Some(PendingSave {
                payload: Payload::Card(card),
                site: site.to_string(),
                subject,
                created,
            }))
        }
        Kind::Contact => {
            let mut contact = contact_from(values);
            let addressed = [
                &contact.street,
                &contact.city,
                &contact.postal_code,
                &contact.phone,
            ]
            .iter()
            .any(|field| !field.trim().is_empty());
            if !addressed || contact.validate().is_err() {
                return Ok(None);
            }
            for row in records::list(conn, service, records::KIND_CONTACT, now)?.records {
                if let Record::Contact(saved) = records::reveal(conn, service, &row.id, now)?.record
                {
                    if saved == contact {
                        return Ok(None);
                    }
                }
            }
            let subject = [&contact.name, &contact.email, &contact.street]
                .into_iter()
                .find(|text| !text.trim().is_empty())
                .cloned()
                .unwrap_or_default();
            Ok(Some(PendingSave {
                payload: Payload::Contact(contact),
                site: site.to_string(),
                subject,
                created,
            }))
        }
    }
}

/// A form was submitted. Remembers a question about it, and asks it on this page.
pub(super) async fn capture(
    app: &AppHandle,
    label: &str,
    token: &str,
    kind: Kind,
    values: HashMap<String, String>,
    claimed_origin: &str,
) -> Result<(), String> {
    if app
        .try_state::<crate::private_session::PrivateRole>()
        .is_some_and(|role| role.is_private())
    {
        return Ok(());
    }
    let Some(page) = page_of(app, label) else {
        return Ok(());
    };
    let service = app.state::<VaultService>().inner().clone();
    if !service.is_unlocked(Instant::now()) {
        return Ok(());
    }
    // The page says where the form was; that only counts when the page showing now is of that site.
    let site = if matching::login_applies(claimed_origin, &page) {
        claimed_origin.to_string()
    } else {
        page.origin().ascii_serialization()
    };
    let decided = {
        let (page, site) = (page.clone(), site.clone());
        executor::run(app.clone(), move |conn| {
            decide(conn, &service, kind, &values, &site, &page)
        })
        .await?
    };
    let Some(pending) = decided else {
        return Ok(());
    };
    let autofill = app.state::<AutofillService>();
    if autofill.is_declined(&pending.declined_key()) {
        return Ok(());
    }
    send(app, label, token, &pending.message());
    autofill.keep_save(label, pending);
    Ok(())
}

/// A page announced itself: a question left over from the last submit is asked here, if it is the
/// same site and recent.
pub(super) fn ready(app: &AppHandle, label: &str, token: &str) {
    let autofill = app.state::<AutofillService>();
    let Some(pending) = autofill.peek_save(label) else {
        return;
    };
    if pending.created.elapsed() > KEEP {
        autofill.take_save(label);
        return;
    }
    if page_of(app, label).is_some_and(|page| related(&pending, &page)) {
        send(app, label, token, &pending.message());
    }
}

pub(super) fn dismiss(app: &AppHandle, label: &str) {
    let autofill = app.state::<AutofillService>();
    if let Some(pending) = autofill.take_save(label) {
        autofill.decline(pending.declined_key());
    }
}

/// The person clicked Save (or Update).
pub(super) async fn confirm(app: &AppHandle, label: &str) -> Result<(), String> {
    let autofill = app.state::<AutofillService>();
    let Some(pending) = autofill.take_save(label) else {
        return Err("autofill.no_request".into());
    };
    let page = page_of(app, label).ok_or("autofill.page_changed")?;
    if pending.created.elapsed() > Duration::from_secs(300) || !related(&pending, &page) {
        return Err("autofill.page_changed".into());
    }
    let service = app.state::<VaultService>().inner().clone();
    executor::run(app.clone(), move |conn| {
        let now = Instant::now();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs() as i64);
        match pending.payload {
            Payload::Login {
                username,
                password,
                update,
            } => {
                let mut data = match &update {
                    Some(id) => ops::reveal(conn, &service, id, now)?.data,
                    None => EntryData {
                        title: matching::display_host(&pending.site),
                        url: pending.site.clone(),
                        username,
                        ..EntryData::default()
                    },
                };
                data.password = password;
                ops::save(conn, &service, update.as_deref(), &data, stamp, now).map(|_| ())
            }
            Payload::Card(card) => {
                records::save(conn, &service, None, Record::Card(card), stamp, now).map(|_| ())
            }
            Payload::Contact(contact) => {
                records::save(conn, &service, None, Record::Contact(contact), stamp, now)
                    .map(|_| ())
            }
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::vault_repo as repo;
    use crate::vault::crypto;

    fn setup() -> (rusqlite::Connection, VaultService) {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        repo::migrate(&conn).unwrap();
        let service = VaultService::default();
        service.unlock(
            crypto::create(Some("correct horse battery"), None)
                .unwrap()
                .1,
            Instant::now(),
        );
        (conn, service)
    }

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn page() -> Url {
        Url::parse("https://example.com/login").unwrap()
    }

    fn login(
        conn: &rusqlite::Connection,
        service: &VaultService,
        password: &str,
    ) -> Option<PendingSave> {
        let values = map(&[("username", "ana"), ("password", password)]);
        decide(
            conn,
            service,
            Kind::Login,
            &values,
            "https://example.com",
            &page(),
        )
        .unwrap()
    }

    #[test]
    fn a_new_login_is_offered_then_a_same_or_changed_password_is_told_apart() {
        let (conn, service) = setup();
        let first = login(&conn, &service, "pw-one").expect("a new login is offered");
        assert!(first.mode() == Mode::Save);
        assert_eq!(first.subject, "example.com");
        ops::save(
            &conn,
            &service,
            None,
            &EntryData {
                title: "example.com".into(),
                url: "https://example.com".into(),
                username: "ana".into(),
                password: "pw-one".into(),
                notes: String::new(),
                totp: String::new(),
            },
            1,
            Instant::now(),
        )
        .unwrap();
        assert!(login(&conn, &service, "pw-one").is_none(), "already saved");
        let changed = login(&conn, &service, "pw-two").expect("a changed password is offered");
        assert!(changed.mode() == Mode::Update);
    }

    #[test]
    fn a_form_without_a_password_is_not_a_login() {
        let (conn, service) = setup();
        let values = map(&[("username", "ana"), ("password", "")]);
        let none = decide(
            &conn,
            &service,
            Kind::Login,
            &values,
            "https://example.com",
            &page(),
        );
        assert!(none.unwrap().is_none());
    }

    #[test]
    fn only_a_valid_new_card_is_offered() {
        let (conn, service) = setup();
        let card = |number: &str| {
            map(&[
                ("cc-number", number),
                ("cc-name", "Ana"),
                ("cc-exp-month", "7"),
                ("cc-exp-year", "2031"),
            ])
        };
        let site = "https://shop.test";
        let offer = |number: &str| {
            decide(&conn, &service, Kind::Card, &card(number), site, &page()).unwrap()
        };
        assert!(offer("4242 4242 4242 4241").is_none(), "fails the checksum");
        let pending = offer("4242 4242 4242 4242").expect("a new card is offered");
        assert_eq!(pending.subject, "Visa •••• 4242");
        let Payload::Card(saved) = pending.payload else {
            panic!("a card")
        };
        records::save(
            &conn,
            &service,
            None,
            Record::Card(saved),
            1,
            Instant::now(),
        )
        .unwrap();
        assert!(offer("4242424242424242").is_none(), "already saved");
    }

    #[test]
    fn contact_info_needs_an_address_or_phone_and_is_not_asked_twice() {
        let (conn, service) = setup();
        let site = "https://shop.test";
        let thin = map(&[
            ("name", "Ana"),
            ("email", "ana@example.test"),
            ("organization", "X"),
        ]);
        assert!(decide(&conn, &service, Kind::Contact, &thin, site, &page())
            .unwrap()
            .is_none());
        let full = map(&[
            ("given-name", "Ana"),
            ("family-name", "Souza"),
            ("email", "ana@example.test"),
            ("address-line1", "Rua A, 10"),
            ("address-level2", "Recife"),
        ]);
        let pending = decide(&conn, &service, Kind::Contact, &full, site, &page())
            .unwrap()
            .expect("an address is offered");
        assert_eq!(pending.subject, "Ana Souza");
        let Payload::Contact(contact) = pending.payload else {
            panic!("a contact")
        };
        assert_eq!(contact.street, "Rua A, 10");
        records::save(
            &conn,
            &service,
            None,
            Record::Contact(contact),
            1,
            Instant::now(),
        )
        .unwrap();
        assert!(decide(&conn, &service, Kind::Contact, &full, site, &page())
            .unwrap()
            .is_none());
    }

    #[test]
    fn a_question_belongs_to_its_site_and_declined_ones_are_remembered() {
        let (conn, service) = setup();
        let pending = login(&conn, &service, "pw").unwrap();
        assert!(related(
            &pending,
            &Url::parse("https://www.example.com/home").unwrap()
        ));
        assert!(related(
            &pending,
            &Url::parse("https://accounts.example.com/").unwrap()
        ));
        assert!(!related(
            &pending,
            &Url::parse("https://evil.test/").unwrap()
        ));
        let service = AutofillService::default();
        let key = pending.declined_key();
        assert!(!service.is_declined(&key));
        service.decline(key.clone());
        assert!(service.is_declined(&key));
    }
}
