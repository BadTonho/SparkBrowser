// Lets the user place the customization panel in the corner (default) or centered and larger.
// The choice is a per-device convenience kept in localStorage, like the clock preferences.
(() => {
  const KEY = 'spark_settings_panel_layout', LAYOUTS = ['corner', 'center'];
  const panel = document.getElementById('settings-panel'), select = document.getElementById('settings-layout');
  if (!panel || !select) return;
  const read = () => { try { const value = localStorage.getItem(KEY); return LAYOUTS.includes(value) ? value : 'corner'; } catch { return 'corner'; } };
  const apply = layout => { panel.dataset.layout = layout; select.value = layout; };
  select.addEventListener('change', () => {
    const layout = LAYOUTS.includes(select.value) ? select.value : 'corner';
    apply(layout);
    try { localStorage.setItem(KEY, layout); } catch {}
  });
  apply(read());
})();
