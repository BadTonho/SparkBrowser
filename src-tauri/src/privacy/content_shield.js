(() => {
  const enabled = __SHIELD_ENABLED__;
  if (!enabled) return;
  const domains = __BLOCKED_DOMAINS__;
  const NativeURL = URL;
  const blocked = (input) => {
    try {
      const raw = input && typeof input === "object" && "url" in input ? input.url : input;
      const host = new NativeURL(String(raw), document.baseURI).hostname.toLowerCase().replace(/\.$/, "");
      return domains.some((domain) => host === domain || host.endsWith(`.${domain}`));
    } catch { return false; }
  };

  const originalFetch = window.fetch;
  if (originalFetch) {
    window.fetch = function(input) {
      if (blocked(input)) return Promise.reject(new TypeError("Blocked by Veloce Shield"));
      return originalFetch.apply(this, arguments);
    };
  }

  const blockedRequests = new WeakSet();
  const originalOpen = XMLHttpRequest.prototype.open;
  const originalSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.open = function(method, url) {
    const result = originalOpen.apply(this, arguments);
    if (blocked(url)) blockedRequests.add(this);
    else blockedRequests.delete(this);
    return result;
  };
  XMLHttpRequest.prototype.send = function() {
    if (blockedRequests.has(this)) {
      this.abort();
      throw new DOMException("Blocked by Veloce Shield", "NetworkError");
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
    if (document.getElementById("veloce-adblock-style")) return;
    const style = document.createElement("style");
    style.id = "veloce-adblock-style";
    style.textContent = 'ins.adsbygoogle,[id^="google_ads_"],[id^="div-gpt-ad"],.ad-banner,.ad-container,.sponsored-content { display: none !important; }';
    (document.head || document.documentElement).appendChild(style);
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", injectStyle, { once: true });
  else injectStyle();
})();
