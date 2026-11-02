/** @param {string} value */
export const normalizeSearch = value => value.normalize('NFD').replace(/\p{Diacritic}/gu, '').toLowerCase();
/** @param {string} id */
const category = id => id.startsWith('note-') ? 'notes' : id.startsWith('session-') ? 'sessions' : id.startsWith('switch-') ? 'tabs' : id.startsWith('ws-') ? 'workspaces' : id.startsWith('bm-') ? 'bookmarks' : id.startsWith('cmd-') ? 'commands' : 'navigation';
/** @param {import('../types/browser').CommandItem[]} items @param {string} query */
export function rankCommands(items, query) {
  const q = normalizeSearch(query.trim());
  const score = (/** @type {import('../types/browser').CommandItem} */ item) => {
    if (category(item.id) === 'navigation') return q ? 5 : 0;
    const title = normalizeSearch(item.title);
    return !q ? 10 : title === q ? 100 : title.startsWith(q) ? 80 : title.includes(q) ? 60 : ['notes', 'sessions'].includes(category(item.id)) ? 50 : normalizeSearch(item.subtitle).includes(q) ? 40 : -1;
  };
  const counts = new Map();
  const ranked = items.map((item, index) => ({ item, index, score: score(item) }))
    .filter(item => item.score >= 0).sort((a, b) => b.score - a.score || a.index - b.index)
    .filter(({ item }) => {
      const key = category(item.id);
      const count = counts.get(key) ?? 0;
      counts.set(key, count + 1);
      return count < (key === 'tabs' ? 3 : 2);
    }).map(({ item }) => item);
  const navigation = q ? ranked.filter(item => category(item.id) === 'navigation').slice(0, 2) : [];
  return [...ranked.filter(item => category(item.id) !== 'navigation').slice(0, 12 - navigation.length), ...navigation];
}
