(() => {
  // Browser shortcuts, links opened in a new tab and pane focus, for the top frame of a page.
  //
  // This script shares its JavaScript world with the page, which can replace any global or prototype
  // method. Everything it uses with the channel token, and every handler that sends a message, is
  // therefore taken now, before any page script runs, and called through `apply`: a page that later
  // replaces `addEventListener`, `setTimeout`, event getters or `URLSearchParams` can neither read the
  // token nor call a handler with a fake event.
  if (window.top !== window) return;
  const bridgeToken = __KEYBOARD_TOKEN__;
  const nativeAltCapture = __NATIVE_ALT_CAPTURE__;
  const nativeCaretBrowsingToggle = __NATIVE_CARET_TOGGLE__;

  const apply = Reflect.apply;
  const describe = Object.getOwnPropertyDescriptor;
  const define = Object.defineProperty;
  const entriesOf = Object.entries;
  const isArray = Array.isArray;
  const encode = encodeURIComponent;
  const later = setTimeout;
  const send = window.__sparkPostContent;
  const getter = (proto, name) => describe(proto, name).get;
  const listen = EventTarget.prototype.addEventListener;
  const on = (target, type, handler, options) => apply(listen, target, [type, handler, options]);
  const read = (proto, name) => { const get = getter(proto, name); return event => apply(get, event, []); };
  const targetOf = read(Event.prototype, 'target');
  const prevented = read(Event.prototype, 'defaultPrevented');
  const keyOf = read(KeyboardEvent.prototype, 'key');
  const repeatOf = read(KeyboardEvent.prototype, 'repeat');
  const modifiers = proto => ({
    ctrl: read(proto, 'ctrlKey'), alt: read(proto, 'altKey'), shift: read(proto, 'shiftKey'), meta: read(proto, 'metaKey'),
  });
  const keyboardModifiers = modifiers(KeyboardEvent.prototype);
  const mouseModifiers = modifiers(MouseEvent.prototype);
  const buttonOf = read(MouseEvent.prototype, 'button');
  const preventDefault = Event.prototype.preventDefault;
  const stopImmediate = Event.prototype.stopImmediatePropagation;
  const closest = Element.prototype.closest;
  const hasAttribute = Element.prototype.hasAttribute;
  const hrefOf = read(HTMLAnchorElement.prototype, 'href');
  const protocolOf = read(HTMLAnchorElement.prototype, 'protocol');
  const test = RegExp.prototype.test;
  const upper = String.prototype.toUpperCase;
  const DIGIT = /^[1-9]$/;
  const WEB = /^https?:$/;

  /** Only an event the browser created for a real action of the person. */
  function trusted(event) {
    try { targetOf(event); } catch { return false; }
    return event.isTrusted === true;
  }
  const stop = event => { apply(preventDefault, event, []); apply(stopImmediate, event, []); };

  const nonce = new Uint32Array(4); crypto.getRandomValues(nonce);
  const documentId = Array.from(nonce, n => n.toString(16).padStart(8, '0')).join('');
  window.__sparkWorkspaceSwitcherOpen = false;
  let shortcuts = {};
  let holdPressed = false;
  let nextSequence = 0;
  let requestInFlight = false;
  // A small linked list: arrays would go through Array.prototype, which the page controls.
  let head = null, tail = null;

  function flushRequestQueue() {
    if (requestInFlight || !head || typeof send !== 'function') return;
    requestInFlight = true;
    const request = head;
    let query = 'token=' + encode(bridgeToken) + '&document=' + encode(documentId) + '&sequence=' + request.sequence;
    if (request.action) query += '&action=' + encode(request.action);
    if (request.key) query += '&' + (request.type === 'link' ? 'input' : 'key') + '=' + encode(request.key);
    send('spark-internal://keyboard-shortcut/' + request.type + '?' + query);
  }

  function notifyBrowser(type, action, key) {
    nextSequence += 1;
    const request = { type, action: action === undefined ? '' : '' + action, key: key === undefined ? '' : '' + key, sequence: nextSequence, next: null };
    if (tail) tail.next = request; else head = request;
    tail = request;
    flushRequestQueue();
  }

  function normalizeKey(key) {
    return key.length === 1 ? apply(upper, key, []) : key;
  }

  function modifierMatch(event, binding, held) {
    return held.ctrl(event) === binding.ctrl && held.alt(event) === binding.alt &&
      (binding.key === '+' && !binding.shift || held.shift(event) === binding.shift) && held.meta(event) === binding.meta;
  }

  function isHoldKey(event, binding) {
    return binding && keyOf(event) === binding.key;
  }

  function resetHold(type) {
    if (!holdPressed) return;
    holdPressed = false;
    notifyBrowser(type);
  }

  function holdBinding() {
    const bindings = shortcuts['workspace-picker'];
    return isArray(bindings) ? bindings[0] : undefined;
  }

  define(window, '__sparkSetKeyboardShortcuts', {
    configurable: false,
    enumerable: false,
    writable: false,
    value: (secret, value) => { if (secret === bridgeToken) shortcuts = value || {}; }
  });

  define(window, '__sparkWorkspaceShortcutAck', {
    configurable: false,
    enumerable: false,
    writable: false,
    value: (sequence) => {
      if (!requestInFlight || !head || head.sequence !== sequence) return;
      head = head.next;
      if (!head) tail = null;
      requestInFlight = false;
      flushRequestQueue();
    }
  });

  on(window, 'keydown', (event) => {
    if (!trusted(event)) return;
    const key = keyOf(event);

    if (key === 'Escape' && document.fullscreenElement) {
      apply(preventDefault, event, []);
      void document.exitFullscreen();
      return;
    }

    if (nativeCaretBrowsingToggle && key === 'F7' && !keyboardModifiers.ctrl(event) && !keyboardModifiers.alt(event) && !keyboardModifiers.shift(event) && !keyboardModifiers.meta(event)) {
      stop(event);
      if (!repeatOf(event)) notifyBrowser('action', 'toggle-caret-browsing');
      return;
    }

    if (key === 'Escape' && window.__sparkWorkspaceSwitcherOpen) {
      stop(event);
      notifyBrowser('alt-cancel');
      return;
    }

    const hold = holdBinding();
    if (isHoldKey(event, hold)) {
      if (nativeAltCapture && hold.key === 'Alt') return;
      if (!repeatOf(event) && !holdPressed) {
        holdPressed = true;
        apply(preventDefault, event, []);
        notifyBrowser('alt-down');
      }
      return;
    }

    if (holdPressed) resetHold('alt-up');

    if (repeatOf(event)) return;
    const entries = entriesOf(shortcuts);
    for (let i = 0; i < entries.length; i++) {
      const action = entries[i][0], bindings = entries[i][1];
      if (action === 'workspace-picker' || !isArray(bindings)) continue;
      const binding = bindings[0];
      if (!binding || !modifierMatch(event, binding, keyboardModifiers)) continue;
      if (
        nativeAltCapture && action === 'switch-workspaces' &&
        binding.key === 'Digit' && binding.alt && !binding.ctrl && !binding.shift && !binding.meta
      ) return;
      const keyMatches = binding.key === 'Digit'
        ? apply(test, DIGIT, [key])
        : normalizeKey(key) === normalizeKey(binding.key) || (binding.key === '+' && key === '=');
      if (!keyMatches) continue;
      stop(event);
      notifyBrowser('action', action, key);
      return;
    }
  }, true);

  on(window, 'keyup', (event) => {
    if (!trusted(event)) return;
    const hold = holdBinding();
    if (!isHoldKey(event, hold)) return;
    if (nativeAltCapture && hold.key === 'Alt') return;
    resetHold('alt-up');
  }, true);

  const sideButton = event => { const button = buttonOf(event); return button === 3 || button === 4; };

  on(window, 'mousedown', (event) => {
    if (!trusted(event) || !sideButton(event)) return;
    stop(event);
    const mouseKey = buttonOf(event) === 3 ? 'Mouse4' : 'Mouse5';
    const entries = entriesOf(shortcuts);
    for (let i = 0; i < entries.length; i++) {
      const action = entries[i][0], bindings = entries[i][1];
      if (!isArray(bindings)) continue;
      const binding = bindings[0];
      if (!binding || binding.key !== mouseKey || !modifierMatch(event, binding, mouseModifiers)) continue;
      notifyBrowser('action', action);
      return;
    }
  }, true);

  // Chromium performs its own back/forward when a side button is released, on top of the
  // shortcut handled on mousedown, so the page moved more than one entry. Cancel the native one.
  const cancelSideButton = (event) => { if (trusted(event) && sideButton(event)) stop(event); };
  on(window, 'mouseup', cancelSideButton, true);
  on(window, 'pointerup', cancelSideButton, true);

  /** The web address of the link under a real click, or null. */
  function linkOf(event) {
    const target = targetOf(event);
    const anchor = target instanceof Element ? apply(closest, target, ['a[href]']) : null;
    if (!anchor || apply(hasAttribute, anchor, ['download'])) return null;
    const href = hrefOf(anchor);
    return href.length <= 1500 && apply(test, WEB, [protocolOf(anchor)]) ? href : null;
  }

  on(window, 'auxclick', (event) => {
    if (!trusted(event)) return;
    if (buttonOf(event) === 1 && !prevented(event)) {
      const href = linkOf(event);
      if (href) { apply(preventDefault, event, []); later(() => notifyBrowser('link', undefined, href), 0); return; }
    }
    if (sideButton(event)) stop(event);
  }, true);

  on(window, 'click', (event) => {
    if (!trusted(event) || prevented(event) || buttonOf(event) !== 0 || !(mouseModifiers.ctrl(event) || mouseModifiers.meta(event)) || mouseModifiers.shift(event) || mouseModifiers.alt(event)) return;
    const href = linkOf(event);
    if (href) { apply(preventDefault, event, []); later(() => notifyBrowser('link', undefined, href), 0); }
  });

  on(window, 'focus', () => {
    if (!nativeAltCapture && typeof send === 'function') send('spark-internal://pane-focus?token=' + encode(bridgeToken));
    holdPressed = false;
  }, true);
  on(document, 'visibilitychange', () => {
    if (document.visibilityState === 'hidden') resetHold('alt-cancel');
    else holdPressed = false;
  }, true);
})();
