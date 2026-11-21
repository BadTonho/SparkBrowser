//! Which saved logins belong to which page.
use tauri::Url;

fn host_of(raw: &str) -> Option<String> {
    let text = raw.trim();
    let parsed = Url::parse(text)
        .or_else(|_| Url::parse(&format!("https://{text}")))
        .ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    Some(host.strip_prefix("www.").unwrap_or(&host).to_string())
}

/// The scheme and host of a page, to notice that the person moved to another site.
pub fn origin(raw: &str) -> Option<(String, String)> {
    let url = Url::parse(raw).ok()?;
    Some((
        url.scheme().to_string(),
        url.host_str()?.to_ascii_lowercase(),
    ))
}

/// Same site, or one is a subdomain of the other (`login.example.com` and `example.com`).
fn same_site(a: &str, b: &str) -> bool {
    a == b || a.ends_with(&format!(".{b}")) || b.ends_with(&format!(".{a}"))
}

/// A login is offered on a page of its own site. A login saved for an HTTPS site is never offered on
/// the HTTP version of it, where anyone on the network could read what gets filled in.
pub fn login_applies(saved_url: &str, page: &Url) -> bool {
    let Some(saved_host) = host_of(saved_url) else {
        return false;
    };
    let Some(page_host) = page.host_str().map(|h| {
        let h = h.to_ascii_lowercase();
        h.strip_prefix("www.").unwrap_or(&h).to_string()
    }) else {
        return false;
    };
    if !same_site(&saved_host, &page_host) {
        return false;
    }
    let saved_https = !saved_url.trim().to_ascii_lowercase().starts_with("http://");
    page.scheme() == "https" || !saved_https || page_host == "localhost"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(url: &str) -> Url {
        Url::parse(url).unwrap()
    }

    #[test]
    fn a_login_belongs_to_its_site_and_its_subdomains() {
        assert!(login_applies(
            "https://example.com",
            &page("https://example.com/login")
        ));
        assert!(login_applies(
            "example.com",
            &page("https://www.example.com/")
        ));
        assert!(login_applies(
            "https://example.com",
            &page("https://accounts.example.com/")
        ));
        assert!(login_applies(
            "https://accounts.example.com",
            &page("https://example.com/")
        ));
    }

    #[test]
    fn look_alike_and_other_sites_are_refused() {
        assert!(!login_applies(
            "https://example.com",
            &page("https://example.com.evil.test/")
        ));
        assert!(!login_applies(
            "https://example.com",
            &page("https://badexample.com/")
        ));
        assert!(!login_applies(
            "https://example.com",
            &page("https://other.org/")
        ));
        assert!(!login_applies("", &page("https://example.com/")));
    }

    #[test]
    fn an_https_login_is_not_offered_over_plain_http() {
        assert!(!login_applies(
            "https://example.com",
            &page("http://example.com/")
        ));
        assert!(!login_applies("example.com", &page("http://example.com/")));
        assert!(login_applies(
            "http://example.com",
            &page("http://example.com/")
        ));
        assert!(login_applies(
            "https://localhost:3000",
            &page("http://localhost:3000/")
        ));
    }

    #[test]
    fn the_origin_changes_with_the_host_or_the_scheme() {
        assert_eq!(origin("https://a.test/x"), origin("https://a.test/y"));
        assert_ne!(origin("https://a.test/"), origin("https://b.test/"));
        assert_ne!(origin("https://a.test/"), origin("http://a.test/"));
    }
}
