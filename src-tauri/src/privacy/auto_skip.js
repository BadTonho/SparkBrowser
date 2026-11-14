(() => {
  if (window.__sparkSetAutoSkip) return;
  const token = __AUTO_SKIP_TOKEN__;
  let enabled = false, observer = null, timer = 0, lastCheck = -Infinity;
  let clicked = new WeakSet(), videoKey = '';
  const supported = () => {
    try {
      const url = new URL(location.href);
      return window.top === window && url.protocol === 'https:'
        && ['youtube.com', 'www.youtube.com'].includes(url.hostname) && url.pathname === '/watch';
    } catch { return false; }
  };
  function check() {
    timer = 0; lastCheck = performance.now();
    if (!enabled || !supported()) return;
    const player = document.querySelector('#movie_player');
    const currentVideo = new URL(location.href).searchParams.get('v') || '';
    if (currentVideo !== videoKey) { clicked = new WeakSet(); videoKey = currentVideo; }
    if (!player?.classList.contains('ad-showing')) { clicked = new WeakSet(); return; }
    for (const button of player.querySelectorAll('button.ytp-ad-skip-button, button.ytp-skip-ad-button, button.ytp-ad-skip-button-modern')) {
      const style = getComputedStyle(button);
      if (clicked.has(button) || button.disabled || button.getAttribute('aria-disabled') === 'true'
        || button.closest('[hidden],[inert]') || !button.getClientRects().length
        || style.display === 'none' || style.visibility !== 'visible' || Number(style.opacity) === 0) continue;
      clicked.add(button);
      button.click();
      break;
    }
  }
  function schedule() {
    if (enabled && !timer) timer = setTimeout(check, Math.max(0, 500 - (performance.now() - lastCheck)));
  }
  function stop() {
    observer?.disconnect(); observer = null;
    clearTimeout(timer); timer = 0;
    document.removeEventListener('yt-navigate-finish', schedule);
    window.removeEventListener('popstate', schedule);
  }
  Object.defineProperty(window, '__sparkSetAutoSkip', { writable: false, configurable: false,
    value(secret, value) {
      if (secret !== token || typeof value !== 'boolean') return;
      stop(); enabled = value;
      if (!enabled) return;
      if (!['youtube.com', 'www.youtube.com'].includes(location.hostname) || location.protocol !== 'https:' || window.top !== window) return;
      observer = new MutationObserver(schedule);
      observer.observe(document, { childList: true, subtree: true, attributes: true,
        attributeFilter: ['class', 'disabled', 'aria-disabled', 'hidden', 'style'] });
      document.addEventListener('yt-navigate-finish', schedule);
      window.addEventListener('popstate', schedule);
      schedule();
    }
  });
  window.addEventListener('pagehide', () => { stop(); });
  window.addEventListener('pageshow', () => { if (enabled) window.__sparkSetAutoSkip(token, true); });
})();
