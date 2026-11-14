import { buildSearchUrl } from './search-engines.js';
/** @param {{keyword:string,name:string,template:string}} item */
export function validateSiteSearch(item) {
  if (!/^[a-z0-9_-]{1,24}$/i.test(item.keyword)) throw new Error('Keyword must contain 1–24 letters, digits, _ or -');
  if (!item.name.trim() || item.name.length > 100) throw new Error('Name must contain 1–100 characters');
  if (item.template.length > 4096 || item.template.split('{query}').length !== 2) throw new Error('Template must contain exactly one {query}');
  const url = new URL(item.template.replace('{query}', 'VELOCEQUERYMARKER'));
  if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password || item.template.split('{query}')[0].includes('#') || url.hostname.includes('velocequerymarker') || !(url.pathname.includes('VELOCEQUERYMARKER') || url.search.includes('VELOCEQUERYMARKER'))) throw new Error('Use an HTTP/HTTPS template with {query} in its path or query');
}
/** @param {string} input @param {{keyword:string,template:string}[]} shortcuts @param {string} engine */
export function resolvePreviewInput(input, shortcuts, engine = 'google') {
  const trimmed = input.trim();
  const match = /^(\S+)\s+(.+)$/s.exec(trimmed);
  const shortcut = match && shortcuts.find(item => item.keyword.toLowerCase() === match[1].toLowerCase());
  if (shortcut && match) {
    const encoded = encodeURIComponent(match[2].trim()).replace(/[!'()~]/g, character => '%' + character.charCodeAt(0).toString(16).toUpperCase());
    return new URL(shortcut.template.replace('{query}', encoded)).href;
  }
  if (/^https?:\/\//.test(trimmed)) return new URL(trimmed).href;
  if (!/\s/.test(trimmed) && trimmed.includes('.')) return new URL('https://' + trimmed).href;
  return buildSearchUrl(engine, trimmed);
}
