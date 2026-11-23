// Plano de fundo > Personalizado: lists the presets saved in the background editor and draws the chosen one
// (the Nebula engine in nebula-background.js) behind the page. The choice is a local setting.
(() => {
  const ACTIVE_KEY = 'spark_custom_background';
  const PRESETS_KEY = 'spark.wallpaperPresets.v1';
  const MAX_PRESETS = 100;
  const $ = id => document.getElementById(id);
  const report = error => window.__sparkLogIgnored?.(error);
  let labels = {};
  let background = null;
  const text = key => labels[`newtab.widgets.${key}`] ?? key;
  const nebula = () => window.__sparkNebula;

  function read(key) {
    try { return JSON.parse(localStorage.getItem(key) ?? 'null'); }
    catch (error) { report(error); return null; }
  }
  /** The presets saved in the editor, with their settings checked. */
  function presets() {
    const stored = read(PRESETS_KEY);
    if (!stored || !Array.isArray(stored.items) || !nebula()) return [];
    return stored.items.slice(0, MAX_PRESETS)
      .filter(item => item && typeof item.id === 'string' && typeof item.name === 'string')
      .map(item => ({ id: item.id, name: item.name.slice(0, 40), look: typeof item.look === 'string' ? item.look : 'nebula', params: nebula().sanitize(item.params) }));
  }
  function active() {
    const value = read(ACTIVE_KEY);
    // A choice saved before there were several looks has none, and was a Nebula.
    return value && typeof value.id === 'string' && nebula() ? { id: value.id, look: typeof value.look === 'string' ? value.look : 'nebula', params: nebula().sanitize(value.params) } : null;
  }

  function choose(preset) {
    try {
      if (active()?.id === preset.id) localStorage.removeItem(ACTIVE_KEY);
      else localStorage.setItem(ACTIVE_KEY, JSON.stringify({ id: preset.id, name: preset.name, look: preset.look, params: preset.params }));
    } catch (error) { report(error); }
    apply();
    renderList();
    window.dispatchEvent(new CustomEvent('spark-custom-background'));
  }

  function renderList() {
    const list = $('wallpaper-presets');
    if (!list) return;
    for (const child of [...list.children]) if (child.id !== 'wallpaper-presets-label') child.remove();
    const saved = presets(), current = active();
    if (!saved.length) {
      const note = document.createElement('p');
      note.className = 'settings-note';
      note.textContent = text('wallpaperNoPresets');
      list.appendChild(note);
      return;
    }
    for (const preset of saved) {
      const card = document.createElement('button');
      card.type = 'button';
      card.className = 'wallpaper-preset';
      card.setAttribute('role', 'radio');
      card.setAttribute('aria-checked', String(current?.id === preset.id));
      card.dataset.preset = preset.id;
      const art = document.createElement('span');
      art.className = 'wallpaper-preset-art';
      art.setAttribute('aria-hidden', 'true');
      art.style.background = nebula().gradientCss(preset.params);
      const name = document.createElement('span');
      name.className = 'wallpaper-preset-name';
      name.textContent = preset.name;
      card.append(art, name);
      card.addEventListener('click', () => choose(preset));
      list.appendChild(card);
    }
  }

  /** Starts, updates or stops the background so it matches the setting. */
  function apply() {
    const canvas = $('custom-background'), current = active();
    if (!canvas) return;
    if (current && !background) background = nebula().start(canvas, report);
    if (current && background) {
      canvas.hidden = false;
      background.update(current.params, current.look);
      return;
    }
    background?.destroy();
    background = null;
    canvas.hidden = true;
  }

  function start() {
    apply();
    renderList();
    $('wallpaper-tab-custom')?.addEventListener('click', renderList);
    window.addEventListener('storage', event => {
      if (event.key === ACTIVE_KEY || event.key === PRESETS_KEY) { apply(); renderList(); }
    });
    window.addEventListener('spark-new-tab', event => { labels = event.detail?.labels || {}; renderList(); });
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start);
  else start();
})();
