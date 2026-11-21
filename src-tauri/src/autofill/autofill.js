(() => {
  // Autofill for the top frame of a page. It finds login, card and contact fields, asks the browser
  // for suggestions when one is focused and shows them in a list under the field. What is filled in
  // comes from the browser after the person picks an item: this script never holds saved data
  // beforehand. The list lives in a closed shadow root and only reacts to real (trusted) events.
  if (window.top !== window) return;
  const token = __AUTOFILL_TOKEN__;
  const entry = __AUTOFILL_FN__;
  const roles = __AUTOFILL_ROLES__;
  const nonce = new Uint32Array(4); crypto.getRandomValues(nonce);
  const documentId = Array.from(nonce, n => n.toString(16).padStart(8, '0')).join('');
  const portuguese = /^pt/i.test(navigator.language || '');
  const text = portuguese ? {
    locked: 'O cofre do Spark está bloqueado', hello: 'Desbloquear com o Windows Hello',
    settings: 'Desbloqueie em Configurações › Preenchimento automático e senhas',
    saveLogin: s => `Salvar a senha de ${s}?`, updateLogin: s => `Atualizar a senha de ${s}?`,
    saveCard: s => `Salvar o cartão ${s}?`, saveContact: s => `Salvar estes dados de contato (${s})?`,
    save: 'Salvar', update: 'Atualizar', notNow: 'Agora não',
  } : {
    locked: 'The Spark vault is locked', hello: 'Unlock with Windows Hello',
    settings: 'Unlock it in Settings › Autofill & passwords',
    saveLogin: s => `Save the password for ${s}?`, updateLogin: s => `Update the password for ${s}?`,
    saveCard: s => `Save the card ${s}?`, saveContact: s => `Save this contact info (${s})?`,
    save: 'Save', update: 'Update', notNow: 'Not now',
  };
  let sequence = 0;
  let current = null;
  let ui = null;

  function post(path, params) {
    const query = new URLSearchParams({ token, document: documentId, sequence: String(++sequence), ...params });
    const url = `spark-internal://autofill/${path}?${query}`;
    if (window.__sparkPostContent) window.__sparkPostContent(url); else window.location.href = url;
  }

  function usable(el) {
    if (!(el instanceof HTMLInputElement || el instanceof HTMLSelectElement) || el.disabled || el.readOnly) return false;
    const rect = el.getBoundingClientRect();
    return rect.width >= 8 && rect.height >= 8 && getComputedStyle(el).visibility !== 'hidden';
  }
  function info(el) {
    const label = el.labels && el.labels[0] ? el.labels[0].textContent : '';
    return {
      type: el instanceof HTMLSelectElement ? 'select' : el.type,
      autocomplete: el.getAttribute('autocomplete') || '',
      text: [el.name, el.id, el.placeholder, el.getAttribute('aria-label'), label].filter(Boolean).join(' '),
    };
  }
  const container = el => el.form || document;
  function passwordIn(root) {
    return [...root.querySelectorAll('input[type=password]')].find(usable) || null;
  }
  /** { role, kind } for a field, or null when it is not one autofill handles. */
  function analyze(el) {
    if (!usable(el)) return null;
    const root = container(el);
    const password = passwordIn(root);
    let role = roles.roleFor(info(el));
    if (!role && password && el instanceof HTMLInputElement && ['text', 'email', 'tel', ''].includes(el.type)) {
      // The field just before a password field is the username, whatever it is called.
      const inputs = [...root.querySelectorAll('input')].filter(usable);
      const before = inputs.slice(0, inputs.indexOf(password)).reverse().find(i => ['text', 'email', 'tel', ''].includes(i.type));
      if (before === el) role = 'username';
    }
    if (!role) return null;
    if (role === 'email' && password) role = 'username';
    const kind = roles.kindOf(role, Boolean(password));
    return kind ? { role, kind } : null;
  }

  function hide() {
    if (ui) ui.host.remove();
    ui = null;
  }

  function show(payload) {
    if (!current || !current.el.isConnected || document.activeElement !== current.el) return;
    hide();
    const host = document.createElement('div');
    const root = host.attachShadow({ mode: __AUTOFILL_SHADOW__ });
    const style = document.createElement('style');
    style.textContent = `
      :host { all: initial; }
      .list { position: fixed; z-index: 2147483647; box-sizing: border-box; min-width: 220px; max-width: 420px; max-height: 260px; overflow: auto;
        padding: 4px; border-radius: 10px; border: 1px solid rgba(128,128,128,.35); background: #20232a; color: #f2f3f5;
        box-shadow: 0 8px 28px rgba(0,0,0,.35); font: 13px/1.35 system-ui, sans-serif; }
      @media (prefers-color-scheme: light) { .list { background: #fff; color: #1d1f23; } }
      button { all: unset; display: block; width: 100%; box-sizing: border-box; padding: 7px 10px; border-radius: 7px; cursor: pointer; }
      button:hover, button.active { background: rgba(128,128,128,.22); }
      small { display: block; opacity: .7; font-size: 11.5px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
      b { font-weight: 500; display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
      p { margin: 0; padding: 7px 10px; opacity: .8; }`;
    const list = document.createElement('div');
    list.className = 'list';
    list.setAttribute('role', 'listbox');
    const choices = [];
    const add = (title, subtitle, action) => {
      const button = document.createElement('button');
      button.type = 'button';
      button.setAttribute('role', 'option');
      const strong = document.createElement('b'); strong.textContent = title; button.append(strong);
      if (subtitle) { const small = document.createElement('small'); small.textContent = subtitle; button.append(small); }
      // Real clicks only: a page cannot pick an item for the person.
      button.addEventListener('click', event => { if (event.isTrusted) { event.preventDefault(); action(); } });
      button.addEventListener('mousedown', event => event.preventDefault());
      list.append(button); choices.push({ button, action });
    };
    if (payload.state === 'locked') {
      const note = document.createElement('p'); note.textContent = text.locked; list.append(note);
      if (payload.hello) add(text.hello, '', () => post('unlock', { kind: current.kind }));
      else { const hint = document.createElement('p'); hint.textContent = text.settings; list.append(hint); }
    } else {
      for (const item of payload.items) add(item.title, item.subtitle, () => post('choose', { kind: current.kind, id: item.id }));
    }
    if (!list.childElementCount) return;
    const rect = current.el.getBoundingClientRect();
    list.style.left = `${Math.max(4, Math.min(rect.left, innerWidth - 230))}px`;
    list.style.top = `${rect.bottom + 4}px`;
    root.append(style, list);
    document.documentElement.append(host);
    ui = { host, choices, active: -1 };
  }

  function request() {
    if (!current) return;
    const rect = current.el.getBoundingClientRect();
    post('request', { kind: current.kind, role: current.role, x: Math.round(rect.left), y: Math.round(rect.top), w: Math.round(rect.width), h: Math.round(rect.height) });
  }

  function target(event) {
    const el = event.target;
    return el instanceof HTMLInputElement || el instanceof HTMLSelectElement ? el : null;
  }
  document.addEventListener('focusin', event => {
    if (!event.isTrusted) return;
    const el = target(event);
    const found = el && analyze(el);
    if (!found) { current = null; hide(); return; }
    current = { el, ...found };
    request();
  }, true);
  document.addEventListener('click', event => {
    const el = target(event);
    if (!event.isTrusted || !el || ui || !current || current.el !== el) return;
    request();
  }, true);
  document.addEventListener('input', event => { if (event.isTrusted && current && event.target === current.el) hide(); }, true);
  document.addEventListener('keydown', event => {
    if (!event.isTrusted || !current || event.target !== current.el) return;
    if (event.key === 'Escape') { hide(); return; }
    if (event.key === 'ArrowDown' && !ui) { request(); event.preventDefault(); return; }
    if (!ui || !ui.choices.length) return;
    const move = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0;
    if (move) {
      event.preventDefault();
      ui.active = (ui.active + move + ui.choices.length) % ui.choices.length;
      ui.choices.forEach((choice, index) => choice.button.classList.toggle('active', index === ui.active));
    } else if (event.key === 'Enter' && ui.active >= 0) {
      event.preventDefault();
      const { action } = ui.choices[ui.active];
      hide();
      action();
    }
  }, true);
  document.addEventListener('pointerdown', event => {
    if (ui && !event.composedPath().includes(ui.host)) hide();
  }, true);
  addEventListener('blur', hide);
  addEventListener('resize', hide);
  addEventListener('scroll', hide, true);

  function setValue(el, value) {
    const proto = el instanceof HTMLSelectElement ? HTMLSelectElement.prototype : HTMLInputElement.prototype;
    Object.getOwnPropertyDescriptor(proto, 'value').set.call(el, value);
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  }
  function choose(select, wanted) {
    const options = [...select.options];
    const match = options.find(o => wanted.includes(o.value.trim())) || options.find(o => wanted.includes(o.textContent.trim()));
    if (match) setValue(select, match.value);
  }
  function fill(kind, values) {
    if (!current || !current.el.isConnected) return;
    const root = container(current.el);
    for (const el of root.querySelectorAll('input,select')) {
      const found = analyze(el);
      if (!found || found.kind !== kind) continue;
      const role = found.role;
      if (el instanceof HTMLSelectElement) {
        if (role === 'cc-exp-month') choose(el, [values['cc-exp-month'], String(Number(values['cc-exp-month']))]);
        else if (role === 'cc-exp-year') choose(el, [values['cc-exp-year'], values['cc-exp-year'].slice(-2)]);
        else if (values[role] !== undefined) choose(el, [values[role]]);
        continue;
      }
      let value = values[role];
      if (role === 'cc-exp') value = roles.expiry(values['cc-exp-month'], values['cc-exp-year'], { placeholder: el.placeholder, maxLength: el.maxLength });
      if (typeof value === 'string' && value !== '') setValue(el, value);
    }
    hide();
  }

  // --- Saving what the person typed -------------------------------------------------------------
  const filled = el => (el.value || '').trim();
  /** Month and year from a "MM/YY", "MM / YYYY" or "MMYY" field. */
  function parseExpiry(value) {
    const match = /^(\d{1,2})\s*[\/\-.]?\s*(\d{2}|\d{4})$/.exec(value.trim());
    if (!match) return null;
    const year = match[2].length === 2 ? 2000 + Number(match[2]) : Number(match[2]);
    return { month: String(Number(match[1])), year: String(year) };
  }
  /** What a submitted form holds, as { kind, values } for the browser to compare with the vault. */
  function capture(form) {
    const found = {};
    for (const el of form.querySelectorAll('input,select')) {
      const info = analyze(el);
      if (!info || !filled(el)) continue;
      if (info.role === 'password' && found.password && !/new-password/.test(el.getAttribute('autocomplete') || '')) continue;
      found[info.role] = filled(el);
    }
    if (found['cc-exp'] && !found['cc-exp-month']) {
      const expiry = parseExpiry(found['cc-exp']);
      if (expiry) { found['cc-exp-month'] = expiry.month; found['cc-exp-year'] = expiry.year; }
    }
    if (found['cc-number']) {
      const values = { 'cc-number': found['cc-number'], 'cc-name': found['cc-name'] || '', 'cc-exp-month': found['cc-exp-month'] || '', 'cc-exp-year': found['cc-exp-year'] || '' };
      return { kind: 'card', values };
    }
    if (found.password) return { kind: 'login', values: { username: found.username || found.email || '', password: found.password } };
    const contact = {};
    for (const [role, value] of Object.entries(found)) if (!role.startsWith('cc-') && role !== 'username') contact[role] = value;
    return Object.keys(contact).length >= 3 ? { kind: 'contact', values: contact } : null;
  }
  document.addEventListener('submit', event => {
    if (!event.isTrusted || !(event.target instanceof HTMLFormElement)) return;
    const found = capture(event.target);
    if (found) post('capture', { kind: found.kind, data: JSON.stringify(found.values), origin: location.origin });
  }, true);

  let banner = null;
  function closeBanner() { if (banner) banner.host.remove(); banner = null; }
  /** A bar at the top of the page: "Save the password for example.com?". Real clicks only. */
  function prompt(message) {
    closeBanner();
    const host = document.createElement('div');
    const root = host.attachShadow({ mode: __AUTOFILL_SHADOW__ });
    const style = document.createElement('style');
    style.textContent = `
      :host { all: initial; }
      .bar { position: fixed; z-index: 2147483647; top: 12px; right: 12px; box-sizing: border-box; max-width: min(420px, calc(100vw - 24px));
        display: flex; flex-direction: column; gap: 10px; padding: 12px 14px; border-radius: 12px; border: 1px solid rgba(128,128,128,.35);
        background: #20232a; color: #f2f3f5; box-shadow: 0 8px 28px rgba(0,0,0,.35); font: 13px/1.4 system-ui, sans-serif; }
      @media (prefers-color-scheme: light) { .bar { background: #fff; color: #1d1f23; } }
      .row { display: flex; gap: 8px; justify-content: flex-end; }
      button { all: unset; padding: 6px 12px; border-radius: 8px; cursor: pointer; background: rgba(128,128,128,.22); }
      button.primary { background: #3b82f6; color: #fff; }
      button:focus-visible { outline: 2px solid #93c5fd; }`;
    const bar = document.createElement('div');
    bar.className = 'bar'; bar.setAttribute('role', 'alertdialog');
    const label = document.createElement('div');
    const sentence = message.kind === 'login' ? (message.mode === 'update' ? text.updateLogin : text.saveLogin)
      : message.kind === 'card' ? text.saveCard : text.saveContact;
    label.textContent = sentence(message.subject);
    const row = document.createElement('div'); row.className = 'row';
    const decide = (name, path, primary) => {
      const button = document.createElement('button');
      button.type = 'button'; button.textContent = name; if (primary) button.className = 'primary';
      button.addEventListener('click', event => { if (!event.isTrusted) return; event.preventDefault(); closeBanner(); post(path, { kind: message.kind }); });
      return button;
    };
    row.append(decide(text.notNow, 'dismiss', false), decide(message.mode === 'update' ? text.update : text.save, 'save', true));
    bar.append(label, row);
    root.append(style, bar);
    document.documentElement.append(host);
    banner = { host };
  }
  // A page that follows a submit (the login redirects) tells the browser it is there, so a pending
  // "save this?" can be shown on it.
  const announce = () => post('ready', { kind: 'login' });
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', announce, { once: true }); else announce();

  // The browser reaches the page only through this one name, which is not writable and not guessable.
  Object.defineProperty(window, entry, {
    value(message) {
      if (message.op === 'show') show(message);
      else if (message.op === 'fill') fill(message.kind, message.values);
      else if (message.op === 'prompt') prompt(message);
      else if (message.op === 'close-prompt') closeBanner();
    },
  });
})();
