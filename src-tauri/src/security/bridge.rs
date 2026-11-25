use std::collections::HashMap;
use std::sync::OnceLock;
use tauri::Url;

/// The page script of the autofill channel.
pub const AUTOFILL_CHANNEL: &str = "autofill";
/// The page script of keyboard shortcuts, links opened in a new tab and pane focus.
pub const KEYBOARD_CHANNEL: &str = "keyboard";

/// A secret of this process only. It never reaches a page, so knowing a tab's bridge token is not
/// enough to compute the token of another channel of that tab.
fn process_key() -> &'static [u8; 32] {
    static KEY: OnceLock<[u8; 32]> = OnceLock::new();
    KEY.get_or_init(|| {
        let mut key = [0u8; 32];
        getrandom::fill(&mut key).expect("OS cryptographic randomness unavailable");
        key
    })
}

fn derive(base: &str, purpose: &str) -> String {
    use hmac::{Hmac, Mac};
    let mut mac = <Hmac<sha2::Sha256> as Mac>::new_from_slice(process_key())
        .expect("HMAC accepts any key length");
    mac.update(purpose.as_bytes());
    mac.update(b":");
    mac.update(base.as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The token one page script uses, derived from the tab's bridge token. Page scripts share the
/// page's JavaScript world, so a page that manages to read one script's token must not gain the
/// channels of the others: the high-value channels (autofill, keyboard) get their own token.
pub fn channel_token(base: &str, channel: &str) -> String {
    derive(base, &format!("channel:{channel}"))
}

/// The name of the native message binding of a tab (Windows). It is random per tab so a page cannot
/// plant a fake function under it before the real one is installed.
pub fn content_binding_name(base: &str) -> String {
    format!("__sparkCS{}", &derive(base, "content-binding")[..24])
}

/// Decode once, bound the input and reject ambiguous fields before touching state.
pub fn query(url: &Url, allowed: &[&str]) -> Option<HashMap<String, String>> {
    if url.as_str().len() > 2_000_000
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return None;
    }
    let mut result = HashMap::new();
    for (key, value) in url.query_pairs() {
        let limit = match key.as_ref() {
            "sequence" | "request" | "value" => 24,
            "reason" => 16,
            "action" | "operation" => 64,
            "key" => 32,
            "token" | "session" | "document" | "site_key" | "workspace_id" => 128,
            "input" | "data" => 8192,
            "payload" => 1_500_000,
            _ => 128,
        };
        if !allowed.contains(&key.as_ref())
            || value.len() > limit
            || result
                .insert(key.into_owned(), value.into_owned())
                .is_some()
        {
            return None;
        }
    }
    Some(result)
}

pub fn boolean(value: Option<&String>) -> Option<bool> {
    match value.map(String::as_str) {
        Some("true") => Some(true),
        Some("false") => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn channel_tokens_differ_per_channel_and_tab_and_do_not_reveal_the_base() {
        let base = "a".repeat(64);
        let autofill = channel_token(&base, AUTOFILL_CHANNEL);
        let keyboard = channel_token(&base, KEYBOARD_CHANNEL);
        assert_eq!(autofill.len(), 64);
        assert_ne!(autofill, keyboard);
        assert_ne!(autofill, base);
        assert_eq!(autofill, channel_token(&base, AUTOFILL_CHANNEL));
        assert_ne!(autofill, channel_token(&"b".repeat(64), AUTOFILL_CHANNEL));
        let name = content_binding_name(&base);
        assert!(name.starts_with("__sparkCS") && name.len() == 9 + 24);
        assert!(!autofill.contains(&name[9..]) && !keyboard.contains(&name[9..]));
    }

    #[test]
    fn duplicated_unknown_or_oversized_parameters_are_rejected() {
        for url in [
            "spark-internal://test/action?token=a&token=b",
            "spark-internal://test/action?%74oken=a&token=b",
            "spark-internal://test/action?command=delete",
            "spark-internal://user@test/action?token=a",
            "spark-internal://test/action?token=a#fragment",
        ] {
            assert!(query(&Url::parse(url).unwrap(), &["token"]).is_none());
        }
        assert!(query(
            &Url::parse(&format!(
                "spark-internal://test/action?token={}",
                "a".repeat(2_000_000)
            ))
            .unwrap(),
            &["token"]
        )
        .is_none());
        assert_eq!(
            query(
                &Url::parse("spark-internal://test/action?token=a%20b").unwrap(),
                &["token"]
            )
            .unwrap()["token"],
            "a b"
        );
        assert_eq!(boolean(Some(&"false".into())), Some(false));
        assert_eq!(boolean(Some(&"invalid".into())), None);
    }
}
