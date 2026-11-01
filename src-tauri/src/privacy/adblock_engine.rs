use serde::{Deserialize, Serialize};
use tauri::Url;

/// Built-in high-frequency ad and tracker host suffixes blocked at the Rust network/navigation layer.
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
    pub blocked_count: u64,
    pub hibernated_tabs_count: usize,
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

/// Returns the initialization script injected into every child WebView to block
/// sub-resource tracker requests (fetch/XHR/beacon) and apply cosmetic ad-hiding CSS.
pub fn build_content_shield_script(shield_enabled: bool) -> String {
    if !shield_enabled {
        return String::from("/* Veloce Shield Disabled */");
    }

    r#"
    (function() {
      const BLOCKED_HOSTS = [
        "doubleclick.net",
        "googlesyndication.com",
        "googleadservices.com",
        "google-analytics.com",
        "googletagmanager.com",
        "adnxs.com",
        "adsrvr.org",
        "criteo.com",
        "outbrain.com",
        "taboola.com",
        "scorecardresearch.com",
        "moatads.com",
        "popads.net"
      ];

      function isBlockedTarget(urlStr) {
        if (!urlStr || typeof urlStr !== "string") return false;
        const lower = urlStr.toLowerCase();
        return BLOCKED_HOSTS.some((d) => lower.includes(d));
      }

      // 1. Intercept fetch()
      const originalFetch = window.fetch;
      if (originalFetch) {
        window.fetch = function(input, init) {
          const url = typeof input === "string" ? input : (input && input.url) || "";
          if (isBlockedTarget(url)) {
            return Promise.resolve(new Response(null, { status: 204, statusText: "Blocked by Veloce Shield" }));
          }
          return originalFetch.apply(this, arguments);
        };
      }

      // 2. Intercept XMLHttpRequest
      const originalOpen = XMLHttpRequest.prototype.open;
      XMLHttpRequest.prototype.open = function(method, url) {
        if (isBlockedTarget(String(url))) {
          this._veloceBlocked = true;
        }
        return originalOpen.apply(this, arguments);
      };

      const originalSend = XMLHttpRequest.prototype.send;
      XMLHttpRequest.prototype.send = function() {
        if (this._veloceBlocked) {
          return;
        }
        return originalSend.apply(this, arguments);
      };

      // 3. Cosmetic CSS Filter
      const injectCosmeticShield = () => {
        if (document.getElementById("veloce-adblock-style")) return;
        const style = document.createElement("style");
        style.id = "veloce-adblock-style";
        style.textContent = `
          ins.adsbygoogle,
          [id^="google_ads_"],
          [id^="div-gpt-ad"],
          iframe[src*="doubleclick.net"],
          iframe[src*="googlesyndication.com"],
          .ad-banner,
          .ad-container,
          .sponsored-content {
            display: none !important;
            height: 0 !important;
            pointer-events: none !important;
          }
        `;
        (document.head || document.documentElement).appendChild(style);
      };

      if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", injectCosmeticShield, { once: true });
      } else {
        injectCosmeticShield();
      }
    })();
    "#
    .to_string()
}

