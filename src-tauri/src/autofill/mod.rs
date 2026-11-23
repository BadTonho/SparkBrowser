//! Autofill in web pages: suggestions and filling for logins, cards and contact details kept in the
//! vault. The page script (`autofill.js`) finds the fields and asks for suggestions when one is
//! focused. Here the browser decides what to offer, asks for confirmation when the vault is set to
//! (Windows Hello each time) and fills the fields after the person picks an item.
//!
//! Rules this module keeps:
//! - the page never holds saved data beforehand: only titles of the items to pick from;
//! - an id sent by a page is checked again here (same document, same site, right kind);
//! - a login is only offered on a page of the site it was saved for, and never over plain HTTP for a
//!   site saved with HTTPS;
//! - secrets are never written to the error log.
mod matching;
mod payload;
mod save;

use crate::database::{executor, preferences_repo};
use crate::error_log::{self, Level};
use crate::state::browser_state::BrowserState;
use crate::vault::{
    hello, ops,
    records::{self, Record},
    service::VaultService,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Manager, Url};

pub const ENABLED_KEY: &str = "autofill_enabled";
pub const PRIVATE_KEY: &str = "autofill_in_private";
const MAX_SUGGESTIONS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Login,
    Card,
    Contact,
}

impl Kind {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "login" => Some(Self::Login),
            "card" => Some(Self::Card),
            "contact" => Some(Self::Contact),
            _ => None,
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Card => "card",
            Self::Contact => "contact",
        }
    }
}

/// The two switches in Settings → Autofill & passwords.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Settings {
    /// Offer to fill in pages at all.
    pub enabled: bool,
    /// Also offer it in private windows (they then get a read-only copy of the vault).
    pub in_private: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            in_private: false,
        }
    }
}

/// What a page asked about, so the next step can be checked against it.
#[derive(Clone)]
struct Pending {
    document: String,
    page: String,
    kind: Kind,
}

#[derive(Default)]
pub struct AutofillService {
    disabled: AtomicBool,
    in_private: AtomicBool,
    pending: Mutex<HashMap<String, Pending>>,
    saves: Mutex<HashMap<String, save::PendingSave>>,
    declined: Mutex<std::collections::HashSet<String>>,
}

impl AutofillService {
    pub fn settings(&self) -> Settings {
        Settings {
            enabled: !self.disabled.load(Ordering::Relaxed),
            in_private: self.in_private.load(Ordering::Relaxed),
        }
    }
    pub fn apply(&self, settings: Settings) {
        self.disabled.store(!settings.enabled, Ordering::Relaxed);
        self.in_private
            .store(settings.in_private, Ordering::Relaxed);
    }
    fn remember(&self, label: &str, pending: Pending) {
        if let Ok(mut map) = self.pending.lock() {
            map.insert(label.to_string(), pending);
        }
    }
    fn keep_save(&self, label: &str, pending: save::PendingSave) {
        if let Ok(mut map) = self.saves.lock() {
            map.insert(label.to_string(), pending);
        }
    }
    fn peek_save(&self, label: &str) -> Option<save::PendingSave> {
        self.saves.lock().ok()?.get(label).cloned()
    }
    fn take_save(&self, label: &str) -> Option<save::PendingSave> {
        self.saves.lock().ok()?.remove(label)
    }
    fn is_declined(&self, key: &str) -> bool {
        self.declined.lock().is_ok_and(|set| set.contains(key))
    }
    fn decline(&self, key: String) {
        if let Ok(mut set) = self.declined.lock() {
            set.insert(key);
        }
    }
    fn recall(&self, label: &str) -> Option<Pending> {
        self.pending.lock().ok()?.get(label).cloned()
    }
}

pub fn load_settings(conn: &rusqlite::Connection) -> Result<Settings, String> {
    Ok(Settings {
        enabled: preferences_repo::load(conn, ENABLED_KEY)?.as_deref() != Some("false"),
        in_private: preferences_repo::load_bool(conn, PRIVATE_KEY)?,
    })
}

pub fn save_settings(conn: &rusqlite::Connection, settings: Settings) -> Result<(), String> {
    preferences_repo::store(conn, ENABLED_KEY, &settings.enabled.to_string())?;
    preferences_repo::store(conn, PRIVATE_KEY, &settings.in_private.to_string())
}

/// Whether autofill may run in this process (a private window needs the second switch).
fn active(app: &AppHandle) -> bool {
    let settings = app.state::<AutofillService>().settings();
    let private = app
        .try_state::<crate::private_session::PrivateRole>()
        .is_some_and(|role| role.is_private());
    settings.enabled && (!private || settings.in_private)
}

fn function_name(token: &str) -> String {
    let short: String = token
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(24)
        .collect();
    format!("__sparkAF{short}")
}

