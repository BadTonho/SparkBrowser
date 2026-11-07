//! Isolated policy candidate. No browser integration or security guarantee.
use url::Url;

#[derive(Debug, PartialEq, Eq)]
struct Attempt {
    tab: String,
    generation: String,
    navigation: u64,
    original: Url,
    upgraded: Url,
}

fn upgrade_http(url: &Url, tab: &str, generation: &str, navigation: u64) -> Result<Attempt, &'static str> {
    if url.scheme() != "http" || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err("invalid HTTP navigation");
    }
    let mut upgraded = url.clone();
    upgraded.set_scheme("https").map_err(|_| "invalid HTTPS upgrade")?;
    Ok(Attempt { tab: tab.into(), generation: generation.into(), navigation, original: url.clone(), upgraded })
}

/// A failure can authorize exactly the original request for one matching tab,
/// WebView generation and navigation. Even a mismatched use consumes it.
struct OneUseHttp(Option<(String, String, u64, Url)>);

impl OneUseHttp {
    fn after_failure(attempt: Attempt, failed_navigation: u64) -> Option<Self> {
        (attempt.navigation == failed_navigation).then_some(Self(Some((attempt.tab, attempt.generation, attempt.navigation, attempt.original))))
    }

    fn consume(&mut self, tab: &str, generation: &str, navigation: u64, destination: &Url) -> bool {
        self.0.take().is_some_and(|(expected_tab, expected_generation, expected_navigation, expected_url)| {
            expected_tab == tab && expected_generation == generation && expected_navigation == navigation && expected_url == *destination
        })
    }
}

fn validate_doh_endpoint(raw: &str) -> Result<Url, &'static str> {
    let url = Url::parse(raw).map_err(|_| "invalid endpoint")?;
    if url.scheme() != "https" || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err("invalid endpoint");
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrades_only_http_and_preserves_url_components() {
        let input = Url::parse("http://example.test:8080/path?q=value#anchor").unwrap();
        let attempt = upgrade_http(&input, "tab", "generation", 8).unwrap();
        assert_eq!(attempt.upgraded.as_str(), "https://example.test:8080/path?q=value#anchor");
        for input in ["https://example.test/", "http://user:secret@example.test/", "file:///tmp/test"] {
            assert!(upgrade_http(&Url::parse(input).unwrap(), "tab", "generation", 8).is_err());
        }
    }

    #[test]
    fn one_time_permission_is_exact_and_cannot_survive_a_new_document() {
        let input = Url::parse("http://example.test/path").unwrap();
        let make = || upgrade_http(&input, "tab", "generation", 8).unwrap();
        assert!(OneUseHttp::after_failure(make(), 9).is_none());
        let mut permit = OneUseHttp::after_failure(make(), 8).unwrap();
        assert!(!permit.consume("tab", "other-generation", 8, &input));
        assert!(!permit.consume("tab", "generation", 8, &input));
        let mut permit = OneUseHttp::after_failure(make(), 8).unwrap();
        assert!(!permit.consume("tab", "generation", 8, &Url::parse("http://example.test/other").unwrap()));
        let mut permit = OneUseHttp::after_failure(make(), 8).unwrap();
        assert!(permit.consume("tab", "generation", 8, &input));
        assert!(!permit.consume("tab", "generation", 8, &input));
    }

    #[test]
    fn doh_endpoint_requires_https_without_credentials_or_fragment() {
        assert!(validate_doh_endpoint("https://dns.example/dns-query").is_ok());
        for input in ["http://dns.example/dns-query", "https://user:secret@dns.example/", "https://dns.example/#fragment", "not a url"] {
            assert!(validate_doh_endpoint(input).is_err());
        }
    }
}
