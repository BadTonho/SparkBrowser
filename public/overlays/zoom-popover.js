(() => {
  const popover = document.getElementById('zoom-popover');
  const slider = document.getElementById('page-zoom');
  const percentage = document.getElementById('zoom-percentage');
  const decrease = document.getElementById('zoom-out');
  const increase = document.getElementById('zoom-in');
  const reset = document.getElementById('zoom-reset');
  const error = document.getElementById('zoom-error');
  let state = null, sequence = 0, closing = false, editing = false, draft = null, dirty = false, pendingFrame = 0;
  const factor = () => draft ?? state.factor;
  function render() {
    const value = factor();
    const text = `${new Intl.NumberFormat(state.locale).format(Math.round(value * 100))}%`;
    percentage.textContent = text;
    slider.setAttribute('aria-valuetext', text);
    slider.max = String(state.levels.length - 1);
    slider.value = String(state.levels.reduce((best, level, index) => Math.abs(level - value) < Math.abs(state.levels[best] - value) ? index : best, 0));
    decrease.disabled = value <= state.levels[0];
    increase.disabled = value >= state.levels.at(-1);
    error.textContent = state.error;
    error.hidden = !state.error;
  }
  function position() {
    if (!state) return;
    const { anchor } = state;
    popover.style.left = `${Math.max(8, Math.min(anchor.left + anchor.width / 2 - popover.offsetWidth / 2, innerWidth - popover.offsetWidth - 8))}px`;
    popover.style.top = `${Math.max(8, Math.min(anchor.top + anchor.height + 6, innerHeight - popover.offsetHeight - 8))}px`;
  }
  function send(action, reason = '') {
    if (!state || closing) return;
    cancelAnimationFrame(pendingFrame); pendingFrame = 0;
    const url = new URL(`spark-internal://zoom-popover/${action}`);
    url.searchParams.set('token', state.token);
    url.searchParams.set('sequence', String(++sequence));
    if (dirty) url.searchParams.set('value', String(draft));
    if (reason) url.searchParams.set('reason', reason);
    if (action === 'dismiss') closing = true;
    window.location.href = url.toString();
  }
  function adjust(direction) {
    if (!state || closing) return;
    const value = factor();
    draft = direction === 0 ? 1 : direction > 0 ? state.levels.find(level => level > value + .001) ?? state.levels.at(-1) : [...state.levels].reverse().find(level => level < value - .001) ?? state.levels[0];
    dirty = true; render(); send('zoom');
  }
  window.__sparkSetZoomPopover = payload => {
    if (!payload.visible) {
      cancelAnimationFrame(pendingFrame); pendingFrame = 0;
      state = null; closing = false; editing = false; draft = null; dirty = false; popover.hidden = true;
      return;
    }
    const opening = state?.token !== payload.token;
    if (!opening && closing) return;
    state = payload;
    if (opening) { sequence = 0; closing = false; editing = false; draft = null; dirty = false; }
    else if (!payload.pending && (payload.error || (!editing && Math.abs(payload.factor - draft) < .00001))) { draft = null; dirty = false; }
    document.documentElement.lang = payload.locale;
    document.title = payload.label;
    popover.setAttribute('aria-label', payload.label);
    slider.setAttribute('aria-label', payload.label);
    decrease.setAttribute('aria-label', payload.outLabel);
    increase.setAttribute('aria-label', payload.inLabel);
    reset.textContent = payload.resetLabel;
    for (const [name, value] of Object.entries(payload.theme || {})) popover.style.setProperty(`--popover-${name}`, value);
    render(); popover.hidden = false; position();
    if (opening) slider.focus({ preventScroll:true });
  };
  decrease.addEventListener('click', () => adjust(-1));
  increase.addEventListener('click', () => adjust(1));
  reset.addEventListener('click', () => adjust(0));
  slider.addEventListener('pointerdown', () => { editing = true; });
  slider.addEventListener('input', () => {
    draft = state.levels[Number(slider.value)]; dirty = true; render();
    if (!pendingFrame) pendingFrame = requestAnimationFrame(() => { pendingFrame = 0; send('zoom'); });
  });
  slider.addEventListener('change', () => { editing = false; send('zoom'); });
  window.addEventListener('pointerup', () => { if (editing) { editing = false; send('zoom'); } });
  window.addEventListener('pointercancel', () => { if (editing) { editing = false; send('zoom'); } });
  document.addEventListener('pointerdown', event => {
    if (!state || popover.contains(event.target)) return;
    const { anchor } = state;
    const trigger = event.clientX >= anchor.left && event.clientX <= anchor.left + anchor.width && event.clientY >= anchor.top && event.clientY <= anchor.top + anchor.height;
    send('dismiss', trigger ? 'trigger' : 'outside');
  }, true);
  window.addEventListener('keydown', event => {
    if (!state) return;
    if (event.ctrlKey && ['+', '=', '-', '0', 'Add', 'Subtract'].includes(event.key)) {
      event.preventDefault(); event.stopImmediatePropagation();
      adjust(event.key === '0' ? 0 : ['-', 'Subtract'].includes(event.key) ? -1 : 1); return;
    }
    const controls = [...popover.querySelectorAll('button:not(:disabled),input:not(:disabled)')];
    const index = controls.indexOf(document.activeElement);
    if (event.key === 'Escape' || (event.key === 'Tab' && (event.shiftKey ? index <= 0 : index === controls.length - 1))) {
      event.preventDefault(); event.stopImmediatePropagation(); send('dismiss', event.key === 'Escape' ? 'escape' : 'tab');
    }
  }, true);
  window.addEventListener('resize', position);
})();
