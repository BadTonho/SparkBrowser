//! Search terms proposed by the default search engine while typing in the address bar.
//!
//! This is the only place where what is typed leaves the computer before Enter, so it is strict:
//! the interface calls it only while the option is on (it is on by default, and the user can turn it
//! off) and never in a private window; it sends nothing but the typed text and a generic client name, follows no redirect,
//! keeps no cookies and gives up after a few seconds. Engines without a public suggestion endpoint
//! get no suggestions. Keep the engine ids in step with `WEB_SUGGESTION_ENGINES` in
//! `src/lib/services/address-suggestions.js`.
use std::io::Read;
use std::time::Duration;
use url::Url;

const MAX_QUERY_CHARS: usize = 100;
const MAX_RESPONSE_BYTES: u64 = 64 * 1024;
const MAX_TERMS: usize = 8;
const MAX_TERM_CHARS: usize = 200;
const TIMEOUT: Duration = Duration::from_secs(3);

/// Endpoint, fixed parameters and the name of the query parameter, per engine id.
fn endpoint(engine: &str) -> Option<(&'static str, &'static [(&'static str, &'static str)], &'static str)> {
    match engine {
        "google" => Some((
            "https://suggestqueries.google.com/complete/search",
            &[("client", "firefox")],
            "q",
        )),
        "duckduckgo" => Some(("https://duckduckgo.com/ac/", &[("type", "list")], "q")),
        "bing" => Some(("https://api.bing.com/osjson.aspx", &[], "query")),
        "brave" => Some(("https://search.brave.com/api/suggest", &[], "q")),
        _ => None,
    }
}

/// The request for `query`, or `None` when the engine has no endpoint or the text is not worth sending.
fn request_url(engine: &str, query: &str) -> Option<Url> {
    let query = query.trim();
    if query.is_empty()
        || query.chars().count() > MAX_QUERY_CHARS
        || query.chars().any(char::is_control)
    {
        return None;
    }
    let (base, fixed, parameter) = endpoint(engine)?;
    let mut url = Url::parse(base).ok()?;
    {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in fixed {
            pairs.append_pair(key, value);
        }
        pairs.append_pair(parameter, query);
    }
    Some(url)
}

/// OpenSearch suggestions are `["query", ["term", ...], ...]`; anything else yields nothing.
fn parse_terms(body: &[u8], typed: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return Vec::new();
    };
    let Some(items) = value.get(1).and_then(|terms| terms.as_array()) else {
        return Vec::new();
    };
    let mut terms: Vec<String> = Vec::new();
    for item in items {
        let Some(text) = item.as_str() else { continue };
        let text = text.trim();
        if text.is_empty()
            || text.chars().count() > MAX_TERM_CHARS
            || text.chars().any(char::is_control)
            || text.eq_ignore_ascii_case(typed.trim())
            || terms.iter().any(|known| known == text)
        {
            continue;
        }
        terms.push(text.to_owned());
        if terms.len() == MAX_TERMS {
            break;
        }
    }
    terms
}

fn fetch(url: Url, typed: &str) -> Vec<String> {
    let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Spark")
        .build()
    else {
        return Vec::new();
    };
    let Ok(response) = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
    else {
        return Vec::new();
    };
    if !response.status().is_success() {
        return Vec::new();
    }
    let mut body = Vec::new();
    if response
        .take(MAX_RESPONSE_BYTES)
        .read_to_end(&mut body)
        .is_err()
    {
        return Vec::new();
    }
    parse_terms(&body, typed)
}

/// Terms the engine `engine` proposes for `query`. A network failure is not an error for the user:
/// the list simply has no engine terms, so this never returns one.
#[tauri::command]
pub async fn fetch_search_suggestions(engine: String, query: String) -> Result<Vec<String>, String> {
    let Some(url) = request_url(&engine, &query) else {
        return Ok(Vec::new());
    };
    Ok(tauri::async_runtime::spawn_blocking(move || fetch(url, &query))
        .await
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::{parse_terms, request_url};

    #[test]
    fn requests_use_each_engines_own_endpoint_and_encode_the_text() {
        let google = request_url("google", "rust & más").unwrap();
        assert_eq!(google.host_str(), Some("suggestqueries.google.com"));
        assert_eq!(google.scheme(), "https");
        let pairs: Vec<_> = google.query_pairs().collect();
        assert!(pairs.iter().any(|(k, v)| k == "client" && v == "firefox"));
        assert!(pairs.iter().any(|(k, v)| k == "q" && v == "rust & más"));
        let bing = request_url("bing", "cats").unwrap();
        assert!(bing.query_pairs().any(|(k, v)| k == "query" && v == "cats"));
        let duck = request_url("duckduckgo", "cats").unwrap();
        assert!(duck.query_pairs().any(|(k, v)| k == "type" && v == "list"));
    }

    #[test]
    fn engines_without_an_endpoint_and_unsuitable_text_send_nothing() {
        assert!(request_url("kagi", "cats").is_none());
        assert!(request_url("unknown", "cats").is_none());
        assert!(request_url("google", "   ").is_none());
        assert!(request_url("google", "line\nbreak").is_none());
        assert!(request_url("google", &"a".repeat(101)).is_none());
        assert!(request_url("google", &"a".repeat(100)).is_some());
    }

    #[test]
    fn only_clean_distinct_terms_are_kept_without_repeating_the_typed_text() {
        let body = br#"["rust",["rust","Rust lang","rust lang","rust book","","bad\u0007term",7,null,"a","b","c","d","e","f","g"]]"#;
        let terms = parse_terms(body, "Rust");
        assert_eq!(terms.first().map(String::as_str), Some("Rust lang"));
        assert!(!terms.iter().any(|term| term.eq_ignore_ascii_case("rust")));
        assert!(!terms.iter().any(|term| term.contains('\u{7}')));
        assert_eq!(terms.len(), 8);
    }

    #[test]
    fn unexpected_responses_yield_no_terms() {
        assert!(parse_terms(b"not json", "x").is_empty());
        assert!(parse_terms(br#"{"suggestions":["a"]}"#, "x").is_empty());
        assert!(parse_terms(br#"["x","oops"]"#, "x").is_empty());
    }
}
