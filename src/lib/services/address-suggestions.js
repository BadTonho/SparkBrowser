/**
 * Suggestions for the address bar: what Enter would do first, then bookmarks and visited pages that match,
 * then terms the search engine proposes. Pure functions, so the ranking is easy to test.
 */

/** @typedef {{ id: number, title: string, url: string, visited_at: number }} HistoryRow */
/** @typedef {{ title: string, url: string }} BookmarkRow */
/**
 * @typedef {object} Suggestion
 * @property {string} id Stable key for the list.
 * @property {'navigate'|'search'|'bookmark'|'history'|'web'} kind
 * @property {string} primary Main line (a title, or the text typed).
 * @property {string} secondary Second line (an address), may be empty.
 * @property {string} target What to send to the address bar's navigation when chosen.
 */

export const MAX_SUGGESTIONS = 8;

/**
 * Search engines that publish a suggestion endpoint. Keep in step with `endpoint()` in
 * `src-tauri/src/commands/search_suggestions.rs`.
 */
export const WEB_SUGGESTION_ENGINES = ['google', 'duckduckgo', 'bing', 'brave'];

/** @param {unknown} engine @returns {boolean} */
export function supportsWebSuggestions(engine) {
  return WEB_SUGGESTION_ENGINES.includes(/** @type {string} */ (engine));
}
const MAX_WEB_TERMS = 4;

/**
 * Address without scheme, `www.` or a trailing slash, so the same page is never listed twice.
 * @param {unknown} address
 * @returns {string}
 */
export function normalizeAddress(address) {
  return String(address ?? '')
    .trim()
    .toLowerCase()
    .replace(/^[a-z][a-z0-9+.-]*:\/\//, '')
    .replace(/^www\./, '')
    .replace(/\/+$/, '');
}

/**
 * True when the text reads as an address, not as words to search for.
 * @param {unknown} text
 * @returns {boolean}
 */
export function looksLikeAddress(text) {
  const value = String(text ?? '').trim();
  if (!value || /\s/.test(value)) return false;
  if (/^[a-z][a-z0-9+.-]*:\/\/\S+$/i.test(value)) return true;
  if (/^(localhost|\d{1,3}(\.\d{1,3}){3})(:\d+)?(\/.*)?$/i.test(value)) return true;
  return /^[^\s/]+\.[a-z]{2,}(:\d+)?(\/.*)?$/i.test(value);
}

/** @param {unknown} query @returns {string[]} */
function tokens(query) {
  return String(query ?? '').toLowerCase().split(/\s+/).filter(Boolean);
}

/** @param {string[]} words @param {...string} fields @returns {boolean} */
function matches(words, ...fields) {
  const haystack = fields.join(' ').toLowerCase();
  return words.every(word => haystack.includes(word));
}

/**
 * How well a page fits the query: starting with it counts most, then repeat visits, then recency.
 * @param {string[]} words @param {HistoryRow} row @param {number} visits @param {number} now
 * @returns {number}
 */
function score(words, row, visits, now) {
  const address = normalizeAddress(row.url);
  const title = String(row.title ?? '').toLowerCase();
  let value = Math.min(visits, 20);
  if (words.length && (address.startsWith(words[0]) || address.split('/')[0].startsWith(words[0]))) value += 40;
  if (words.length && title.startsWith(words[0])) value += 20;
  const ageDays = Math.max(0, (now - Number(row.visited_at ?? 0)) / 86400);
  value += Math.max(0, 14 - ageDays) * 0.5;
  return value;
}

/**
 * History rows collapsed to one per address, with how often each was visited.
 * @param {HistoryRow[] | undefined} rows
 * @returns {{ row: HistoryRow, visits: number }[]}
 */
export function groupHistory(rows) {
  /** @type {Map<string, { row: HistoryRow, visits: number }>} */
  const byAddress = new Map();
  for (const row of rows ?? []) {
    const key = normalizeAddress(row.url);
    if (!key) continue;
    const known = byAddress.get(key);
    if (known) {
      known.visits += 1;
      if (row.visited_at > known.row.visited_at) known.row = row;
    } else {
      byAddress.set(key, { row, visits: 1 });
    }
  }
  return [...byAddress.values()];
}

/**
 * @param {object} input
 * @param {string} input.query What is typed in the address bar (empty when it was just focused).
 * @param {HistoryRow[]} input.history Visits to consider, newest first.
 * @param {BookmarkRow[]} input.bookmarks
 * @param {string[]} [input.webTerms] Terms proposed by the search engine, when that option is on.
 * @param {string} [input.engineLabel] Name of the search engine, for the first line.
 * @param {number} [input.now] Seconds since the epoch (for tests).
 * @param {number} [input.limit]
 * @returns {Suggestion[]}
 */
export function buildSuggestions({ query, history, bookmarks, webTerms = [], engineLabel = '', now = Date.now() / 1000, limit = MAX_SUGGESTIONS }) {
  const text = String(query ?? '').trim();
  const words = tokens(text);
  const seen = new Set();
  /** @type {Suggestion[]} */
  const list = [];
  /** @param {Suggestion} item */
  const push = item => {
    const key = `${item.kind === 'web' || item.kind === 'search' ? 'q' : 'u'}:${normalizeAddress(item.target)}`;
    if (seen.has(key) || list.length >= limit) return;
    seen.add(key);
    list.push(item);
  };

  if (text) {
    if (looksLikeAddress(text)) {
      push({ id: `go:${text}`, kind: 'navigate', primary: text, secondary: '', target: text });
    } else {
      push({ id: `q:${text}`, kind: 'search', primary: text, secondary: engineLabel, target: text });
    }
  }

  const grouped = groupHistory(history);
  if (!text) {
    // Just focused: the sites visited most recently, as the page the user is likely to come back to.
    for (const { row } of grouped.slice(0, limit)) {
      push({ id: `h:${row.id}`, kind: 'history', primary: row.title || row.url, secondary: row.url, target: row.url });
    }
    return list;
  }

  for (const bookmark of bookmarks ?? []) {
    if (matches(words, bookmark.title ?? '', bookmark.url ?? '')) {
      push({ id: `b:${bookmark.url}`, kind: 'bookmark', primary: bookmark.title || bookmark.url, secondary: bookmark.url, target: bookmark.url });
    }
  }
  const pages = grouped
    .filter(({ row }) => matches(words, row.title ?? '', row.url ?? ''))
    .map(entry => ({ ...entry, value: score(words, entry.row, entry.visits, now) }))
    .sort((a, b) => b.value - a.value);
  for (const { row } of pages) {
    push({ id: `h:${row.id}`, kind: 'history', primary: row.title || row.url, secondary: row.url, target: row.url });
  }

  for (const term of (webTerms ?? []).slice(0, MAX_WEB_TERMS)) {
    if (typeof term === 'string' && term.trim()) push({ id: `w:${term}`, kind: 'web', primary: term.trim(), secondary: '', target: term.trim() });
  }
  return list;
}
