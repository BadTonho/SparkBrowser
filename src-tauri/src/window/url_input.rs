use crate::state::browser_state::DEFAULT_HOME_URL;
use tauri::Url;

/// Parses user omnibox input using one of the supported search engines.
pub fn resolve_input_to_url_with_search_engine(
    raw_input: &str,
    search_engine: &str,
) -> Result<Url, String> {
    let trimmed = raw_input.trim();
    if super::new_tab::is_new_tab(trimmed) {
        return Url::parse(super::new_tab::URL).map_err(|e| e.to_string());
    }
    if trimmed.is_empty() {
        return Url::parse(DEFAULT_HOME_URL).map_err(|e| e.to_string());
    }

    if let Ok(parsed) = Url::parse(trimmed) {
        if matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_some() {
            return Ok(parsed);
        }
    }

    let host = trimmed.split(['/', '?', '#']).next().unwrap_or("");
    let local = host.eq_ignore_ascii_case("localhost")
        || host.to_ascii_lowercase().starts_with("localhost:")
        || host.starts_with('[');
    if !trimmed.contains(char::is_whitespace) && (host.contains('.') || local) {
        let with_https = format!("{}://{}", if local { "http" } else { "https" }, trimmed);
        if let Ok(parsed) = Url::parse(&with_https) {
            return Ok(parsed);
        }
    }

    let search_url_base = match search_engine {
        "duckduckgo" => "https://duckduckgo.com/",
        _ => "https://www.google.com/search",
    };
    let mut search_url = Url::parse(search_url_base).map_err(|e| e.to_string())?;
    search_url.query_pairs_mut().append_pair("q", trimmed);
    Ok(search_url)
}

pub fn search_engine_home_url(search_engine: Option<&str>) -> &'static str {
    match search_engine {
        Some("duckduckgo") => "https://duckduckgo.com/",
        _ => "https://www.google.com/",
    }
}

/// Resolve explicit shell input through stored site searches before the default parser.
pub fn resolve_browser_input(
    app: &tauri::AppHandle,
    input: &str,
    engine: &str,
) -> Result<Url, String> {
    use tauri::Manager;
    if input.trim().contains(char::is_whitespace) {
        let state = app.state::<crate::state::browser_state::BrowserState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(conn) = db.as_ref() {
            if let Some(url) = crate::database::site_search_repo::resolve(conn, input)? {
                let settings = state.tracking_settings.lock().map_err(|e| e.to_string())?;
                return Ok(crate::privacy::tracking_parameters::clean(&url, &settings));
            }
        }
    }
    let url = resolve_input_to_url_with_search_engine(input, engine)?;
    let state = app.state::<crate::state::browser_state::BrowserState>();
    let settings = state.tracking_settings.lock().map_err(|e| e.to_string())?;
    Ok(crate::privacy::tracking_parameters::clean(&url, &settings))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search_uses_the_requested_engine_and_encodes_the_query() {
        let url = resolve_input_to_url_with_search_engine("rust & svelte", "duckduckgo").unwrap();
        assert_eq!(url.host_str(), Some("duckduckgo.com"));
        assert_eq!(
            url.query_pairs().find(|(key, _)| key == "q").unwrap().1,
            "rust & svelte"
        );
        assert_eq!(
            resolve_input_to_url_with_search_engine("rust & svelte", "google")
                .unwrap()
                .host_str(),
            Some("www.google.com")
        );
    }
    #[test]
    fn explicit_web_addresses_are_preserved() {
        let url = "http://127.0.0.1:8080/a?b=c";
        assert_eq!(
            resolve_input_to_url_with_search_engine(url, "duckduckgo")
                .unwrap()
                .as_str(),
            url
        );
        assert_eq!(
            resolve_input_to_url_with_search_engine("example.test/path", "google")
                .unwrap()
                .as_str(),
            "https://example.test/path"
        );
        for (input, expected) in [
            ("localhost:3000/path", "http://localhost:3000/path"),
            ("localhost", "http://localhost/"),
            ("[::1]:8080/", "http://[::1]:8080/"),
            ("HTTP://example.test/", "http://example.test/"),
        ] {
            assert_eq!(
                resolve_input_to_url_with_search_engine(input, "google")
                    .unwrap()
                    .as_str(),
                expected
            );
        }
    }
}
