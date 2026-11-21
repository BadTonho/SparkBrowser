//! What the vault stores for one login, and what the interface may see of it.
use serde::{Deserialize, Serialize};

pub const MAX_TITLE: usize = 200;
pub const MAX_URL: usize = 2048;
pub const MAX_USERNAME: usize = 320;
pub const MAX_PASSWORD: usize = 1024;
pub const MAX_NOTES: usize = 5000;
/// A vault is a personal list, not a database: this also bounds how much is decrypted at once.
pub const MAX_ENTRIES: usize = 5000;

/// One saved login. It is encrypted as a whole; only `Summary` ever leaves the backend by default.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct EntryData {
    pub title: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub notes: String,
}

/// An entry as the list shows it: no password and no notes.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Summary {
    pub id: String,
    pub title: String,
    pub url: String,
    pub username: String,
    pub updated_at: i64,
}

/// An entry as the editor needs it, after the user asked to see or edit it.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Revealed {
    pub id: String,
    #[serde(flatten)]
    pub data: EntryData,
    pub updated_at: i64,
}

/// What the interface may know without unlocking.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Status {
    pub initialized: bool,
    /// The vault opens with a master password.
    pub has_password: bool,
    /// The vault opens with Windows Hello.
    pub has_hello: bool,
    /// This computer offers Windows Hello (always false outside Windows).
    pub hello_available: bool,
    /// Windows Hello is asked again every time a password is shown or copied.
    pub require_hello: bool,
    pub unlocked: bool,
    pub entry_count: usize,
    pub auto_lock_minutes: u32,
    /// Seconds to wait before another unlock attempt is accepted (0 when none).
    pub retry_after_seconds: u64,
}

pub const AUTO_LOCK_CHOICES: [u32; 5] = [1, 5, 15, 30, 60];
pub const DEFAULT_AUTO_LOCK_MINUTES: u32 = 5;

impl EntryData {
    pub fn validate(&self) -> Result<(), String> {
        let long = |text: &str, max: usize| text.chars().count() > max;
        if self.title.trim().is_empty()
            || long(&self.title, MAX_TITLE)
            || long(&self.url, MAX_URL)
            || long(&self.username, MAX_USERNAME)
            || long(&self.password, MAX_PASSWORD)
            || long(&self.notes, MAX_NOTES)
        {
            return Err("vault.invalid_entry".into());
        }
        Ok(())
    }

    pub fn summary(&self, id: &str, updated_at: i64) -> Summary {
        Summary {
            id: id.into(),
            title: self.title.clone(),
            url: self.url.clone(),
            username: self.username.clone(),
            updated_at,
        }
    }
}

/// Entry ids are random hex made by the backend; anything else is refused before it reaches SQL.
pub fn valid_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> EntryData {
        EntryData {
            title: "Example".into(),
            url: "https://example.test".into(),
            username: "me".into(),
            password: "pw".into(),
            notes: String::new(),
        }
    }

    #[test]
    fn a_title_is_required_and_every_field_is_bounded() {
        assert!(entry().validate().is_ok());
        let mut e = entry();
        e.title = "   ".into();
        assert!(e.validate().is_err());
        for (field, max) in [
            (0, MAX_TITLE),
            (1, MAX_URL),
            (2, MAX_USERNAME),
            (3, MAX_PASSWORD),
            (4, MAX_NOTES),
        ] {
            let mut e = entry();
            let text = "a".repeat(max + 1);
            match field {
                0 => e.title = text,
                1 => e.url = text,
                2 => e.username = text,
                3 => e.password = text,
                _ => e.notes = text,
            }
            assert!(e.validate().is_err(), "field {field}");
        }
    }

    #[test]
    fn unknown_fields_are_rejected_and_the_summary_hides_secrets() {
        assert!(serde_json::from_str::<EntryData>(r#"{"title":"a","extra":1}"#).is_err());
        let json = serde_json::to_string(&entry().summary("id", 1)).unwrap();
        assert!(!json.contains("pw") && !json.contains("password") && !json.contains("notes"));
    }

    #[test]
    fn ids_are_32_hex_characters() {
        assert!(valid_id(&"a1".repeat(16)));
        for bad in [
            "",
            "abc",
            &"g".repeat(32),
            &"a".repeat(33),
            "'; DROP TABLE vault_entries;--",
        ] {
            assert!(!valid_id(bad), "{bad}");
        }
    }
}
