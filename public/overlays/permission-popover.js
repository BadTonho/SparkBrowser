(() => {
  const panel = document.getElementById('permission-popover');
  const $ = id => document.getElementById(id);
  let state = null, sequence = 0, closing = false, pending = false;
  function position() {
    if (!state) return;
    const a = state.anchor, width = panel.offsetWidth, height = panel.offsetHeight;
    panel.style.left = `${Math.max(8, Math.min(a.left, innerWidth - width - 8))}px`;
    panel.style.top = `${Math.max(8, Math.min(a.top + a.height + 8, innerHeight - height - 8))}px`;
  }
  function send(action, value = {}, reason = '') {
    if (!state || closing || (pending && action !== 'dismiss')) return;
    const url = new URL('veloce-internal://permission-popover/action');
    url.searchParams.set('token', state.token);
    url.searchParams.set('sequence', String(++sequence));
    url.searchParams.set('payload', JSON.stringify({ action, ...value, reason }));
    if (action === 'dismiss' || action === 'respond') closing = action === 'dismiss';
    else { pending = true; panel.querySelectorAll('button,select,input').forEach(control => control.disabled = true); }
    location.href = url.toString();
  }
  function setText(id, value) { $(id).textContent = value || ''; }
  function renderSettings(payload) {
    const labels = payload.labels, settings = payload.settings || {};
    setText('note', labels.note); setText('refresh', labels.refresh); setText('reload', labels.reload);
    setText('confirm-reload', labels.confirmReload); setText('cancel-reload', labels.cancel);
    $('note').hidden = !labels.note;
    const rows = $('permission-rows'); rows.replaceChildren();
    for (const kind of ['camera', 'microphone', 'location', 'notifications']) {
      const row = document.createElement('label'); row.className = 'permission-row';
      const name = document.createElement('span'); name.textContent = labels.kinds[kind];
      const select = document.createElement('select'); select.setAttribute('aria-label', labels.kinds[kind]);
      for (const state of ['ask', 'allow', 'block']) {
        const option = document.createElement('option'); option.value = state; option.textContent = labels.states[state]; select.append(option);
      }
      select.value = settings[kind] || 'ask'; select.disabled = !!payload.busy;
      select.addEventListener('change', () => send('set-permission', { origin: payload.origin, kind, value: select.value }));
      row.append(name, select); rows.append(row);
    }
    $('refresh').disabled = !!payload.busy; $('reload').disabled = !!payload.busy;
    $('reload').hidden = !!payload.confirmingReload;
    $('reload-warning').hidden = !payload.confirmingReload; setText('reload-warning', labels.reloadWarning);
    $('reload-actions').hidden = !payload.confirmingReload;
  }
  function renderRequest(payload) {
    const request = payload.request, labels = payload.labels;
    setText('description', `${request.originLabel} ${labels.wantsToUse} ${labels.kinds[request.kind]}.`);
    setText('details-label', labels.moreDetails); setText('remember-label', labels.rememberBlock);
    setText('block', labels.block); setText('allow-once', labels.allowOnce); setText('allow-always', labels.allowAlways);
    const details = $('details-content'); details.replaceChildren();
    for (const detail of request.details || []) { const p = document.createElement('p'); p.textContent = detail; details.append(p); }
    const pendingLabel = document.createElement('p'); pendingLabel.textContent = `${labels.expiry} ${labels.pending}`; details.append(pendingLabel);
    $('remember').checked = false;
    ['block','allow-once','allow-always','remember','close'].forEach(id => $(id).disabled = !!payload.busy);
  }
  window.__veloceSetPermissionPopover = payload => {
    if (!payload.visible) { state = null; panel.hidden = true; closing = pending = false; return; }
    if (state && state.token !== payload.token && closing) return;
    const opening = state?.token !== payload.token;
    state = payload; if (opening) { sequence = 0; closing = false; }
    pending = false; panel.querySelectorAll('button,select,input').forEach(control => control.disabled = false);
    panel.dataset.mode = payload.mode;
    document.documentElement.lang = payload.locale; document.title = payload.labels.title;
    panel.style.setProperty('--surface', payload.theme.surface); panel.style.setProperty('--border', payload.theme.border);
    panel.style.setProperty('--text', payload.theme.text); panel.style.setProperty('--muted', payload.theme.muted);
    panel.style.setProperty('--accent', payload.theme.accent); panel.style.setProperty('--accent-text', payload.theme.accentText);
    panel.style.setProperty('--control', payload.theme.control);
    setText('title', payload.mode === 'settings' ? payload.labels.currentSite : payload.labels.requestTitle);
    setText('origin', payload.mode === 'settings' ? payload.origin : payload.request.displayOrigin);
    setText('error', payload.error); $('error').hidden = !payload.error;
    $('description').hidden = payload.mode !== 'request'; $('settings').hidden = payload.mode !== 'settings'; $('request').hidden = payload.mode !== 'request';
    $('close').hidden = payload.mode !== 'settings'; $('close').setAttribute('aria-label', payload.labels.close);
    if (payload.mode === 'settings') renderSettings(payload); else renderRequest(payload);
    panel.hidden = false; position();
    if (opening) (payload.mode === 'settings' ? $('close') : $('block')).focus({ preventScroll: true });
  };
  $('close').addEventListener('click', () => send('dismiss', {}, 'trigger'));
  $('refresh').addEventListener('click', () => send('refresh'));
  $('reload').addEventListener('click', () => send('reload'));
  $('confirm-reload').addEventListener('click', () => send('confirm-reload'));
  $('cancel-reload').addEventListener('click', () => send('cancel-reload'));
  $('block').addEventListener('click', () => send('respond', { decision: $('remember').checked ? 'block_always' : 'block' }));
  $('allow-once').addEventListener('click', () => send('respond', { decision: 'allow_once' }));
  $('allow-always').addEventListener('click', () => send('respond', { decision: 'allow_always' }));
  document.addEventListener('pointerdown', event => {
    if (!state || panel.contains(event.target)) return;
    if (state.mode === 'request') return;
    const a = state.anchor, trigger = event.clientX >= a.left && event.clientX <= a.left + a.width && event.clientY >= a.top && event.clientY <= a.top + a.height;
    send('dismiss', {}, trigger ? 'trigger' : 'outside');
  }, true);
  window.addEventListener('keydown', event => {
    if (!state) return;
    if (event.key === 'Escape') { event.preventDefault(); event.stopImmediatePropagation(); state.mode === 'request' ? send('respond', { decision: 'block' }) : send('dismiss', {}, 'escape'); }
    if (event.key === 'Tab' && state.mode === 'settings' && event.shiftKey && document.activeElement === $('close')) { event.preventDefault(); $('reload').focus(); }
  }, true);
  window.addEventListener('resize', position);
})();
