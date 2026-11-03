(() => {
  if (location.href !== __NEW_TAB_URL__ || window.top !== window) return;
  window.__veloceNewTabId = __NEW_TAB_ID__;
  const token = __NEW_TAB_TOKEN__, documentId = crypto.randomUUID();
  let sequence = 0, request = 0;
  const send = (action, values = {}) => {
    const params = new URLSearchParams({ token, document: documentId, sequence: String(++sequence), ...values });
    location.href = `veloce-internal://new-tab/${action}?${params}`;
  };
  window.__veloceRefreshNewTab = () => send('read', { request: String(++request) });
  window.__veloceNewTabReply = (id, number, payload) => {
    if (id !== documentId || number !== request || location.href !== __NEW_TAB_URL__) return;
    window.dispatchEvent(new CustomEvent('veloce-new-tab', { detail: payload }));
  };
  window.addEventListener('veloce-new-tab-action', event => {
    const { action, input = '', site_key = '' } = event.detail;
    if (action === 'read') window.__veloceRefreshNewTab();
    else if (action === 'navigate') send('navigate', { input });
    else if (action === 'visit') send('visit', { input, site_key });
  });
  document.addEventListener('DOMContentLoaded', () => window.__veloceRefreshNewTab(), { once: true });
  window.addEventListener('pageshow', event => { if (event.persisted) window.__veloceRefreshNewTab(); });
})();
