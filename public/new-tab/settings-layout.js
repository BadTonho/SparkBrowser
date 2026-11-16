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
        try { localStorage.setItem(choice.key, value); } catch {}
      });
    }
    // Another new tab changed the choice.
    window.addEventListener('storage', event => { if (event.key === choice.key) apply(read(choice)); });
    apply(read(choice));
  }
})();
