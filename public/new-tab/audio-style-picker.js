// Visual picker for the audio equalizer style: a grid of small previews drawn with the real style functions
// (see audio-styles.js), in place of the plain dropdown. The <select> stays the source of truth: choosing a
// card sets it and fires its `change` event, so audio-background.js and the settings store need no changes.
(() => {
  const $ = id => document.getElementById(id);
  const select = $('audio-background-style'), color = $('audio-background-color');
  const host = $('audio-background-options');
  if (!select || !color || !host) return;
  const W = 128, H = 64, FRAMES = 14;
  const grid = document.createElement('div');
  grid.id = 'audio-style-grid'; grid.setAttribute('role', 'radiogroup');
  grid.setAttribute('aria-labelledby', 'audio-style-label');
  const heading = host.querySelector('label[for="audio-background-style"]');
  if (heading) heading.id = 'audio-style-label';
  const cards = new Map();
  // Fixed, smooth-looking levels so every preview is the same and does not depend on audio playing.
  const levels = Array.from({ length: 32 }, (_, i) => 0.18 + 0.72 * Math.pow(Math.sin((i / 31) * Math.PI), 1.1) * (0.62 + 0.38 * Math.sin(i * 1.7 + 0.6)));
  function drawBars(ctx, hex) {
    const gradient = ctx.createLinearGradient(0, H, 0, 0);
    gradient.addColorStop(0, hex); gradient.addColorStop(1, `${hex}20`);
    ctx.fillStyle = gradient;
    const slot = W / 32, gap = Math.max(1, slot * 0.15);
    levels.forEach((v, i) => ctx.fillRect(i * slot + gap / 2, H - v * H * 0.9, slot - gap, v * H * 0.9));
  }
  function paint(canvas, id, hex) {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.clearRect(0, 0, W, H);
    try {
      if (id === 'bars') { drawBars(ctx, hex); return; }
      const style = window.__sparkAudioStyles?.[id];
      if (!style) return;
      // Stateful styles (particles, history, trails) need a few frames before they look like themselves.
      const store = {};
      for (let f = 0; f < FRAMES; f++) {
        ctx.clearRect(0, 0, W, H);
        style.draw(ctx, W, H, levels, { color: hex, gain: 1, t: 1 + f * 0.033, dt: 0.033, store });
      }
    } catch { ctx.clearRect(0, 0, W, H); }
  }
  function build() {
    for (const option of select.options) {
      const button = document.createElement('button');
      button.type = 'button'; button.className = 'style-card'; button.setAttribute('role', 'radio');
      button.dataset.value = option.value;
      const canvas = document.createElement('canvas');
      canvas.width = W; canvas.height = H; canvas.setAttribute('aria-hidden', 'true');
      const label = document.createElement('span');
      button.append(canvas, label);
      button.addEventListener('click', () => {
        if (select.disabled || select.value === option.value) return;
        select.value = option.value;
        select.dispatchEvent(new Event('change', { bubbles: true }));
        sync();
      });
      grid.append(button);
      cards.set(option.value, { button, canvas, label, painted: '' });
    }
    select.after(grid);
    host.classList.add('has-style-grid');
  }
  function sync() {
    const hex = /^#[0-9a-f]{6}$/i.test(color.value) ? color.value : '#1db954';
    for (const option of select.options) {
      const card = cards.get(option.value);
      if (!card) continue;
      const selected = option.value === select.value;
      card.button.setAttribute('aria-checked', String(selected));
      card.button.tabIndex = selected ? 0 : -1;
      card.button.disabled = select.disabled;
      if (card.label.textContent !== option.textContent) card.label.textContent = option.textContent;
      // Draw lazily: only while the group is visible, and again only when the color changed.
      if (!host.hidden && card.painted !== hex) { paint(card.canvas, option.value, hex); card.painted = hex; }
    }
  }
  grid.addEventListener('keydown', event => {
    const keys = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 };
    if (!(event.key in keys) || select.disabled) return;
    const values = [...select.options].map(option => option.value);
    const next = values[(values.indexOf(select.value) + keys[event.key] + values.length) % values.length];
    event.preventDefault();
    select.value = next; select.dispatchEvent(new Event('change', { bubbles: true })); sync();
    cards.get(next)?.button.focus();
  });
  color.addEventListener('input', sync);
  color.addEventListener('change', sync);
  build();
  sync();
  // The settings render sets the select programmatically (no event), and the labels arrive later.
  const render = window.__sparkAudioRender;
  if (typeof render === 'function') window.__sparkAudioRender = (...args) => { const result = render(...args); sync(); return result; };
  window.addEventListener('spark-new-tab', () => window.setTimeout(sync, 0));
})();
