(() => {
  const panel = document.getElementById('history-panel');
  const $ = id => document.getElementById(id);
  const menu = $('menu');
  let state = null, sequence = 0, closing = false, inputTimer = 0;
  /** Checked visits (ids). Kept across paints while those visits are still shown. */
  const selected = new Set();
  let lastChecked = null, menuEntry = null, menuButton = null;

  function send(action, value = {}) {
    if (!state || closing) return;
    if (action === 'close' || action === 'open' || action === 'open-many') closing = true;
    const url = new URL('spark-internal://history-overlay/action');
    url.searchParams.set('token', state.token);
    url.searchParams.set('sequence', String(++sequence));
    url.searchParams.set('payload', JSON.stringify({ action, ...value }));
    location.href = url.toString();
  }

  function closeMenu(focusButton = false) {
    if (menu.hidden) return;
    menu.hidden = true;
    menuButton?.setAttribute('aria-expanded', 'false');
    if (focusButton) menuButton?.focus();
    menuEntry = null; menuButton = null;
  }

  function openMenu(entry, button) {
    closeMenu();
    menuEntry = entry; menuButton = button;
    button.setAttribute('aria-expanded', 'true');
    menu.hidden = false;
    const rect = button.getBoundingClientRect();
    const { width, height } = menu.getBoundingClientRect();
    const top = rect.bottom + height + 8 > innerHeight ? rect.top - height - 4 : rect.bottom + 4;
    menu.style.left = `${Math.max(8, Math.min(rect.right - width, innerWidth - width - 8))}px`;
    menu.style.top = `${Math.max(8, top)}px`;
    menu.querySelector('button')?.focus();
  }

  /** Shows the selection bar instead of the title and search while visits are checked. */
  function paintSelection() {
    const count = selected.size;
    $('main-bar').hidden = count > 0;
    $('selection-bar').hidden = count === 0;
    if (state) $('selected-count').textContent = state.labels.selected.replace('{count}', String(count));
    for (const row of document.querySelectorAll('.history-row')) {
      const on = selected.has(Number(row.dataset.id));
      row.classList.toggle('selected', on);
      row.querySelector('input').checked = on;
    }
  }

  function toggle(id, on, range) {
    const ids = state.entries.map(entry => entry.id);
    if (range && lastChecked !== null && ids.includes(lastChecked)) {
      const [from, to] = [ids.indexOf(lastChecked), ids.indexOf(id)].sort((a, b) => a - b);
      for (const each of ids.slice(from, to + 1)) on ? selected.add(each) : selected.delete(each);
    } else {
      on ? selected.add(id) : selected.delete(id);
    }
    lastChecked = id;
    paintSelection();
  }

  function favicon(entry, icons) {
    const icon = entry.site ? icons[entry.site] : null;
    if (typeof icon === 'string' && icon.startsWith('data:image/')) {
      const img = document.createElement('img'); img.className = 'favicon'; img.alt = ''; img.src = icon; img.draggable = false;
      return img;
    }
    const letter = document.createElement('span'); letter.className = 'favicon letter'; letter.setAttribute('aria-hidden', 'true');
    letter.textContent = (entry.host || entry.title || '?').slice(0, 1).toUpperCase();
    return letter;
  }

  function row(entry, labels, icons) {
    const item = document.createElement('div'); item.className = 'history-row'; item.dataset.id = String(entry.id); item.dataset.testid = 'history-row';
    const check = document.createElement('label'); check.className = 'check'; check.title = labels.select;
    const box = document.createElement('input'); box.type = 'checkbox'; box.setAttribute('aria-label', `${labels.select}: ${entry.title || entry.url}`);
    box.addEventListener('click', event => toggle(entry.id, box.checked, event.shiftKey));
    check.append(box);
    const time = document.createElement('span'); time.className = 'time'; time.textContent = entry.time;
    const link = document.createElement('button'); link.type = 'button'; link.className = 'link'; link.title = entry.url;
    const title = document.createElement('span'); title.className = 'title'; title.textContent = entry.title || entry.url;
    const host = document.createElement('span'); host.className = 'host'; host.textContent = entry.host;
    link.append(title, host);
    link.addEventListener('click', event => {
      // With a selection, clicking a row checks it, like the box; otherwise it opens the page.
      if (selected.size > 0) { toggle(entry.id, !selected.has(entry.id), event.shiftKey); return; }
      send('open', { entryId: entry.id });
    });
    const kebab = document.createElement('button'); kebab.type = 'button'; kebab.className = 'kebab'; kebab.textContent = '⋮';
    kebab.setAttribute('aria-label', labels.menu); kebab.setAttribute('aria-haspopup', 'menu'); kebab.setAttribute('aria-expanded', 'false');
    kebab.addEventListener('click', event => { event.stopPropagation(); menuEntry === entry ? closeMenu() : openMenu(entry, kebab); });
    item.append(check, time, favicon(entry, icons), link, kebab);
    return item;
  }

  function render(payload) {
    const labels = payload.labels;
    document.documentElement.lang = payload.locale;
    document.title = labels.title;
    for (const [name, value] of Object.entries({
      surface: payload.theme.surface, 'surface-border': payload.theme.surfaceBorder ?? '', 'surface-filter': payload.theme.surfaceFilter ?? '',
      'surface-shadow': payload.theme.surfaceShadow ?? '', border: payload.theme.border, text: payload.theme.text, muted: payload.theme.muted,
      accent: payload.theme.accent, hover: payload.theme.hover, field: payload.theme.field,
    })) document.documentElement.style.setProperty(`--${name}`, value);
    $('title').textContent = labels.title;
    $('clear').textContent = labels.clear;
    $('clear').hidden = payload.entries.length === 0;
    $('clear').disabled = payload.busy;
    $('close').setAttribute('aria-label', labels.close);
    $('cancel-selection').setAttribute('aria-label', labels.cancelSelection);
    $('open-selected').textContent = labels.open;
    $('delete-selected').textContent = labels.delete;
    $('open-selected').disabled = payload.busy;
    $('delete-selected').disabled = payload.busy;
    menu.querySelector('[data-action="more-from-site"]').textContent = labels.moreFromSite;
    menu.querySelector('[data-action="delete"]').textContent = labels.deleteEntry;
    $('query').placeholder = labels.search;
    if (document.activeElement !== $('query')) $('query').value = payload.query;
    $('error').textContent = payload.error || '';
    $('error').hidden = !payload.error;
    closeMenu();
    const shown = new Set(payload.entries.map(entry => entry.id));
    for (const id of [...selected]) if (!shown.has(id)) selected.delete(id);

    const entries = $('entries');
    entries.setAttribute('aria-busy', String(payload.busy));
    const scroll = entries.scrollTop;
    entries.replaceChildren();
    if (payload.entries.length === 0) {
      const empty = document.createElement('div'); empty.className = 'empty-state'; empty.textContent = labels.empty; entries.append(empty);
    } else {
      let card = null, list = null, day = null;
      for (const entry of payload.entries) {
        if (entry.day !== day || !card) {
          day = entry.day;
          card = document.createElement('section'); card.className = 'day';
          const heading = document.createElement('h3'); heading.textContent = entry.day;
          list = document.createElement('div'); list.className = 'rows';
          card.append(heading, list); entries.append(card);
        } else if (entry.gap) {
          const gap = document.createElement('div'); gap.className = 'gap'; gap.setAttribute('aria-hidden', 'true'); gap.append(document.createElement('span'));
          list.append(gap);
        }
        list.append(row(entry, labels, payload.icons || {}));
      }
    }
    if (payload.hasMore) {
      const more = document.createElement('button'); more.className = 'more'; more.type = 'button'; more.textContent = labels.loadMore; more.disabled = payload.busy;
      more.addEventListener('click', () => send('load-more')); entries.append(more);
    }
    entries.scrollTop = scroll;
    paintSelection();
    panel.hidden = false;
  }

  window.__sparkSetHistoryOverlay = payload => {
    if (!payload.visible) {
      if (!state || state.sessionId <= payload.sessionId) { state = null; panel.hidden = true; closing = false; selected.clear(); closeMenu(); }
      return;
    }
    if (state && state.sessionId > payload.sessionId) return;
    const opening = state?.sessionId !== payload.sessionId;
    state = payload;
    if (opening) { sequence = 0; closing = false; selected.clear(); lastChecked = null; }
    render(payload);
    if (opening) $('query').focus({ preventScroll: true });
  };

  function clearSelection() { selected.clear(); lastChecked = null; paintSelection(); }

  $('clear').addEventListener('click', () => send('clear'));
  $('close').addEventListener('click', () => send('close'));
  $('cancel-selection').addEventListener('click', () => { clearSelection(); $('query').focus(); });
  $('open-selected').addEventListener('click', () => { if (selected.size) send('open-many', { entryIds: [...selected] }); });
  $('delete-selected').addEventListener('click', () => {
    if (!selected.size) return;
    send('delete', { entryIds: [...selected] });
    clearSelection();
  });
  menu.addEventListener('click', event => {
    const action = event.target.closest('button')?.dataset.action;
    const entry = menuEntry;
    if (!action || !entry) return;
    closeMenu();
    if (action === 'more-from-site') send('more-from-site', { entryId: entry.id });
    if (action === 'delete') { selected.delete(entry.id); send('delete', { entryIds: [entry.id] }); paintSelection(); }
  });
  $('query').addEventListener('input', () => {
    clearTimeout(inputTimer);
    const query = $('query').value;
    inputTimer = setTimeout(() => send('search', { query }), 180);
  });
  document.addEventListener('pointerdown', event => {
    if (!menu.hidden && !menu.contains(event.target) && !event.target.closest?.('.kebab')) closeMenu();
    if (state && !panel.contains(event.target) && !menu.contains(event.target)) send('close');
  }, true);
  window.addEventListener('keydown', event => {
    if (!state) return;
    if (event.key === 'Escape') {
      event.preventDefault(); event.stopImmediatePropagation();
      // Escape closes the menu first, then the selection, then the panel.
      if (!menu.hidden) closeMenu(true);
      else if (selected.size) clearSelection();
      else send('close');
      return;
    }
    if (!menu.hidden && (event.key === 'ArrowDown' || event.key === 'ArrowUp')) {
      event.preventDefault();
      const items = [...menu.querySelectorAll('button')];
      const index = items.indexOf(document.activeElement);
      items[(index + (event.key === 'ArrowDown' ? 1 : -1) + items.length) % items.length].focus();
      return;
    }
    if (event.key === 'Tab' && !menu.hidden) { event.preventDefault(); closeMenu(true); return; }
    if (event.key === 'Tab') {
      const controls = [...panel.querySelectorAll('button:not([disabled]):not([hidden]),input:not([disabled]):not([hidden])')].filter(node => node.getClientRects().length);
      if (!controls.length) { event.preventDefault(); panel.focus(); return; }
      const first = controls[0], last = controls.at(-1);
      if (event.shiftKey && (document.activeElement === first || !panel.contains(document.activeElement))) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    }
  }, true);
})();
