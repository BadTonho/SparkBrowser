import { writable, derived, readonly } from "svelte/store";
import { NEW_TAB_URL } from "../../services/new-tab";
import { tr } from "../../i18n";
import type { TabItem } from "../../types/browser";

/**
 * Reactive state for tabs and active URL.
 */
const mutableTabs = writable<TabItem[]>([
  {
    id: "tab-1",
    title: tr("newtab.title"),
    url: NEW_TAB_URL,
    workspace_id: "ws-personal",
    is_active: true,
    is_hibernated: false,
  },
]);

const previewOrder = writable<string[] | null>(null);
export const confirmedTabs = readonly(mutableTabs);
export const tabs = derived([mutableTabs, previewOrder], ([items, ids]) => {
  if (!ids) return items;
  const byId = new Map(items.map(tab => [tab.id, tab]));
  return [...new Set([...ids, ...items.map(tab => tab.id)])]
    .flatMap(id => { const tab = byId.get(id); return tab ? [tab] : []; });
});
export const previewTabOrder = previewOrder.set;
export const activeNavigation = derived(tabs, items => items.find(tab => tab.is_active)?.navigation);
export const isLoading = derived(activeNavigation, navigation => navigation?.loading ?? false);
// Internal mutation ports; consumers use the public facade's actions.
export const replaceTabs = (items: TabItem[]) => mutableTabs.set(items);
export const updateTabs = (change: (items: TabItem[]) => TabItem[]) => mutableTabs.update(change);
export { currentUrl, addressInput, isAddressEditing, setConfirmedUrl } from "./address";
export { isSidebarOpen, sidebarWidth, setSidebarOpen, SIDEBAR_EXPANDED_WIDTH, SIDEBAR_COLLAPSED_WIDTH } from "./presentation";
