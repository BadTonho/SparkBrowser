/** @param {import('../types/browser').TabItem} tab */
export function protectedTab(tab) {
  return Boolean(tab.is_visible || tab.is_active || tab.is_pinned || tab.navigation?.loading || tab.activity?.dirty_form || tab.activity?.playing_media || tab.activity?.downloading);
}
/** @param {string} raw */
export function tabUrlKey(raw) { try { return new URL(raw).href; } catch { return raw; } }
/** @param {import('../types/browser').TabItem[]} tabs */
export function duplicateGroups(tabs) {
  /** @type {Map<string,import('../types/browser').TabItem[]>} */ const groups = new Map();
  for (const tab of tabs) { const key = tabUrlKey(tab.url); const group = groups.get(key) ?? []; group.push(tab); groups.set(key, group); }
  return [...groups].filter(([, items]) => items.length > 1);
}
/** @param {import('../types/browser').TabItem[]} tabs */
export function duplicateCandidates(tabs) {
  return duplicateGroups(tabs).flatMap(([, items]) => {
    const ordered = [...items].sort((a, b) => Number(b.is_active) - Number(a.is_active) || (b.last_active_epoch ?? 0) - (a.last_active_epoch ?? 0) || a.id.localeCompare(b.id));
    return ordered.slice(1).filter(tab => !protectedTab(tab)).map(tab => tab.id);
  });
}
