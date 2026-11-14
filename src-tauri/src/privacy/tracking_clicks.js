(() => {
  if (window.__sparkTrackingPolicy) return;
  let settings = __SETTINGS__;
  const token = __TOKEN__;
  Object.defineProperty(window, '__sparkTrackingPolicy', { value: (secret, next) => { if (secret === token) settings = next; } });
  document.addEventListener('click', event => {
    if (!settings.enabled || !event.isTrusted || event.defaultPrevented || event.button !== 0) return;
    const anchor = event.target instanceof Element ? event.target.closest('a[href]') : null;
    if (!anchor || anchor.hasAttribute('download')) return;
    let url;
    try { url = new URL(anchor.href, document.baseURI); } catch { return; }
    if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) return;
    const host = url.hostname.toLowerCase().replace(/\.$/, '');
    if (settings.exceptions.some(domain => host === domain || host.endsWith('.' + domain))) return;
    if (['/oauth', '/authorize', '/login', '/signin', '/saml'].some(path => url.pathname.toLowerCase().includes(path))) return;
    const raw = url.search.slice(1).split('&');
    const fields = raw.map(part => {
      try { return new URLSearchParams(part).keys().next().value?.toLowerCase() ?? ''; } catch { return ''; }
    });
    if (fields.some(key => ['state', 'code', 'nonce', 'client_id', 'redirect_uri', 'response_type', 'sig', 'hmac', 'hash', 'key', 'policy', 'expires', 'samlrequest', 'samlresponse'].includes(key) || ['signature', 'token', 'credential', 'auth'].some(s => key.includes(s)) || key.startsWith('x-amz-') || key.startsWith('x-goog-'))) return;
    const tracking = ['utm_source', 'utm_medium', 'utm_campaign', 'utm_term', 'utm_content', 'utm_id', 'fbclid', 'gclid', 'dclid', 'msclkid'];
    const kept = raw.filter((_, index) => !tracking.includes(fields[index]));
    if (kept.length !== raw.length) { url.search = kept.length ? '?' + kept.join('&') : ''; anchor.href = url.href; }
  }, true);
})();
