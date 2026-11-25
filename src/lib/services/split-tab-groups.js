/**
 * The workspace list remains the order source for both independent pane lists.
 * @param {import('../types/browser').TabItem[]} tabs
 * @param {import('../types/browser').SplitLayout | null} layout
 * @param {string} workspaceId
 */
export function focusedTabs(tabs, layout, workspaceId) {
  const workspace = tabs.filter(tab => !tab.workspace_id || tab.workspace_id === workspaceId);
  if (!layout || layout.workspace_id !== workspaceId) return workspace;
  const ids = layout.focused_tab_id === layout.right_tab_id ? layout.right_tab_ids : layout.left_tab_ids;
  return workspace.filter(tab => ids.includes(tab.id));
}

/** @param {{id: string}[]} tabs @param {string[]} ids */
export function replaceGroupOrder(tabs, ids) {
  const group = new Set(ids);
  let index = 0;
  return tabs.map(tab => group.has(tab.id) ? ids[index++] : tab.id);
}

/** @param {{x: number, width: number}} bounds @param {number} ratio */
export function splitHeaderGeometry(bounds, ratio) {
  const available = Math.max(200, bounds.width - 8);
  const left = Math.max(100, Math.min(available - 100, available * Math.max(.2, Math.min(.8, ratio))));
  return { left: bounds.x, leftWidth: left, right: bounds.x + left + 8, rightWidth: available - left };
}

/** The tab that opens beside the active one when a split starts: the other tab of the workspace used most recently
 *  (a strictly greater epoch wins, so ties keep the tab order, like the docking rule). Null when there is none.
 * @param {{id: string, workspace_id?: string, last_active_epoch?: number}[]} tabs @param {string} activeId @param {string} workspaceId */
export function pickSplitCompanion(tabs, activeId, workspaceId) {
  /** @type {typeof tabs[number] | null} */
  let best = null;
  for (const tab of tabs) {
    if (tab.id === activeId || (tab.workspace_id && tab.workspace_id !== workspaceId)) continue;
    if (!best || (tab.last_active_epoch ?? 0) > (best.last_active_epoch ?? 0)) best = tab;
  }
  return best ? best.id : null;
}
