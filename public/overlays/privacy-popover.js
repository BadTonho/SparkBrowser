(() => {
  const panel = document.getElementById('privacy-popover');
  const shield = document.getElementById('adblock'), autoSkip = document.getElementById('auto-skip');
  let state = null, sequence = 0, closing = false, pending = false;
  function position() {
    if (!state) return;
    const a = state.anchor;
    panel.style.left = `${Math.max(8, Math.min(a.left + a.width - panel.offsetWidth, innerWidth - panel.offsetWidth - 8))}px`;
    panel.style.top = `${Math.max(8, Math.min(a.top + a.height + 6, innerHeight - panel.offsetHeight - 8))}px`;
  }
  function send(action, value, reason = '') {
    if (!state || closing || (action !== 'dismiss' && (pending || state.busy))) return;
    const url = new URL(`veloce-internal://privacy-popover/${action}`);
    url.searchParams.set('token', state.token); url.searchParams.set('sequence', String(++sequence));
    if (typeof value === 'boolean') url.searchParams.set('value', String(value));
    url.searchParams.set('reason', reason);
    if (action === 'dismiss') closing = true;
    else { pending = true; shield.disabled = autoSkip.disabled = true; }
    window.location.href = url.toString();
  }
  window.__veloceSetPrivacyPopover = payload => {
    if (!payload.visible) { state = null; panel.hidden = true; closing = pending = false; return; }
    const opening = state?.token !== payload.token;
    if (!opening && closing) return;
    state = payload;
    if (opening) { sequence = 0; closing = false; }
    pending = false;
    document.documentElement.lang = payload.locale;
    document.title = payload.label; panel.setAttribute('aria-label', payload.label);
    const texts = { 'privacy-title': payload.label, 'adblock-label': payload.adblockLabel,
      'auto-skip-label': payload.autoSkipLabel, 'blocked-count': payload.blocked, 'auto-skip-description': payload.description,
      'privacy-error': payload.error };
    for (const [id, text] of Object.entries(texts)) document.getElementById(id).textContent = text;
    shield.checked = payload.shield; autoSkip.checked = payload.autoSkip;
    shield.disabled = autoSkip.disabled = payload.busy;
    document.getElementById('privacy-error').hidden = !payload.error;
    for (const [key, value] of Object.entries(payload.theme || {})) panel.style.setProperty(`--popover-${key}`, value);
    panel.hidden = false; position();
    if (opening) (shield.disabled ? panel : shield).focus({ preventScroll: true });
  };
  shield.addEventListener('change', () => { const value = shield.checked; shield.checked = state.shield; send('shield', value); });
  autoSkip.addEventListener('change', () => { const value = autoSkip.checked; autoSkip.checked = state.autoSkip; send('auto-skip', value); });
  document.addEventListener('pointerdown', event => {
    if (!state || panel.contains(event.target)) return;
    const a = state.anchor;
    const trigger = event.clientX >= a.left && event.clientX <= a.left + a.width && event.clientY >= a.top && event.clientY <= a.top + a.height;
    send('dismiss', undefined, trigger ? 'trigger' : 'outside');
  }, true);
  window.addEventListener('keydown', event => {
    if (!state) return;
    if (event.key === 'Escape') { event.preventDefault(); event.stopImmediatePropagation(); send('dismiss', undefined, 'escape'); }
    if (event.key === 'Tab') {
      const controls = [shield, autoSkip].filter(control => !control.disabled);
      const index = controls.indexOf(document.activeElement);
      if (!controls.length || (event.shiftKey ? index <= 0 : index === controls.length - 1)) {
        event.preventDefault(); send('dismiss', undefined, 'tab');
      }
    }
  }, true);
  window.addEventListener('resize', position);
})();
