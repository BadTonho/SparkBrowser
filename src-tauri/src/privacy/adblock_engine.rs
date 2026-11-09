use serde::{Deserialize, Serialize};
use tauri::Url;

/// Built-in high-frequency ad and tracker host suffixes checked by Rust navigation filtering and the page-level script.
const BLOCKED_DOMAINS: &[&str] = &[
    "doubleclick.net",
    "googlesyndication.com",
    "googleadservices.com",
    "google-analytics.com",
    "googletagmanager.com",
    "adnxs.com",
    "adsrvr.org",
    "criteo.com",
    "criteo.net",
    "outbrain.com",
    "taboola.com",
    "scorecardresearch.com",
    "quantserve.com",
    "rubiconproject.com",
    "pubmatic.com",
    "openx.net",
    "moatads.com",
    "advertising.com",
    "adform.net",
    "popads.net",
    "popcash.net",
    "propellerads.com",
    "zedo.com",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyStats {
    pub shield_enabled: bool,
    pub auto_skip_ads_enabled: bool,
    pub blocked_count: u64,
    pub hibernated_tabs_count: usize,
    pub supports_live_policy_updates: bool,
}

/// Checks if a URL belongs to a known ad or tracking network.
pub fn should_block_url(raw_url: &str, shield_enabled: bool) -> bool {
    if !shield_enabled {
        return false;
    }

    if let Ok(parsed) = Url::parse(raw_url) {
        if let Some(host) = parsed.host_str() {
            let lower_host = host.to_lowercase();
            for domain in BLOCKED_DOMAINS {
                if lower_host == *domain || lower_host.ends_with(&format!(".{}", domain)) {
                    return true;
                }
            }
        }
    }

    false
}

/// Best-effort page-level request filtering and cosmetic rules. The host list
/// is shared with Rust navigation checks; this is not a full network filter.
pub fn build_content_shield_script(shield_enabled: bool, token: &str) -> String {
    build_content_shield_script_with_exceptions(shield_enabled, token, &[])
}

pub fn build_content_shield_script_with_exceptions(
    shield_enabled: bool,
    token: &str,
    exceptions: &[String],
) -> String {
    include_str!("content_shield.js")
        .replace(
            "__SHIELD_EXCEPTIONS__",
            &serde_json::to_string(exceptions).expect("domains serializable"),
        )
        .replace(
            "__SHIELD_TOKEN__",
            &serde_json::to_string(token).expect("token serializable"),
        )
        .replace(
            "__BLOCKED_DOMAINS__",
            &serde_json::to_string(BLOCKED_DOMAINS).expect("static domains are serializable"),
        )
        .replace(
            "__SHIELD_ENABLED__",
            if shield_enabled { "true" } else { "false" },
        )
}

pub fn site_is_protected(state: &crate::state::browser_state::BrowserState, raw: &str) -> bool {
    let Ok(url) = Url::parse(raw) else {
        return true;
    };
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return true;
    }
    state
        .shield_settings
        .lock()
        .map(|settings| settings.applies_to(&url))
        .unwrap_or(true)
}

/// Apply a site exception when the originating tab still exists. When it does
/// not, retain the normal shield policy and only consider the destination.
pub fn download_shield_enabled(
    state: &crate::state::browser_state::BrowserState,
    tab_id: Option<&str>,
    destination: &str,
) -> bool {
    let enabled = state
        .shield_enabled
        .lock()
        .map(|value| *value)
        .unwrap_or(true);
    if !enabled || !site_is_protected(state, destination) {
        return false;
    }
    let source = tab_id.and_then(|id| {
        state.tabs.lock().ok().and_then(|tabs| {
            tabs.iter()
                .find(|tab| tab.id == id)
                .map(|tab| tab.url.clone())
        })
    });
    source
        .as_deref()
        .is_none_or(|url| site_is_protected(state, url))
}

/// Both possible document-created ordering sequences converge to the new policy.
pub fn build_replacement_shield_script(enabled: bool, token: &str) -> String {
    build_replacement_shield_script_with_exceptions(enabled, token, &[])
}

pub fn build_replacement_shield_script_with_exceptions(
    enabled: bool,
    token: &str,
    exceptions: &[String],
) -> String {
    let encoded = serde_json::to_string(token).expect("token serializable");
    let initialize = build_content_shield_script_with_exceptions(enabled, token, exceptions);
    format!("if (typeof window.__veloceSetShieldPolicy === 'function') {{ window.__veloceSetShieldPolicy({encoded}, {enabled}); }} else {{ {initialize} }}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_only_exact_hosts_or_subdomains() {
        assert!(should_block_url(
            "https://sub.google-analytics.com/collect",
            true
        ));
        for url in [
            "https://example.test/google-analytics.com",
            "https://notgoogle-analytics.com/",
            "https://google-analytics.com.example.test/",
        ] {
            assert!(!should_block_url(url, true));
        }
        assert!(!should_block_url("https://google-analytics.com/", false));
    }
    #[test]
    fn download_exceptions_require_an_actual_excluded_site() {
        let state = crate::state::browser_state::BrowserState::default();
        state.shield_settings.lock().unwrap().exceptions = vec!["example.test".into()];
        assert!(!download_shield_enabled(
            &state,
            None,
            "https://example.test/file"
        ));
        assert!(download_shield_enabled(
            &state,
            None,
            "https://doubleclick.net/file"
        ));
        assert!(site_is_protected(&state, "about:blank"));
        assert!(!site_is_protected(&state, "https://sub.example.test/"));
        state.tabs.lock().unwrap()[0].url = "https://sub.example.test/".into();
        assert!(!download_shield_enabled(
            &state,
            Some("tab-1"),
            "https://doubleclick.net/file"
        ));
        assert!(download_shield_enabled(
            &state,
            Some("missing-tab"),
            "https://doubleclick.net/file"
        ));
    }
}
