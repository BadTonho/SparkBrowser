(() => {
  // Autofill for the top frame of a page. It finds login, card and contact fields, asks the browser
  // for suggestions when one is focused and shows them in a list under the field. What is filled in
  // comes from the browser after the person picks an item: this script never holds saved data
  // beforehand. The list lives in a closed shadow root and only reacts to real (trusted) events.
  //
  // This script shares its JavaScript world with the page, which can replace any global or prototype
  // method. Everything that touches the channel token, the item ids or the functions that act on
  // the person's behalf is therefore taken now, before any page script runs, and called through
  // `apply`: a page that later replaces `URLSearchParams`, `addEventListener`, `Array.prototype.push`
  // or an event getter gains nothing (it can neither read the token nor call a handler itself).
  if (window.top !== window) return;
  const token = __AUTOFILL_TOKEN__;
  const entry = __AUTOFILL_FN__;
  const roles = __AUTOFILL_ROLES__;
  const shadowMode = __AUTOFILL_SHADOW__;

  const apply = Reflect.apply;
  const describe = Object.getOwnPropertyDescriptor;
  const define = Object.defineProperty;
  const ownKeys = Object.keys;
  const blank = () => Object.create(null);
  const stringify = JSON.stringify;
  const encode = encodeURIComponent;
  const send = window.__sparkPostContent;
  const getter = (proto, name) => describe(proto, name).get;
  const setter = (proto, name) => describe(proto, name).set;
  const listen = EventTarget.prototype.addEventListener;
  const eventTarget = getter(Event.prototype, 'target');
  const eventKey = getter(KeyboardEvent.prototype, 'key');
  const preventDefault = Event.prototype.preventDefault;
  const composedPath = Event.prototype.composedPath;
  const activeElement = getter(Document.prototype, 'activeElement');
  const rootElement = getter(Document.prototype, 'documentElement');
  const createElement = Document.prototype.createElement;
  const attachShadow = Element.prototype.attachShadow;
  const appendToElement = Element.prototype.append;
  const appendToFragment = DocumentFragment.prototype.append;
  const removeElement = Element.prototype.remove;
  const setAttribute = Element.prototype.setAttribute;
  const setText = setter(Node.prototype, 'textContent');
  const isConnected = getter(Node.prototype, 'isConnected');
  const rectOf = Element.prototype.getBoundingClientRect;

  const on = (target, type, handler, options) => apply(listen, target, [type, handler, options]);
  /** Only an event the browser created for a real action of the person. */
  function trusted(event) {
    try { apply(eventTarget, event, []); } catch { return false; }
    return event.isTrusted === true;
  }
  const targetOf = event => apply(eventTarget, event, []);
  const keyOf = event => apply(eventKey, event, []);
  const cancel = event => apply(preventDefault, event, []);
  function element(tag, attributes, content) {
    const node = apply(createElement, document, [tag]);
    if (attributes) {
      const names = ownKeys(attributes);
      for (let i = 0; i < names.length; i++) apply(setAttribute, node, [names[i], attributes[names[i]]]);
    }
    if (content !== undefined) apply(setText, node, [content]);
    return node;
  }
  function shadowHost() {
    const host = element('div');
    const options = blank(); options.mode = shadowMode;
    return { host, root: apply(attachShadow, host, [options]) };
  }

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

  /** The query is built by hand: only captured functions and string concatenation touch the token. */
  function post(path, params) {
    if (typeof send !== 'function') return;
    sequence += 1;
    let query = 'token=' + encode(token) + '&document=' + encode(documentId) + '&sequence=' + sequence;
    const names = ownKeys(params);
    for (let i = 0; i < names.length; i++) query += '&' + encode(names[i]) + '=' + encode('' + params[names[i]]);
    send('spark-internal://autofill/' + path + '?' + query);
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
    if (ui) apply(removeElement, ui.host, []);
    ui = null;
  }
  const focused = () => current && apply(isConnected, current.el, []) && apply(activeElement, document, []) === current.el;

  const LIST_STYLE = `
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

  function show(payload) {
    if (!focused()) return;
    hide();
    const { host, root } = shadowHost();
    const list = element('div', { class: 'list', role: 'listbox' });
    // Not an array: arrays go through Array.prototype, which the page controls.
    const choices = blank();
    let count = 0, shown = 0;
    const add = (title, subtitle, action) => {
      const button = element('button', { type: 'button', role: 'option' });
      apply(appendToElement, button, [element('b', null, '' + title)]);
      if (subtitle) apply(appendToElement, button, [element('small', null, '' + subtitle)]);
      // Real clicks only: a page cannot pick an item for the person.
      on(button, 'click', event => { if (trusted(event)) { cancel(event); action(); } });
      on(button, 'mousedown', event => cancel(event));
      apply(appendToElement, list, [button]);
      const choice = blank(); choice.button = button; choice.action = action;
      choices[count] = choice; count += 1; shown += 1;
    };
    const note = content => { apply(appendToElement, list, [element('p', null, content)]); shown += 1; };
    if (payload.state === 'locked') {
      note(text.locked);
      if (payload.hello) {
        const kind = current.kind;
        add(text.hello, '', () => post('unlock', { kind }));
      } else note(text.settings);
    } else {
      const items = payload.items;
      for (let i = 0; items && i < items.length; i++) {
        const item = items[i];
        const kind = current.kind, id = '' + item.id;
        add(item.title, item.subtitle, () => post('choose', { kind, id }));
      }
    }
    if (!shown) return;
    const rect = apply(rectOf, current.el, []);
    apply(setAttribute, list, ['style', `left: ${Math.max(4, Math.min(rect.left, innerWidth - 230))}px; top: ${rect.bottom + 4}px`]);
    apply(appendToFragment, root, [element('style', null, LIST_STYLE), list]);
    apply(appendToElement, apply(rootElement, document, []), [host]);
    ui = blank(); ui.host = host; ui.choices = choices; ui.count = count; ui.active = -1;
  }

  function highlight(index) {
    for (let i = 0; i < ui.count; i++) apply(setAttribute, ui.choices[i].button, ['class', i === index ? 'active' : '']);
  }

  function request() {
    if (!current) return;
    const rect = apply(rectOf, current.el, []);
    post('request', { kind: current.kind, role: current.role, x: Math.round(rect.left), y: Math.round(rect.top), w: Math.round(rect.width), h: Math.round(rect.height) });
  }

  function field(event) {
    const el = targetOf(event);
    return el instanceof HTMLInputElement || el instanceof HTMLSelectElement ? el : null;
  }
  on(document, 'focusin', event => {
    if (!trusted(event)) return;
    const el = field(event);
    const found = el && analyze(el);
    if (!found) { current = null; hide(); return; }
    current = { el, role: found.role, kind: found.kind };
    request();
  }, true);
  on(document, 'click', event => {
    if (!trusted(event)) return;
    const el = field(event);
    if (!el || ui || !current || current.el !== el) return;
    request();
  }, true);
  on(document, 'input', event => { if (trusted(event) && current && targetOf(event) === current.el) hide(); }, true);
  on(document, 'keydown', event => {
    if (!trusted(event) || !current || targetOf(event) !== current.el) return;
    const key = keyOf(event);
    if (key === 'Escape') { hide(); return; }
    if (key === 'ArrowDown' && !ui) { request(); cancel(event); return; }
    if (!ui || !ui.count) return;
    const move = key === 'ArrowDown' ? 1 : key === 'ArrowUp' ? -1 : 0;
    if (move) {
      cancel(event);
      ui.active = (ui.active + move + ui.count) % ui.count;
      highlight(ui.active);
    } else if (key === 'Enter' && ui.active >= 0) {
      cancel(event);
      const action = ui.choices[ui.active].action;
      hide();
      action();
    }
  }, true);
  on(document, 'pointerdown', event => {
    if (!ui) return;
    const path = apply(composedPath, event, []);
    for (let i = 0; i < path.length; i++) if (path[i] === ui.host) return;
    hide();
  }, true);
  on(window, 'blur', hide);
  on(window, 'resize', hide);
  on(window, 'scroll', hide, true);

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
    if (!current || !apply(isConnected, current.el, [])) return;
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
  on(document, 'submit', event => {
    if (!trusted(event)) return;
    const form = targetOf(event);
    if (!(form instanceof HTMLFormElement)) return;
    const found = capture(form);
    if (found) post('capture', { kind: found.kind, data: stringify(found.values), origin: location.origin });
  }, true);

  let banner = null;
  function closeBanner() { if (banner) apply(removeElement, banner.host, []); banner = null; }
  const BAR_STYLE = `
      :host { all: initial; }
      .bar { position: fixed; z-index: 2147483647; top: 12px; right: 12px; box-sizing: border-box; max-width: min(420px, calc(100vw - 24px));
        display: flex; flex-direction: column; gap: 10px; padding: 12px 14px; border-radius: 12px; border: 1px solid rgba(128,128,128,.35);
        background: #20232a; color: #f2f3f5; box-shadow: 0 8px 28px rgba(0,0,0,.35); font: 13px/1.4 system-ui, sans-serif; }
      @media (prefers-color-scheme: light) { .bar { background: #fff; color: #1d1f23; } }
      .row { display: flex; gap: 8px; justify-content: flex-end; }
      button { all: unset; padding: 6px 12px; border-radius: 8px; cursor: pointer; background: rgba(128,128,128,.22); }
      button.primary { background: #3b82f6; color: #fff; }
      button:focus-visible { outline: 2px solid #93c5fd; }`;
  /** A bar at the top of the page: "Save the password for example.com?". Real clicks only. */
  function prompt(message) {
    closeBanner();
    const { host, root } = shadowHost();
    const kind = '' + message.kind, mode = message.mode;
    const sentence = kind === 'login' ? (mode === 'update' ? text.updateLogin : text.saveLogin)
      : kind === 'card' ? text.saveCard : text.saveContact;
    const bar = element('div', { class: 'bar', role: 'alertdialog' });
    const row = element('div', { class: 'row' });
    const decide = (name, path, primary) => {
      const button = element('button', primary ? { type: 'button', class: 'primary' } : { type: 'button' }, name);
      on(button, 'click', event => { if (!trusted(event)) return; cancel(event); closeBanner(); post(path, { kind }); });
      return button;
    };
    apply(appendToElement, row, [decide(text.notNow, 'dismiss', false), decide(mode === 'update' ? text.update : text.save, 'save', true)]);
    apply(appendToElement, bar, [element('div', null, sentence('' + message.subject)), row]);
    apply(appendToFragment, root, [element('style', null, BAR_STYLE), bar]);
    apply(appendToElement, apply(rootElement, document, []), [host]);
    banner = blank(); banner.host = host;
  }
  // A page that follows a submit (the login redirects) tells the browser it is there, so a pending
  // "save this?" can be shown on it.
  const announce = () => post('ready', { kind: 'login' });
  if (document.readyState === 'loading') on(document, 'DOMContentLoaded', announce, { once: true }); else announce();

  // The browser reaches the page only through this one name, which is not writable and not guessable.
  define(window, entry, {
    value(message) {
      if (message.op === 'show') show(message);
      else if (message.op === 'fill') fill(message.kind, message.values);
      else if (message.op === 'prompt') prompt(message);
      else if (message.op === 'close-prompt') closeBanner();
    },
  });
})();
