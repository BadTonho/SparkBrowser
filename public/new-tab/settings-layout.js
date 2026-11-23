// Layout choices for the new tab, shown as segmented radio buttons and kept per device in localStorage
// like the clock preferences:
//  - where the customization panel opens (corner, the default, or centered and larger);
//  - where the widgets sit (at the sides, the default, or centered along the bottom).
(() => {
  const choices = [
    { key: 'spark_settings_panel_layout', values: ['corner', 'center'], target: 'settings-panel', group: 'settings-layout' },
    { key: 'spark_widgets_layout', values: ['sides', 'bottom'], target: 'widgets', group: 'widgets-layout' },
  ];
  const read = ({ key, values }) => { try { const value = localStorage.getItem(key); return values.includes(value) ? value : values[0]; } catch { return values[0]; } };
  for (const choice of choices) {
    const target = document.getElementById(choice.target), group = document.getElementById(choice.group);
    if (!target || !group) continue;
    const inputs = [...group.querySelectorAll('input[type="radio"]')];
    const apply = value => { target.dataset.layout = value; for (const input of inputs) input.checked = input.value === value; };
    for (const input of inputs) {
      input.addEventListener('change', () => {
        if (!input.checked) return;
        const value = choice.values.includes(input.value) ? input.value : choice.values[0];
        apply(value);
        try { localStorage.setItem(choice.key, value); } catch (error) { window.__sparkLogIgnored?.(error); }
      });
    }
    // Another new tab changed the choice.
    window.addEventListener('storage', event => { if (event.key === choice.key) apply(read(choice)); });
    apply(read(choice));
  }
})();

// Plano de fundo > Personalizado: the button that opens the editor in a tab of its own. The preset cards
// are built by custom-background.js.
(() => {
  document.getElementById('wallpaper-customize')?.addEventListener('click', () => {
    window.dispatchEvent(new CustomEvent('spark-new-tab-action', { detail: { action: 'widgets', operation: 'wallpaper_editor', data: {} } }));
  });
})();

// Plano de fundo: a vertical list of its groups (audio equalizer, background media, custom), one panel shown at a time.
(() => {
  const ids = [['wallpaper-tab-audio', 'audio-panel'], ['wallpaper-tab-media', 'wallpaper-media-panel'], ['wallpaper-tab-custom', 'wallpaper-custom-panel']];
  const tabs = ids.map(([tab, panel]) => ({ tab: document.getElementById(tab), panel: document.getElementById(panel) }));
  if (tabs.some(({ tab, panel }) => !tab || !panel)) return;
  const available = () => tabs.filter(item => !(document.documentElement.dataset.private === 'true' && item.tab.id === 'wallpaper-tab-audio'));
  function select(name) {
    const list = available();
    const chosen = list.find(item => item.tab.id === name) ?? list[0];
    for (const item of tabs) {
      const selected = item === chosen;
      item.tab.setAttribute('aria-selected', String(selected));
      item.tab.tabIndex = selected ? 0 : -1;
      item.panel.hidden = !selected;
    }
  }
  for (const item of tabs) {
    item.tab.addEventListener('click', () => select(item.tab.id));
    item.tab.addEventListener('keydown', event => {
      if (event.key !== 'ArrowUp' && event.key !== 'ArrowDown') return;
      const list = available(), at = list.indexOf(item);
      if (at < 0) return;
      const next = list[(at + (event.key === 'ArrowDown' ? 1 : list.length - 1)) % list.length];
      select(next.tab.id); next.tab.focus();
    });
  }
  // Private windows have no audio equalizer; page.js marks them after this script runs.
  new MutationObserver(() => select(tabs.find(item => item.tab.getAttribute('aria-selected') === 'true')?.tab.id)).observe(document.documentElement, { attributes: true, attributeFilter: ['data-private'] });
  select('wallpaper-tab-audio');
})();
