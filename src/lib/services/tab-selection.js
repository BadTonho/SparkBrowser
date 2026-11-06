/**
 * Resolve a Ctrl/Meta/Shift selection for an ordered list of tabs.
 * Plain activation clears the selection; the caller still performs activation.
 * @param {string[]} selectedIds
 * @param {string|null} anchorId
 * @param {string[]} orderedIds
 * @param {string} targetId
 * @param {{ctrlKey?:boolean,metaKey?:boolean,shiftKey?:boolean}} modifiers
 */
export function resolveTabSelection(selectedIds, anchorId, orderedIds, targetId, modifiers) {
  const targetIndex = orderedIds.indexOf(targetId);
  if (targetIndex < 0) return { selectedIds: [], anchorId: null };

  const additive = Boolean(modifiers.ctrlKey || modifiers.metaKey);
  if (modifiers.shiftKey) {
    const anchorIndex = anchorId ? orderedIds.indexOf(anchorId) : -1;
    const start = anchorIndex < 0 ? targetIndex : Math.min(anchorIndex, targetIndex);
    const end = anchorIndex < 0 ? targetIndex : Math.max(anchorIndex, targetIndex);
    const range = orderedIds.slice(start, end + 1);
    const next = additive ? new Set([...selectedIds, ...range]) : new Set(range);
    return {
      selectedIds: orderedIds.filter(id => next.has(id)),
      anchorId: anchorIndex < 0 ? targetId : anchorId,
    };
  }

  if (additive) {
    const next = new Set(selectedIds);
    if (next.has(targetId)) next.delete(targetId);
    else next.add(targetId);
    return {
      selectedIds: orderedIds.filter(id => next.has(id)),
      anchorId: targetId,
    };
  }

  return { selectedIds: [], anchorId: null };
}
