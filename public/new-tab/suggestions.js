// The suggestion list of the search field. Rows and ranking are the address bar's (`suggestions-core.js`);
// history, bookmarks and search terms come from the native side through the page bridge.
(() => {
  const input = document.querySelector('#search');
  const list = document.querySelector('#suggestions');
  const form = document.querySelector('#search-form');
  const core = window.SparkSuggestions;
  if (!input || !list || !form || !core) return;

  const DEBOUNCE_MS = 70;
  const WEB_DELAY_MS = 150;
  const ENGINE_LABELS = { google: 'Google', duckduckgo: 'DuckDuckGo', bing: 'Bing', brave: 'Brave Search' };
  const SVG = 'http://www.w3.org/2000/svg';
  const GLYPHS = {
    bookmark: 'm12 3 2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9Z',
    history: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18ZM12 7v5l3 2',
    navigate: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18ZM3 12h18M12 3c3 3.5 3 14.5 0 18M12 3c-3 3.5-3 14.5 0 18',
    search: 'M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14ZM21 21l-4.3-4.3',
  };

  let labels = {};
  let engine = 'google';
  let isPrivate = false;
  let open = false;
  let items = [];
  let selected = -1;
  /** What is typed; the field shows nothing else while the arrows only move the highlight. */
  let query = '';
  let data = null;
  let terms = [];
  let request = 0;
  let timer;
  let termsTimer;

  const send = (action, text) => window.dispatchEvent(new CustomEvent('spark-new-tab-action', { detail: { action, input: text, request } }));

  // The same setting as the address bar, shared through storage like the theme. On unless turned off.
  const webEnabled = () => {
    try { return localStorage.getItem('spark_web_suggestions') !== 'false'; }
    catch (error) { window.__sparkLogIgnored?.(error); return true; }
  };

  function cancel() {
    clearTimeout(timer);
    clearTimeout(termsTimer);
    request += 1;
  }

  function close() {
    cancel();
    open = false;
    items = [];
    selected = -1;
    data = null;
    terms = [];
    render();
  }

  function ask(text, immediate) {
    cancel();
    open = true;
    query = text;
    selected = -1;
    terms = [];
    // Typing restarts the pause, so only the latest text is asked for.
    if (immediate) send('suggest', query);
    else timer = setTimeout(() => send('suggest', query), DEBOUNCE_MS);
  }

  // Terms need the option on, a normal window, words (not an address) and an engine that has them.
  function askTerms() {
    const text = query.trim();
    if (!text || isPrivate || !webEnabled() || core.looksLikeAddress(text) || !core.supportsWebSuggestions(engine)) return;
    clearTimeout(termsTimer);
    termsTimer = setTimeout(() => send('terms', text), WEB_DELAY_MS);
  }

  function build() {
    const highlighted = selected >= 0 ? items[selected]?.id : undefined;
    items = core.buildSuggestions({
      query,
      history: data.history,
      bookmarks: data.bookmarks,
      webTerms: terms,
      engineLabel: ENGINE_LABELS[engine] ?? engine,
    });
    selected = highlighted ? items.findIndex(item => item.id === highlighted) : -1;
    render();
  }

  function glyph(kind) {
    const svg = document.createElementNS(SVG, 'svg');
    for (const [name, value] of Object.entries({ width: '15', height: '15', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '1.8', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' })) svg.setAttribute(name, value);
    const path = document.createElementNS(SVG, 'path');
    path.setAttribute('d', GLYPHS[kind] ?? GLYPHS.search);
    svg.append(path);
    return svg;
  }

  function icon(item) {
    const key = item.kind === 'bookmark' || item.kind === 'history' ? data?.sites?.[item.target] : undefined;
    const source = key ? data.icons?.[key] : undefined;
    if (typeof source !== 'string' || !/^data:image\/png;base64,[A-Za-z0-9+/=]+$/.test(source)) return glyph(item.kind);
    const image = document.createElement('img');
    image.className = 'suggestion-favicon';
    image.alt = '';
    image.width = 16;
    image.height = 16;
    image.src = source;
    return image;
  }

  function span(className, text) {
    const node = document.createElement('span');
    node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }

  function label(kind) {
    const name = { search: 'suggest_search', navigate: 'suggest_go', bookmark: 'suggest_bookmark', web: 'suggest_web' }[kind] ?? 'suggest_history';
    return (labels[`newtab.${name}`] ?? '').replace('{engine}', ENGINE_LABELS[engine] ?? engine);
  }

  function row(item, index) {
    const button = document.createElement('button');
    button.type = 'button';
    button.id = `suggestion-${index}`;
    button.className = 'suggestion';
    // The field keeps the keyboard: Tab goes past the list, the arrows walk it.
    button.tabIndex = -1;
    button.setAttribute('role', 'option');
    const mark = span('suggestion-icon');
    mark.setAttribute('aria-hidden', 'true');
    mark.append(icon(item));
    const text = span('suggestion-text');
    text.append(span('suggestion-primary', item.primary));
    const action = item.kind === 'search' || item.kind === 'navigate';
    if (action) text.append(span('suggestion-secondary', label(item.kind)));
    button.append(mark, text);
    if (!action) button.append(span('suggestion-kind', label(item.kind)));
    button.addEventListener('click', () => choose(item));
    return button;
  }

  function highlight() {
    [...list.children].forEach((node, index) => {
      node.className = index === selected ? 'suggestion highlighted' : 'suggestion';
      node.setAttribute('aria-selected', String(index === selected));
    });
    if (selected >= 0) input.setAttribute('aria-activedescendant', `suggestion-${selected}`);
    else input.removeAttribute('aria-activedescendant');
  }

  function render() {
    list.replaceChildren(...items.map(row));
    list.hidden = !open || !items.length;
    input.setAttribute('aria-expanded', String(!list.hidden));
    highlight();
  }

  function choose(item) {
    input.value = item.target;
    close();
    send('navigate', item.target);
  }

  input.setAttribute('role', 'combobox');
  input.setAttribute('aria-controls', 'suggestions');
  input.setAttribute('aria-autocomplete', 'list');
  input.setAttribute('aria-expanded', 'false');

  // Not on focus: a new tab focuses the field by itself and should open without a list over it.
  input.addEventListener('pointerdown', () => { if (!open) ask(input.value, true); });
  input.addEventListener('input', () => ask(input.value, false));
  input.addEventListener('blur', close);
  input.addEventListener('keydown', event => {
    if (event.isComposing) return;
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      if (list.hidden) {
        if (event.key === 'ArrowDown') { event.preventDefault(); ask(input.value, true); }
        return;
      }
      event.preventDefault();
      // Past either end the highlight returns to "nothing highlighted".
      selected += event.key === 'ArrowDown' ? 1 : -1;
      if (selected >= items.length) selected = -1;
      else if (selected < -1) selected = items.length - 1;
      highlight();
    } else if (event.key === 'Escape') {
      if (!list.hidden) event.preventDefault();
      close();
    } else if (event.key === 'Enter' && !list.hidden && selected >= 0) {
      event.preventDefault();
      choose(items[selected]);
    }
  });
  // Cancelled so the field keeps the focus until the click lands.
  list.addEventListener('mousedown', event => event.preventDefault());
  form.addEventListener('submit', close);

  window.addEventListener('spark-new-tab', event => {
    labels = event.detail.labels;
    engine = event.detail.preferences.engine;
    isPrivate = Boolean(event.detail.private);
    list.setAttribute('aria-label', labels['newtab.suggestions']);
  });
  window.addEventListener('spark-new-tab-suggestions', event => {
    const reply = event.detail;
    if (!open || reply.request !== request) return;
    if (reply.local) {
      data = reply.local;
      build();
      askTerms();
    } else if (data && Array.isArray(reply.terms)) {
      terms = reply.terms;
      build();
    }
  });
})();
