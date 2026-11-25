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

/// The host of an address without `www.`, for showing ("example.com").
pub fn display_host(raw: &str) -> String {
    host_of(raw).unwrap_or_else(|| raw.to_string())
}

/// A page is of the same site as an address (or a subdomain of it, or the other way round).
pub fn same_family(site_url: &str, page: &Url) -> bool {
    let (Some(site), Some(host)) = (host_of(site_url), page.host_str()) else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    same_site(&site, host.strip_prefix("www.").unwrap_or(&host))
}

/// Same host, or one is a subdomain of the other (`login.example.com` and `example.com`) inside one
/// registrable domain. The registrable domain comes from the Public Suffix List, so hosts that merely
/// share a public suffix never match: a login for `github.io` is not offered on `anyone.github.io`,
/// nor the other way round, and the same goes for `co.uk`, `blogspot.com` and similar suffixes.
fn same_site(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let nested = a.ends_with(&format!(".{b}")) || b.ends_with(&format!(".{a}"));
    nested
        && matches!(
            (psl::domain_str(a), psl::domain_str(b)),
            (Some(x), Some(y)) if x.eq_ignore_ascii_case(y)
        )
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
    fn hosts_that_only_share_a_public_suffix_never_match() {
        for (saved, visited) in [
            ("https://github.io", "https://attacker.github.io/"),
            ("https://alice.github.io", "https://github.io/"),
            ("https://alice.github.io", "https://bob.github.io/"),
            ("https://myshop.blogspot.com", "https://blogspot.com/"),
            ("https://co.uk", "https://bank.co.uk/"),
            ("https://bank.co.uk", "https://co.uk/"),
        ] {
            assert!(
                !login_applies(saved, &page(visited)),
                "{saved} on {visited}"
            );
        }
        // Inside one registrable domain, subdomains still match both ways.
        assert!(login_applies(
            "https://bank.co.uk",
            &page("https://login.bank.co.uk/")
        ));
        assert!(login_applies(
            "https://alice.github.io",
            &page("https://docs.alice.github.io/")
        ));
        assert!(login_applies(
            "http://localhost:8080",
            &page("http://localhost:8080/")
        ));
        assert!(login_applies(
            "http://192.168.0.10",
            &page("http://192.168.0.10/")
        ));
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
