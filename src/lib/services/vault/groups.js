/**
 * How the vault list is grouped by site, like the password pages of other browsers: every account of
 * one site sits under one row. Pure functions, so they can be tested without the interface.
 */

/**
 * The site an entry belongs to: the host of its address without "www.", or its title when it has none.
 * @param {{ title: string, url: string }} entry
 * @returns {string}
 */
export function siteOf(entry) {
  const url = (entry.url ?? '').trim();
  if (url) {
    try {
      const host = new URL(url.includes('://') ? url : `https://${url}`).hostname.replace(/^www\./, '').toLowerCase();
      if (host) return host;
    } catch {
      // Not an address: fall back to the title.
    }
  }
  return (entry.title ?? '').trim().toLowerCase() || '—';
}

/**
 * Groups entries by site, sites in alphabetical order and the accounts of a site by user name.
 * A search matches title, address and user name.
 * @template {{ title: string, url: string, username: string }} T
 * @param {T[]} entries
 * @param {string} [query]
 * @returns {{ site: string, entries: T[] }[]}
 */
export function groupEntries(entries, query = '') {
  const needle = query.trim().toLowerCase();
  /** @type {Map<string, T[]>} */
  const groups = new Map();
  for (const entry of entries) {
    if (needle && !`${entry.title} ${entry.url} ${entry.username}`.toLowerCase().includes(needle)) continue;
    const site = siteOf(entry);
    const list = groups.get(site);
    if (list) list.push(entry);
    else groups.set(site, [entry]);
  }
  return [...groups]
    .map(([site, list]) => ({ site, entries: list.sort((a, b) => (a.username || a.title).toLowerCase().localeCompare((b.username || b.title).toLowerCase())) }))
    .sort((a, b) => a.site.localeCompare(b.site));
}

/**
 * A stable colour for a site's letter, so the same site always looks the same.
 * @param {string} text
 * @returns {number} a hue from 0 to 359
 */
export function avatarHue(text) {
  let hash = 0;
  for (const char of text) hash = (hash * 31 + (char.codePointAt(0) ?? 0)) % 360;
  return hash;
}
