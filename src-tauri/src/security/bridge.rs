use std::collections::HashMap;
use tauri::Url;

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
    fn duplicated_unknown_or_oversized_parameters_are_rejected() {
        for url in [
            "veloce-internal://test/action?token=a&token=b",
            "veloce-internal://test/action?%74oken=a&token=b",
            "veloce-internal://test/action?command=delete",
            "veloce-internal://user@test/action?token=a",
            "veloce-internal://test/action?token=a#fragment",
        ] {
            assert!(query(&Url::parse(url).unwrap(), &["token"]).is_none());
        }
        assert!(query(
            &Url::parse(&format!(
                "veloce-internal://test/action?token={}",
                "a".repeat(2_000_000)
            ))
            .unwrap(),
            &["token"]
        )
        .is_none());
        assert_eq!(
            query(
                &Url::parse("veloce-internal://test/action?token=a%20b").unwrap(),
                &["token"]
            )
            .unwrap()["token"],
            "a b"
        );
        assert_eq!(boolean(Some(&"false".into())), Some(false));
        assert_eq!(boolean(Some(&"invalid".into())), None);
    }
}