/// The page script of one web view, bound to its bridge token.
pub fn initializer(token: &str, inspectable: bool) -> String {
    include_str!("autofill.js")
        .replace(
            "__AUTOFILL_TOKEN__",
            &serde_json::to_string(token).expect("token serializable"),
        )
        .replace(
            "__AUTOFILL_FN__",
            &serde_json::to_string(&function_name(token)).expect("name serializable"),
        )
        // Closed, so the page cannot read the list. Open only in the isolated native-test build,
        // where WebDriver has to look inside it.
        .replace(
            "__AUTOFILL_SHADOW__",
            if inspectable { "'open'" } else { "'closed'" },
        )
        .replace("__AUTOFILL_ROLES__", include_str!("roles.js"))
}

fn send(app: &AppHandle, label: &str, token: &str, message: &serde_json::Value) {
    let Some(view) = app.get_webview(label) else {
        return;
    };
    let name = serde_json::to_string(&function_name(token)).unwrap_or_default();
    let body = message.to_string();
    error_log::ignored(view.eval(format!("window[{name}]&&window[{name}]({body});")));
}

fn page_of(app: &AppHandle, label: &str) -> Option<Url> {
    let url = app.get_webview(label)?.url().ok()?;
    let text = url.as_str();
    let web = matches!(url.scheme(), "http" | "https");
    (web && !crate::window::new_tab::is_new_tab(text)
        && !crate::window::settings_tab::is_settings_url(text)
        && !crate::window::wallpaper_tab::is_wallpaper_url(text))
    .then_some(url)
}

/// A message from the page script (`spark-internal://autofill/...`). The caller has already checked
/// the token and the document generation; this checks the sequence and the fields.
pub(crate) fn handle(app: &AppHandle, label: &str, token: &str, url: &Url) {
    let Some(values) = crate::security::bridge::query(
        url,
        &[
            "token", "document", "sequence", "kind", "role", "x", "y", "w", "h", "id", "data",
            "origin",
        ],
    ) else {
        return;
    };
    let (Some(document), Some(sequence), Some(kind)) = (
        values.get("document"),
        values.get("sequence").and_then(|v| v.parse::<u64>().ok()),
        values.get("kind").and_then(|v| Kind::parse(v)),
    ) else {
        return;
    };
    if values.get("token").map(String::as_str) != Some(token)
        || !crate::window::navigation_state::is_current_generation(app, label, token)
    {
        return;
    }
    let admitted = app
        .state::<BrowserState>()
        .security()
        .keyboard_bridge_sequences()
        .lock()
        .is_ok_and(|mut states| states.admit(&format!("{label}:autofill"), document, sequence));
    if !admitted || !active(app) {
        return;
    }
    let Some(page) = page_of(app, label) else {
        return;
    };
    let (app, label, token, document) = (
        app.clone(),
        label.to_string(),
        token.to_string(),
        document.clone(),
    );
    let id = values.get("id").cloned();
    match url.path() {
        "/capture" => {
            let Some(data) = values
                .get("data")
                .and_then(|text| serde_json::from_str::<HashMap<String, String>>(text).ok())
            else {
                return;
            };
            let origin = values.get("origin").cloned().unwrap_or_default();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = save::capture(&app, &label, &token, kind, data, &origin).await {
                    report("could not look at a submitted form", &error);
                }
            });
        }
        "/ready" => save::ready(&app, &label, &token),
        "/dismiss" => save::dismiss(&app, &label),
        "/save" => {
            tauri::async_runtime::spawn(async move {
                if let Err(error) = save::confirm(&app, &label).await {
                    report("could not save what was submitted", &error);
                }
            });
        }
        "/request" => {
            app.state::<AutofillService>().remember(
                &label,
                Pending {
                    document,
                    page: page.to_string(),
                    kind,
                },
            );
            tauri::async_runtime::spawn(async move {
                if let Err(error) = suggest(&app, &label, &token, kind, &page).await {
                    report("could not prepare autofill suggestions", &error);
                }
            });
        }
        "/choose" => {
            let Some(id) = id else { return };
            tauri::async_runtime::spawn(async move {
                if let Err(error) = choose(&app, &label, &token, &document, kind, &id).await {
                    report("could not fill the page", &error);
                }
            });
        }
        "/unlock" => {
            tauri::async_runtime::spawn(async move {
                if let Err(error) = unlock(&app, &label, &token, &document, kind).await {
                    report("could not unlock the vault from a page", &error);
                }
            });
        }
        _ => {}
    }
}

/// Codes like `vault.hello_cancelled` are expected outcomes, not failures worth an error.
fn report(what: &str, code: &str) {
    let level = if code.starts_with("vault.hello_cancelled") || code == "vault.locked" {
        Level::Info
    } else {
        Level::Warning
    };
    error_log::report(level, "autofill", &format!("Autofill {what}"), code);
}

