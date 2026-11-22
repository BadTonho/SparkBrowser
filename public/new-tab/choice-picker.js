// Turns a <select data-choice="..."> into a visible group of choices (one click, every option in sight).
// The <select> stays the source of truth: choosing an option sets it and fires its `change` event, so the code
// that already listens to the select needs no changes. Variants:
//  - "segmented": a row of pill buttons, with an optional small diagram per option;
//  - "font": a card per option showing a sample time in that clock font.
//  - data-choice-sample: clone the supplied template into each option and expose its value for styling.
(() => {
  const ICONS = {
    'clock-position:bottom': '<rect x="1" y="1" width="26" height="18" rx="3" fill="none" stroke="currentColor" stroke-width="1.4"/><rect x="8" y="6" width="12" height="3" rx="1.5" fill="currentColor" opacity=".45"/><rect x="9" y="12" width="10" height="4" rx="1" fill="currentColor" opacity=".8"/>',
    'clock-position:top': '<rect x="1" y="1" width="26" height="18" rx="3" fill="none" stroke="currentColor" stroke-width="1.4"/><rect x="9" y="4" width="10" height="4" rx="1" fill="currentColor" opacity=".8"/><rect x="8" y="11" width="12" height="3" rx="1.5" fill="currentColor" opacity=".45"/>',
    'clock-position:card': '<rect x="1" y="1" width="26" height="18" rx="3" fill="none" stroke="currentColor" stroke-width="1.4"/><rect x="3.5" y="3.5" width="8" height="6" rx="1.5" fill="currentColor" opacity=".8"/><rect x="8" y="13" width="12" height="3" rx="1.5" fill="currentColor" opacity=".45"/>',
  };
  const SAMPLE = '14:30';
  const groups = [];
  function enhance(select) {
    const variant = select.dataset.choice;
    const sampleId = select.dataset.choiceSample;
    const sampleContent = sampleId ? document.getElementById(sampleId)?.content : null;
    if (sampleId && !sampleContent) {
      window.__sparkLogIgnored?.(new Error(`New tab choice picker: missing sample template for ${select.id}`));
      return;
    }
    const heading = select.parentElement?.querySelector(`label[for="${select.id}"]`);
    if (heading) heading.id ||= `${select.id}-label`;
    const group = document.createElement('div');
    group.className = `choice-group choice-${variant}`;
    group.setAttribute('role', 'radiogroup');
    if (heading) group.setAttribute('aria-labelledby', heading.id);
    const items = new Map();
    const choose = value => {
      if (select.disabled || select.value === value) return;
      select.value = value;
      select.dispatchEvent(new Event('change', { bubbles: true }));
      sync();
    };
    for (const option of select.options) {
      const button = document.createElement('button');
      button.type = 'button'; button.className = 'choice'; button.setAttribute('role', 'radio');
      const label = document.createElement('span');
      label.className = 'choice-label';
      if (sampleContent) {
        button.dataset.value = option.value;
        button.append(sampleContent.cloneNode(true), label);
      } else if (variant === 'font') {
        const sample = document.createElement('span');
        sample.className = `choice-sample clock-font-${option.value}`; sample.textContent = SAMPLE; sample.setAttribute('aria-hidden', 'true');
        button.append(sample, label);
      } else {
        const icon = ICONS[`${select.id}:${option.value}`];
        if (icon) {
          const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
          svg.setAttribute('width', '28'); svg.setAttribute('height', '20'); svg.setAttribute('viewBox', '0 0 28 20'); svg.setAttribute('aria-hidden', 'true');
          svg.innerHTML = icon;
          button.append(svg);
        }
        button.append(label);
      }
      button.addEventListener('click', () => choose(option.value));
      group.append(button);
      items.set(option.value, { button, label });
    }
    group.addEventListener('keydown', event => {
      const step = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[event.key];
      if (!step || select.disabled) return;
      const values = [...select.options].map(option => option.value);
      const next = values[(values.indexOf(select.value) + step + values.length) % values.length];
      event.preventDefault(); choose(next); items.get(next)?.button.focus();
    });
    select.classList.add('choice-enhanced');
    select.after(group);
    groups.push({ select, items });
  }
  function sync() {
    for (const { select, items } of groups) {
      for (const option of select.options) {
        const item = items.get(option.value);
        if (!item) continue;
        const selected = option.value === select.value;
        item.button.setAttribute('aria-checked', String(selected));
        item.button.tabIndex = selected ? 0 : -1;
        item.button.disabled = select.disabled;
        if (item.label.textContent !== option.textContent) item.label.textContent = option.textContent;
      }
    }
  }
  for (const select of document.querySelectorAll('select[data-choice]')) enhance(select);
  sync();
  // Other scripts set the selects programmatically (no event) and the labels arrive later.
  window.__sparkChoicePickers = { sync };
  window.addEventListener('spark-new-tab', () => window.setTimeout(sync, 0));
})();
