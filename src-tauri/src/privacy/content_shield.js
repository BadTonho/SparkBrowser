(() => {
  // A replacement native policy script can run before Tauri's immutable initializer.
  if (typeof window.__sparkSetShieldPolicy === "function") return;
  let enabled = __SHIELD_ENABLED__;
  const token = __SHIELD_TOKEN__;
  const domains = __BLOCKED_DOMAINS__;
  const exceptions = __SHIELD_EXCEPTIONS__;
  const siteExcluded = () => {
    const host = location.hostname.toLowerCase().replace(/\.$/, '');
    return exceptions.some(domain => host === domain || host.endsWith('.' + domain));
  };
  let refreshYoutube;
  /*__YOUTUBE_ADS__*/
  const NativeURL = URL;
  const blocked = (input) => {
    if (!enabled || siteExcluded()) return false;
    try {
      const raw = input && typeof input === "object" && "url" in input ? input.url : input;
      const host = new NativeURL(String(raw), document.baseURI).hostname.toLowerCase().replace(/\.$/, "");
      return domains.some((domain) => host === domain || host.endsWith(`.${domain}`));
    } catch { return false; }
  };

  const originalFetch = window.fetch;
  if (originalFetch) {
    window.fetch = function(input) {
      if (blocked(input)) return Promise.reject(new TypeError("Blocked by Spark Shield"));
      return originalFetch.apply(this, arguments);
    };
  }

  const requestURLs = new WeakMap();
  const originalOpen = XMLHttpRequest.prototype.open;
  const originalSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.open = function(method, url) {
    const result = originalOpen.apply(this, arguments);
    requestURLs.set(this, url);
    return result;
  };
  XMLHttpRequest.prototype.send = function() {
    if (blocked(requestURLs.get(this))) {
      this.abort();
      throw new DOMException("Blocked by Spark Shield", "NetworkError");
    }
    return originalSend.apply(this, arguments);
  };

  const originalBeacon = navigator.sendBeacon;
  if (originalBeacon) {
    navigator.sendBeacon = function(url) {
      if (blocked(url)) return false;
      return originalBeacon.apply(this, arguments);
    };
  }

  const injectStyle = () => {
    if (!enabled || siteExcluded()) { document.getElementById("spark-adblock-style")?.remove(); return; }
    if (document.getElementById("spark-adblock-style")) return;
    const style = document.createElement("style");
    style.id = "spark-adblock-style";
    style.textContent = 'ins.adsbygoogle,[id^="google_ads_"],[id^="div-gpt-ad"],.ad-banner,.ad-container,.sponsored-content { display: none !important; }';
    (document.head || document.documentElement).appendChild(style);
  };
  Object.defineProperty(window, "__sparkSetShieldPolicy", { configurable: false, writable: false,
    value: (secret, value) => {
      if (secret !== token) return;
      enabled = Boolean(value);
      refreshYoutube?.();
      if (document.readyState !== "loading") injectStyle();
    }
  });
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", injectStyle, { once: true });
  else injectStyle();
})();
