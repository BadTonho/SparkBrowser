(() => {
  if (window.top !== window) return;
  const native = __CONTENT_NATIVE__;
  const pending = [];
  let timer = 0, attempts = 0;
  function drain() {
    timer = 0;
    if (typeof window.__sparkContentSignal === 'function') {
      for (const url of pending.splice(0)) window.__sparkContentSignal(JSON.stringify({ sparkContent: url }));
      attempts = 0;
    } else if (++attempts < 200) timer = setTimeout(drain, 10);
    else {
      // Retain the legacy transport if this runtime cannot install the binding.
      const latest = pending.pop(); pending.length = 0; attempts = 0;
      if (latest) window.location.href = latest;
    }
  }
  Object.defineProperty(window, '__sparkPostContent', { value(url) {
    if (!native) { window.location.href = url; return; }
    if (pending.length === 64) pending.shift();
    pending.push(url);
    if (!timer) drain();
  } });
})();
