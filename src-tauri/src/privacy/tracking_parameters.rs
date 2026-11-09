//! Conservative cleaning for explicitly entered top-level destinations.
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TrackingSettings {
    pub enabled: bool,
    pub exceptions: Vec<String>,
}

impl TrackingSettings {
    pub fn applies_to(&self, url: &Url) -> bool {
        self.enabled
            && url.host_str().is_some_and(|host| {
                let host = host.trim_end_matches('.');
                !self.exceptions.iter().any(|domain| {
                    host == domain
                        || host
                            .strip_suffix(domain)
                            .is_some_and(|prefix| prefix.ends_with('.'))
                })
            })
    }
    pub fn validate(mut self) -> Result<Self, String> {
        if self.exceptions.len() > 256 {
            return Err("Too many exceptions".into());
        }
        let mut result = Vec::new();
        for raw in self.exceptions {
            let input = raw.trim().trim_end_matches('.');
            if input.is_empty() || input.contains(['/', ':', '@', '?', '#', '\\']) {
                return Err("Invalid exception domain".into());
            }
            let url =
                Url::parse(&format!("https://{input}/")).map_err(|_| "Invalid exception domain")?;
            let host = url
                .host_str()
                .ok_or("Invalid exception domain")?
                .to_ascii_lowercase();
            if !result.contains(&host) {
                result.push(host);
            }
        }
        self.exceptions = result;
        Ok(self)
    }
}

pub fn clean(url: &Url, settings: &TrackingSettings) -> Url {
    if !settings.enabled
        || !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return url.clone();
    }
    let Some(host) = url.host_str().map(|value| value.trim_end_matches('.')) else {
        return url.clone();
    };
    if settings.exceptions.iter().any(|domain| {
        host == domain
            || host
                .strip_suffix(domain)
                .is_some_and(|prefix| prefix.ends_with('.'))
    }) {
        return url.clone();
    }
    let path = url.path().to_ascii_lowercase();
    if ["/oauth", "/authorize", "/login", "/signin", "/saml"]
        .iter()
        .any(|segment| path.contains(segment))
    {
        return url.clone();
    }
    let Some(query) = url.query() else {
        return url.clone();
    };
    let fields: Vec<(&str, String)> = query
        .split('&')
        .map(|part| {
            let name = part.split('=').next().unwrap_or("");
            let decoded = url::form_urlencoded::parse(name.as_bytes())
                .next()
                .map(|(key, _)| key.to_ascii_lowercase())
                .unwrap_or_default();
            (part, decoded)
        })
        .collect();
    // Never alter URLs that may be signed or participate in OAuth / SSO.
    if fields.iter().any(|(_, key)| {
        matches!(
            key.as_str(),
            "state"
                | "code"
                | "nonce"
                | "client_id"
                | "redirect_uri"
                | "response_type"
                | "sig"
                | "hmac"
                | "hash"
                | "key"
                | "policy"
                | "expires"
                | "samlrequest"
                | "samlresponse"
        ) || key.contains("signature")
            || key.contains("token")
            || key.contains("credential")
            || key.contains("auth")
            || key.starts_with("x-amz-")
            || key.starts_with("x-goog-")
    }) {
        return url.clone();
    }
    let kept: Vec<&str> = fields
        .iter()
        .filter(|(_, key)| {
            !matches!(
                key.as_str(),
                "utm_source"
                    | "utm_medium"
                    | "utm_campaign"
                    | "utm_term"
                    | "utm_content"
                    | "utm_id"
                    | "fbclid"
                    | "gclid"
                    | "dclid"
                    | "msclkid"
            )
        })
        .map(|(raw, _)| *raw)
        .collect();
    if kept.len() == fields.len() {
        return url.clone();
    }
    let mut result = url.clone();
    if kept.is_empty() {
        result.set_query(None);
    } else {
        result.set_query(Some(&kept.join("&")));
    }
    result
}

pub fn click_script(settings: &TrackingSettings, token: &str) -> String {
    include_str!("tracking_clicks.js")
        .replace(
            "__SETTINGS__",
            &serde_json::to_string(settings).expect("settings serializable"),
        )
        .replace(
            "__TOKEN__",
            &serde_json::to_string(token).expect("token serializable"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn removes_only_selected_parameters_without_reencoding_others() {
        let settings = TrackingSettings {
            enabled: true,
            exceptions: vec![],
        };
        let url = Url::parse("https://example.test/p?a=%2F+%20&utm_source=ad&x=1&fbclid=123#part")
            .unwrap();
        assert_eq!(
            clean(&url, &settings).as_str(),
            "https://example.test/p?a=%2F+%20&x=1#part"
        );
        assert_eq!(clean(&url, &TrackingSettings::default()), url);
    }
    #[test]
    fn preserves_sensitive_urls_and_exceptions() {
        let settings = TrackingSettings {
            enabled: true,
            exceptions: vec!["example.test".into()],
        }
        .validate()
        .unwrap();
        for raw in [
            "https://sub.example.test/?utm_source=a",
            "https://other.test/?utm_source=a&state=abc",
            "https://other.test/?utm_source=a&X-Amz-Signature=abc",
            "https://other.test/?utm_source=a&sig=abc",
            "https://other.test/login?utm_source=a",
            "https://other.test/?utm_source=a&auth_code=abc",
        ] {
            let url = Url::parse(raw).unwrap();
            assert_eq!(clean(&url, &settings), url);
        }
        assert!(TrackingSettings {
            enabled: true,
            exceptions: vec!["example.test/path".into()]
        }
        .validate()
        .is_err());
    }

    #[test]
    fn exception_domains_are_normalized_and_are_label_bounded() {
        let settings = TrackingSettings {
            enabled: true,
            exceptions: vec![" EXAMPLE.Test. ".into(), "example.test".into()],
        }
        .validate()
        .unwrap();
        assert_eq!(settings.exceptions, ["example.test"]);
        let excluded = Url::parse("https://shop.example.test./?utm_source=x").unwrap();
        assert_eq!(clean(&excluded, &settings), excluded);
        let unrelated = Url::parse("https://notexample.test/?utm_source=x").unwrap();
        assert_eq!(
            clean(&unrelated, &settings).as_str(),
            "https://notexample.test/"
        );
    }
}