async fn suggest(
    app: &AppHandle,
    label: &str,
    token: &str,
    kind: Kind,
    page: &Url,
) -> Result<(), String> {
    let service = app.state::<VaultService>().inner().clone();
    let status = {
        let service = service.clone();
        executor::run(app.clone(), move |conn| {
            ops::status(conn, &service, Instant::now())
        })
        .await?
    };
    if !status.initialized {
        return Ok(());
    }
    if !status.unlocked {
        let hello = status.has_hello && hello::is_available();
        send(app, label, token, &payload::locked(hello));
        return Ok(());
    }
    let page = page.clone();
    let items = executor::run(app.clone(), move |conn| {
        items_for(conn, &service, kind, &page)
    })
    .await?;
    send(app, label, token, &payload::items(kind.as_str(), items));
    Ok(())
}

fn items_for(
    conn: &rusqlite::Connection,
    service: &VaultService,
    kind: Kind,
    page: &Url,
) -> Result<Vec<payload::Item>, String> {
    let now = Instant::now();
    let mut items = Vec::new();
    match kind {
        Kind::Login => {
            for entry in ops::list(conn, service, now)?.entries {
                if matching::login_applies(&entry.url, page) {
                    items.push(payload::Item {
                        id: entry.id,
                        title: if entry.username.is_empty() {
                            entry.title.clone()
                        } else {
                            entry.username.clone()
                        },
                        subtitle: entry.title,
                    });
                }
            }
        }
        Kind::Card | Kind::Contact => {
            let wanted = if kind == Kind::Card {
                records::KIND_CARD
            } else {
                records::KIND_CONTACT
            };
            for record in records::list(conn, service, wanted, now)?.records {
                items.push(payload::Item {
                    id: record.id,
                    title: record.title,
                    subtitle: record.subtitle,
                });
            }
        }
    }
    items.truncate(MAX_SUGGESTIONS);
    Ok(items)
}

/// The page and document of the last request must still be the ones showing, or nothing is filled.
fn still_current(app: &AppHandle, label: &str, document: &str, kind: Kind) -> Result<Url, String> {
    let pending = app
        .state::<AutofillService>()
        .recall(label)
        .ok_or("autofill.no_request")?;
    let page = page_of(app, label).ok_or("autofill.page_changed")?;
    if pending.document != document
        || pending.kind != kind
        || matching::origin(&pending.page) != matching::origin(page.as_str())
    {
        return Err("autofill.page_changed".into());
    }
    Ok(page)
}

async fn choose(
    app: &AppHandle,
    label: &str,
    token: &str,
    document: &str,
    kind: Kind,
    id: &str,
) -> Result<(), String> {
    let page = still_current(app, label, document, kind)?;
    let service = app.state::<VaultService>().inner().clone();
    service.key(Instant::now())?;
    // Logins and cards ask Windows Hello each time when the vault is set to; contact details do not.
    let needs_hello =
        kind != Kind::Contact && executor::run(app.clone(), ops::requires_hello).await?;
    let secret = if needs_hello {
        Some(
            crate::commands::vault::hello_step("filling a page from the vault", hello::secret)
                .await?,
        )
    } else {
        None
    };
    let id = id.to_string();
    let values = executor::run(app.clone(), move |conn| {
        if let Some(secret) = &secret {
            ops::confirm_hello(conn, secret)?;
        }
        values_for(conn, &service, kind, &id, &page)
    })
    .await?;
    send(app, label, token, &payload::fill(kind.as_str(), values));
    Ok(())
}

fn values_for(
    conn: &rusqlite::Connection,
    service: &VaultService,
    kind: Kind,
    id: &str,
    page: &Url,
) -> Result<HashMap<&'static str, String>, String> {
    let now = Instant::now();
    match kind {
        Kind::Login => {
            let entry = ops::reveal(conn, service, id, now)?;
            if !matching::login_applies(&entry.data.url, page) {
                return Err("autofill.wrong_site".into());
            }
            Ok(payload::login_values(&entry.data))
        }
        Kind::Card => match records::reveal(conn, service, id, now)?.record {
            Record::Card(card) => Ok(payload::card_values(&card)),
            Record::Contact(_) => Err("autofill.wrong_kind".into()),
        },
        Kind::Contact => match records::reveal(conn, service, id, now)?.record {
            Record::Contact(contact) => Ok(payload::contact_values(&contact)),
            Record::Card(_) => Err("autofill.wrong_kind".into()),
        },
    }
}

async fn unlock(
    app: &AppHandle,
    label: &str,
    token: &str,
    document: &str,
    kind: Kind,
) -> Result<(), String> {
    let page = still_current(app, label, document, kind)?;
    if !hello::is_available() {
        return Err("vault.hello_unavailable".into());
    }
    let service = app.state::<VaultService>().inner().clone();
    let secret =
        crate::commands::vault::hello_step("unlocking the vault from a page", hello::secret)
            .await?;
    {
        let service = service.clone();
        executor::run(app.clone(), move |conn| {
            ops::unlock_with_hello(conn, &service, &secret, Instant::now())
        })
        .await?;
    }
    suggest(app, label, token, kind, &page).await
}
