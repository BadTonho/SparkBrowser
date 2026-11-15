(() => {
  if (location.href !== __NEW_TAB_URL__ || window.top !== window) return;
  window.__sparkNewTabId = __NEW_TAB_ID__;
  const token = __NEW_TAB_TOKEN__, documentId = crypto.randomUUID();
  let sequence = 0, request = 0;
  let widgetsRequest = 0, widgetsVersion = -1;
  const widgetRequests = new Map();
  let audioSubscribed = false, audioGeneration = -1, audioSequence = -1;
  const send = (action, values = {}) => {
    const params = new URLSearchParams({ token, document: documentId, sequence: String(++sequence), ...values });
    const url = `spark-internal://new-tab/${action}?${params}`;
    if (window.__sparkPostContent) window.__sparkPostContent(url);
    else location.href = url;
  };
  window.__sparkRefreshNewTab = () => send('read', { request: String(++request) });
  window.__sparkNewTabReply = (id, number, payload) => {
    if (id !== documentId || number !== request || location.href !== __NEW_TAB_URL__) return;
    window.dispatchEvent(new CustomEvent('spark-new-tab', { detail: payload }));
    if (document.querySelector?.('#widget-settings')) window.__sparkRefreshWidgets();
  };
  const widgets = (operation, data = {}) => {
    if (operation === 'audio_subscribe') audioSubscribed = true;
    if (operation === 'audio_unsubscribe') audioSubscribed = false;
    const number = ++widgetsRequest;
    widgetRequests.set(operation, number);
    send('widgets', { operation, data: JSON.stringify(data), request: String(number) });
  };
  window.__sparkRefreshWidgets = () => widgets('read');
  window.__sparkAudioFrame = (id, payload) => {
    if (id !== documentId || location.href !== __NEW_TAB_URL__ || !audioSubscribed) return;
    if (!Number.isSafeInteger(payload.generation) || !Number.isSafeInteger(payload.sequence)
        || payload.generation < audioGeneration || (payload.generation === audioGeneration && payload.sequence <= audioSequence)) return;
    if (!Array.isArray(payload.bands) || payload.bands.length !== 32 || payload.bands.some(v => !Number.isFinite(v) || v < 0 || v > 1)) return;
    audioGeneration = payload.generation; audioSequence = payload.sequence;
    window.dispatchEvent(new CustomEvent('spark-audio-frame', { detail: payload }));
  };
  window.__sparkWidgetsReply = (id, number, payload) => {
    if (id !== documentId || location.href !== __NEW_TAB_URL__ || ![...widgetRequests.values()].includes(number)) return;
    if (payload.version < widgetsVersion) return;
    widgetsVersion = payload.version;
    window.dispatchEvent(new CustomEvent('spark-new-tab-widgets', { detail: payload }));
  };
  window.addEventListener('spark-new-tab-action', event => {
    const { action, input = '', site_key = '' } = event.detail;
    if (action === 'read') window.__sparkRefreshNewTab();
    else if (action === 'navigate') send('navigate', { input });
    else if (action === 'visit') send('visit', { input, site_key });
    else if (action === 'widgets') widgets(event.detail.operation, event.detail.data);
  });
  document.addEventListener('DOMContentLoaded', () => window.__sparkRefreshNewTab(), { once: true });
  window.addEventListener('pageshow', event => { if (event.persisted) window.__sparkRefreshNewTab(); });
})();
