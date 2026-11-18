// Spark's own removal of YouTube advertising, written for Spark (Apache-2.0).
//
// A network list cannot separate a YouTube video ad from the video: both come from the same servers.
// What does separate them is the player's own description of the page, which lists the ad breaks
// (`adPlacements`, `playerAds`, `adSlots`). When the player never receives those lists, it has nothing
// to play. This runs at document start, before the page's scripts, and does three things:
//   1. removes those lists from the player data the page assigns or fetches;
//   2. hides the ad boxes of the home page, search results and watch page;
//   3. never touches anything that is not a YouTube page.
// It is part of the shield: it follows the shield switch and the site exceptions through `isOn`.
// The "Auto skip ads" switch is separate and only acts on ads that still reach the player.
const youtubeAds = (isOn) => {
  if (!/^(www\.|m\.)?youtube\.com$/i.test(location.hostname)) return undefined;
  const AD_LISTS = ['adPlacements', 'playerAds', 'adSlots', 'adBreakHeartbeatParams'];
  const prune = (value) => {
    if (!isOn() || !value || typeof value !== 'object') return value;
    try {
      // Player data is either the response itself or wrapped in `playerResponse` (watch and next calls).
      for (const target of [value, value.playerResponse]) {
        if (!target || typeof target !== 'object') continue;
        for (const key of AD_LISTS) if (key in target) delete target[key];
      }
    } catch { /* a frozen or exotic object is left as it is */ }
    return value;
  };

  // The first player data comes inline in the page as `var ytInitialPlayerResponse = {...}`.
  try {
    let initial;
    Object.defineProperty(window, 'ytInitialPlayerResponse', {
      configurable: true,
      enumerable: true,
      get: () => initial,
      set: (value) => { initial = prune(value); },
    });
  } catch { /* the page already defined it */ }

  // Later player data arrives by `fetch(...).then(r => r.json())` and by `JSON.parse`.
  try {
    JSON.parse = new Proxy(JSON.parse, {
      apply: (target, thisArg, args) => prune(Reflect.apply(target, thisArg, args)),
    });
  } catch { /* leave JSON alone */ }
  try {
    if (typeof Response === 'function') {
      Response.prototype.json = new Proxy(Response.prototype.json, {
        apply: (target, thisArg, args) => Reflect.apply(target, thisArg, args).then(prune),
      });
    }
  } catch { /* leave Response alone */ }

  // Ad boxes that are part of the page layout rather than of the player.
  const css = [
    '#masthead-ad', '#player-ads', '.video-ads.ytp-ad-module', 'ytd-ad-slot-renderer',
    'ytd-in-feed-ad-layout-renderer', 'ytd-display-ad-renderer', 'ytd-promoted-sparkles-web-renderer',
    'ytd-promoted-video-renderer', 'ytd-banner-promo-renderer', 'ytd-statement-banner-renderer',
    'ytd-companion-slot-renderer', 'ytd-action-companion-ad-renderer', 'ytd-player-legacy-desktop-watch-ads-renderer',
    'ytm-promoted-sparkles-web-renderer', 'ytm-companion-ad-renderer', '.ytd-merch-shelf-renderer',
  ].join(',') + '{display:none!important}';
  let sheet = null;
  const apply = () => {
    if (sheet) { sheet.disabled = !isOn(); return; }
    if (!isOn()) return;
    try {
      sheet = new CSSStyleSheet();
      sheet.replaceSync(css);
      document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
    } catch {
      sheet = null;
      const style = document.createElement('style');
      style.textContent = css;
      (document.head || document.documentElement).appendChild(style);
    }
  };
  apply();
  // The shield can be switched while the page is open.
  return apply;
};
