(() => {
  const panel = document.getElementById('history-panel');
  const $ = id => document.getElementById(id);
  let state = null, sequence = 0, closing = false, inputTimer = 0;
  function send(action, value = {}) {
    if (!state || closing) return;
    if (action === 'close') closing = true;
    const url = new URL('spark-internal://history-overlay/action');
    url.searchParams.set('token', state.token);
    url.searchParams.set('sequence', String(++sequence));
    url.searchParams.set('payload', JSON.stringify({ action, ...value }));
    location.href = url.toString();
  }
  function render(payload) {
    const labels = payload.labels;
    document.documentElement.lang = payload.locale;
    document.title = labels.title;
    panel.style.setProperty('--surface', payload.theme.surface);
    panel.style.setProperty('--surface-border', payload.theme.surfaceBorder ?? '');
    panel.style.setProperty('--surface-filter', payload.theme.surfaceFilter ?? '');
    panel.style.setProperty('--surface-shadow', payload.theme.surfaceShadow ?? '');
    panel.style.setProperty('--border', payload.theme.border);
    panel.style.setProperty('--text', payload.theme.text);
    panel.style.setProperty('--muted', payload.theme.muted);
    panel.style.setProperty('--accent', payload.theme.accent);
    panel.style.setProperty('--hover', payload.theme.hover);
    panel.style.setProperty('--field', payload.theme.field);
    $('title').textContent = labels.title;
    $('clear').textContent = labels.clear;
    $('clear').hidden = payload.entries.length === 0;
    $('clear').disabled = payload.busy;
    $('close').setAttribute('aria-label', labels.close);
    $('query').placeholder = labels.search;
    if (document.activeElement !== $('query')) $('query').value = payload.query;
    $('error').textContent = payload.error || '';
    $('error').hidden = !payload.error;
    const entries = $('entries');
    entries.setAttribute('aria-busy', String(payload.busy));
    entries.replaceChildren();
    if (payload.entries.length === 0) {
      const empty = document.createElement('div'); empty.className = 'empty-state'; empty.textContent = labels.empty; entries.append(empty);
    } else {
      for (const item of payload.entries) {
        const row = document.createElement('button'); row.className = 'history-row'; row.dataset.testid = 'history-row'; row.type = 'button';
        const meta = document.createElement('span'); meta.className = 'history-meta';
        const title = document.createElement('span'); title.className = 'history-title'; title.textContent = item.title || item.url;
        const url = document.createElement('span'); url.className = 'history-url'; url.textContent = item.url;
        meta.append(title, url);
        const time = document.createElement('span'); time.className = 'history-time'; time.textContent = item.time;
        row.append(meta, time); row.addEventListener('click', () => send('open', { entryId: item.id })); entries.append(row);
      }
    }
    let more = document.querySelector('.more'); more?.remove();
    if (payload.hasMore) {
      more = document.createElement('button'); more.className = 'more'; more.type = 'button'; more.textContent = labels.loadMore; more.disabled = payload.busy;
      more.addEventListener('click', () => send('load-more')); panel.append(more);
    }
    panel.hidden = false;
  }
  window.__sparkSetHistoryOverlay = payload => {
    if (!payload.visible) {
      if (!state || state.sessionId <= payload.sessionId) { state = null; panel.hidden = true; closing = false; }
      return;
    }
    if (state && state.sessionId > payload.sessionId) return;
    const opening = state?.sessionId !== payload.sessionId;
    state = payload;
    if (opening) { sequence = 0; closing = false; }
    render(payload);
    if (opening) $('query').focus({ preventScroll: true });
  };
  $('clear').addEventListener('click', () => send('clear'));
  $('close').addEventListener('click', () => send('close'));
  $('query').addEventListener('input', () => {
    clearTimeout(inputTimer);
    const query = $('query').value;
    inputTimer = setTimeout(() => send('search', { query }), 180);
  });
  document.addEventListener('pointerdown', event => {
    if (state && !panel.contains(event.target)) send('close');
  }, true);
  window.addEventListener('keydown', event => {
    if (!state) return;
    if (event.key === 'Escape') { event.preventDefault(); event.stopImmediatePropagation(); send('close'); return; }
    if (event.key === 'Tab') {
      const controls = [...panel.querySelectorAll('button:not([disabled]):not([hidden]),input:not([disabled]):not([hidden])')].filter(node => node.getClientRects().length);
      if (!controls.length) { event.preventDefault(); panel.focus(); return; }
      const first = controls[0], last = controls.at(-1);
      if (event.shiftKey && (document.activeElement === first || !panel.contains(document.activeElement))) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    }
  }, true);
})();
