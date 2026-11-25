(() => {
  if (window.top !== window) return;
  // The messages carry the secret of a page script, and this script shares its JavaScript world with
  // the page. Everything it relies on is therefore taken now, before any page script runs: a page that
  // later replaces a global, a prototype method or the native function cannot see what is sent.
  const engine = __CONTENT_NATIVE__;
  const bindingName = __CONTENT_BINDING__;
  const stringify = JSON.stringify;
  const describe = Object.getOwnPropertyDescriptor;
  const remove = Reflect.deleteProperty;
  const apply = Reflect.apply;
  const later = setTimeout;
  const MAX_QUEUED = 64;
  // WebKitGTK: the namespace of script message handlers, read before the page can replace it.
  const handlers = engine === 'linux' && window.webkit ? window.webkit.messageHandlers : null;

  function find() {
    if (engine === 'windows') {
      // Only an own property, so nothing on the prototype chain (which the page can change) is read.
      // The name is random per tab, so the page cannot have planted a function under it.
      const binding = describe(window, bindingName);
      if (!binding || typeof binding.value !== 'function') return null;
      // Once taken, the page cannot call it at all.
      remove(window, bindingName);
      const signal = binding.value;
      return message => { signal(message); };
    }
    if (engine === 'linux') {
      // Normally registered before the page starts; if not, this is retried while messages wait.
      const handler = handlers && handlers.sparkContent;
      const post = handler && handler.postMessage;
      if (typeof post !== 'function') return null;
      return message => { apply(post, handler, [message]); };
    }
    return null;
  }

  let deliver = find();
  // A small linked list: arrays would go through Array.prototype, which the page controls.
  let head = null, tail = null, queued = 0, timer = 0, attempts = 0;
  function drain() {
    timer = 0;
    if (!deliver) deliver = find();
    if (deliver) {
      for (let node = head; node; node = node.next) deliver(node.message);
      head = tail = null; queued = 0; attempts = 0;
    } else if (++attempts < 200) timer = later(drain, 10);
    else { head = tail = null; queued = 0; attempts = 0; }
  }

  Object.defineProperty(window, '__sparkPostContent', { value(url) {
    if (engine === 'none') { window.location.href = url; return; }
    // `url` is a string from Spark's scripts; serializing a string runs no page code.
    const message = '{"sparkContent":' + stringify('' + url) + '}';
    if (!deliver) deliver = find();
    if (deliver) { deliver(message); return; }
    // The native function is not installed yet: wait for it. Never fall back to navigating to the
    // message, which the page could read through the Navigation API.
    const node = { message, next: null };
    if (queued === MAX_QUEUED) { head = head.next; queued -= 1; if (!head) tail = null; }
    if (tail) tail.next = node; else head = node;
    tail = node; queued += 1;
    if (!timer) timer = later(drain, 10);
  } });
})();
