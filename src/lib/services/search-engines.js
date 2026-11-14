/**
 * The search engines the address bar can use. Ids are stored in settings and backups, so they never
 * change. Keep this list in step with `src-tauri/src/search_engines.rs`; `tests/search-engines.test.mjs`
 * fails when the two drift apart. `color` only tints the picker's monogram.
 */
export const SEARCH_ENGINES = /** @type {const} */ ([
  { id: 'google', label: 'Google', home: 'https://www.google.com/', search: 'https://www.google.com/search', param: 'q', color: '#4285F4' },
  { id: 'duckduckgo', label: 'DuckDuckGo', home: 'https://duckduckgo.com/', search: 'https://duckduckgo.com/', param: 'q', color: '#DE5833' },
  { id: 'bing', label: 'Bing', home: 'https://www.bing.com/', search: 'https://www.bing.com/search', param: 'q', color: '#0C8484' },
  { id: 'brave', label: 'Brave Search', home: 'https://search.brave.com/', search: 'https://search.brave.com/search', param: 'q', color: '#FB542B' },
  { id: 'startpage', label: 'Startpage', home: 'https://www.startpage.com/', search: 'https://www.startpage.com/do/search', param: 'q', color: '#5B6CFF' },
  { id: 'ecosia', label: 'Ecosia', home: 'https://www.ecosia.org/', search: 'https://www.ecosia.org/search', param: 'q', color: '#2E9E6B' },
  { id: 'qwant', label: 'Qwant', home: 'https://www.qwant.com/', search: 'https://www.qwant.com/', param: 'q', color: '#4F7CFF' },
  { id: 'yahoo', label: 'Yahoo', home: 'https://search.yahoo.com/', search: 'https://search.yahoo.com/search', param: 'p', color: '#7B2FE0' },
  { id: 'mojeek', label: 'Mojeek', home: 'https://www.mojeek.com/', search: 'https://www.mojeek.com/search', param: 'q', color: '#4C9A2A' },
  { id: 'kagi', label: 'Kagi', home: 'https://kagi.com/', search: 'https://kagi.com/search', param: 'q', color: '#E0A100' },
]);

export const DEFAULT_SEARCH_ENGINE = 'google';

/** @param {unknown} id */
export function isSearchEngineId(id) {
  return SEARCH_ENGINES.some((engine) => engine.id === id);
}

/** Unknown ids fall back to the default engine, like the native side does. */
/** @param {unknown} id */
export function findSearchEngine(id) {
  return SEARCH_ENGINES.find((engine) => engine.id === id) ?? SEARCH_ENGINES[0];
}

/**
 * @param {unknown} id
 * @param {string} query
 */
export function buildSearchUrl(id, query) {
  const engine = findSearchEngine(id);
  const url = new URL(engine.search);
  url.searchParams.set(engine.param, query);
  return url.href;
}
