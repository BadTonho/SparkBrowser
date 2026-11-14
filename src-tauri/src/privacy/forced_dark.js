(() => {
  if (window.__sparkSetForcedDark || window.top !== window) return;
  const token = __FORCED_DARK_TOKEN__;
  const CLASS = '__spark-forced-dark';
  const STYLE_ID = '__spark-forced-dark-style';
  // Dark pages are left alone: the filter only makes sense on a page that is actually light.
  const LIGHT_LUMINANCE = 0.6;
  let enabled = __FORCED_DARK_INITIAL__, applied = false, observer = null, timer = 0;

  function channels(color) {
    const match = /rgba?\(([^)]+)\)/.exec(color);
    if (!match) return null;
    const [r, g, b, a = 1] = match[1].split(/[ ,/]+/).filter(Boolean).map(Number);
    return [r, g, b, a];
  }
  function luminance([r, g, b]) {
    const lin = value => { const v = value / 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; };
    return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
  }
  /** The colour the user actually sees behind the content: the first opaque background from body outwards. */
  function pageBackground() {
    for (const node of [document.body, document.documentElement]) {
      if (!node) continue;
      const parsed = channels(getComputedStyle(node).backgroundColor);
      if (parsed && parsed[3] > 0.5) return parsed;
    }
    // Nothing painted: the browser canvas, which is dark only when the page opted into a dark colour scheme.
    const scheme = getComputedStyle(document.documentElement).colorScheme || '';
    return /\bdark\b/.test(scheme) && !/\blight\b/.test(scheme) ? [0, 0, 0, 1] : [255, 255, 255, 1];
  }
  function pageIsLight() {
    if (!document.body) return false;
    return luminance(pageBackground()) > LIGHT_LUMINANCE;
  }
  function install() {
    if (document.getElementById(STYLE_ID)) return;
    const style = document.createElement('style');
    style.id = STYLE_ID;
    // Inverting the whole page darkens it; media is inverted back so photos and video keep their colours.
    style.textContent = `
      html.${CLASS} { background: #fff !important; filter: invert(1) hue-rotate(180deg) !important; }
      html.${CLASS} :is(img, video, picture, canvas, embed, object, iframe) { filter: invert(1) hue-rotate(180deg) !important; }
      html.${CLASS} { color-scheme: dark; }
    `;
    (document.head || document.documentElement).append(style);
  }
  function evaluate() {
    timer = 0;
    const root = document.documentElement;
    if (!root) return;
    if (!enabled) { root.classList.remove(CLASS); applied = false; return; }
    // The filter does not change computed colours, so the page can be re-measured while it is applied.
    const light = pageIsLight();
    if (light && !applied) { install(); root.classList.add(CLASS); applied = true; }
    else if (!light && applied) { root.classList.remove(CLASS); applied = false; }
  }
  function schedule() { if (!timer) timer = setTimeout(evaluate, 120); }
  function watch() {
    observer?.disconnect(); observer = null;
    if (!enabled) return;
    observer = new MutationObserver(schedule);
    // Themes are switched through attributes on the root and body, so those are all that needs watching.
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class', 'style', 'data-theme', 'data-color-mode'] });
    if (document.body) observer.observe(document.body, { attributes: true, attributeFilter: ['class', 'style'] });
  }
  Object.defineProperty(window, '__sparkSetForcedDark', { writable: false, configurable: false,
    value(secret, value) {
      if (secret !== token || typeof value !== 'boolean') return;
      enabled = value;
      watch();
      if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', evaluate, { once: true });
      else evaluate();
    }
  });
  window.addEventListener('load', schedule);
  window.addEventListener('pageshow', schedule);
  // While the page is still loading its stylesheets the first reading can be wrong.
  document.addEventListener('DOMContentLoaded', () => { watch(); schedule(); }, { once: true });
})();
