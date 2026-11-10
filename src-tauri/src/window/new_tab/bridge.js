(() => {
  if (location.href !== __NEW_TAB_URL__ || window.top !== window) return;
  window.__veloceNewTabId = __NEW_TAB_ID__;
  const token = __NEW_TAB_TOKEN__, documentId = crypto.randomUUID();
  let sequence = 0, request = 0;
  let widgetsRequest = 0, widgetsVersion = -1;
  const widgetRequests = new Map();
  const send = (action, values = {}) => {
    const params = new URLSearchParams({ token, document: documentId, sequence: String(++sequence), ...values });
    const url = `veloce-internal://new-tab/${action}?${params}`;
    if (window.__velocePostContent) window.__velocePostContent(url);
    else location.href = url;
  };
  window.__veloceRefreshNewTab = () => send('read', { request: String(++request) });
  window.__veloceNewTabReply = (id, number, payload) => {
    if (id !== documentId || number !== request || location.href !== __NEW_TAB_URL__) return;
    window.dispatchEvent(new CustomEvent('veloce-new-tab', { detail: payload }));
    if (document.querySelector?.('#widget-settings')) window.__veloceRefreshWidgets();
  };
  const widgets = (operation, data = {}) => {
    const number = ++widgetsRequest;
    widgetRequests.set(operation, number);
    send('widgets', { operation, data: JSON.stringify(data), request: String(number) });
  };
  window.__veloceRefreshWidgets = () => widgets('read');
  window.__veloceWidgetsReply = (id, number, payload) => {
    if (id !== documentId || location.href !== __NEW_TAB_URL__ || ![...widgetRequests.values()].includes(number)) return;
    if (payload.version < widgetsVersion) return;
    widgetsVersion = payload.version;
    window.dispatchEvent(new CustomEvent('veloce-new-tab-widgets', { detail: payload }));
  };
  window.addEventListener('veloce-new-tab-action', event => {
    const { action, input = '', site_key = '' } = event.detail;
    if (action === 'read') window.__veloceRefreshNewTab();
    else if (action === 'navigate') send('navigate', { input });
    else if (action === 'visit') send('visit', { input, site_key });
    else if (action === 'widgets') widgets(event.detail.operation, event.detail.data);
  });
  document.addEventListener('DOMContentLoaded', () => window.__veloceRefreshNewTab(), { once: true });
  window.addEventListener('pageshow', event => { if (event.persisted) window.__veloceRefreshNewTab(); });
})();
