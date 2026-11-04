(() => {
  if (window.top !== window) return;
  const token = __ACTIVITY_TOKEN__;
  const nonce = new Uint32Array(4); crypto.getRandomValues(nonce);
  const documentId = Array.from(nonce, n => n.toString(16).padStart(8, '0')).join('');
  let sequence = 0;
  let dirty = false;
  let last = '';
  let scheduled = false;
  function report() {
    scheduled = false;
    const media = [...document.querySelectorAll('audio,video')].some(el => !el.paused && !el.ended);
    const key = `${dirty}:${media}`;
    if (key === last) return;
    last = key;
    const query = new URLSearchParams({ token, document: documentId, sequence: String(++sequence), dirty: String(dirty), media: String(media) });
    window.location.href = `veloce-internal://page-activity/state?${query}`;
  }
  function schedule() {
    if (scheduled || document.readyState === 'loading') return;
    scheduled = true;
    setTimeout(report, 250);
  }
  document.addEventListener('input', event => {
    if (event.target instanceof Element && event.target.matches('input,textarea,select,[contenteditable], [contenteditable] *')) { dirty = true; schedule(); }
  }, true);
  document.addEventListener('change', event => {
    if (event.target instanceof Element && event.target.matches('input,textarea,select')) { dirty = true; schedule(); }
  }, true);
  document.addEventListener('reset', () => { dirty = false; schedule(); }, true);
  for (const event of ['play', 'pause', 'ended', 'loadedmetadata']) document.addEventListener(event, schedule, true);
  document.addEventListener('DOMContentLoaded', schedule, { once: true });
})();
