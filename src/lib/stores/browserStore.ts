import { NEW_TAB_URL, isNewTab } from "../services/new-tab";
import { tr } from "../i18n";
import { splitLayout } from "./splitViewStore";
import { focusedTabs, replaceGroupOrder } from "../services/split-tab-groups.js";
import { writable, derived, get } from "svelte/store";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { spring } from "svelte/motion";
import { createSnapshotGate } from "../services/tab-snapshots.js";
import type { QuickLink, TabItem, TabSnapshot, NavigationPatch } from "../types/browser";
import type { AutoHibernateMode } from "./settingsStore";
import {
  defaultSearchEngine,
} from "./searchEngineStore";
import {
  navigateToUrl,
  navigateBack,
  navigateForward,
  reloadCurrentPage,
  fetchTabs,
  createNewTab,
  switchActiveTab,
  closeExistingTab,
  hibernateSingleTab,
  hibernateBackgroundTabs,
  autoHibernateBackgroundTabs,
  reorderTabs,
  moveTabToWorkspace,
  isTauriRuntime,
  setTabPinned,
} from "../services/tauri-bridge";

export const SIDEBAR_EXPANDED_WIDTH = 276;
export const SIDEBAR_COLLAPSED_WIDTH = 0;

/**
 * Reactive state for tabs and active URL.
 */
export const tabs = writable<TabItem[]>([
  {
    id: "tab-1",
    title: tr("newtab.title"),
    url: NEW_TAB_URL,
    workspace_id: "ws-personal",
    is_active: true,
    is_hibernated: false,
  },
]);

export const currentUrl = writable<string>(NEW_TAB_URL);
export const addressInput = writable<string>("");
export const isAddressEditing = writable(false);
export const activeNavigation = derived(tabs, items => items.find(tab => tab.is_active)?.navigation);
export const isLoading = derived(activeNavigation, navigation => navigation?.loading ?? false);
export const isSidebarOpen = writable<boolean>(true);

/**
 * Spring-physics animated width for the collapsible sidebar (60/120 FPS).
 */
export const sidebarWidth = spring<number>(SIDEBAR_EXPANDED_WIDTH, {
  stiffness: 0.14,
  damping: 0.72,
});

export const quickLinks: QuickLink[] = [
  {
    id: "duckduckgo",
    title: "DuckDuckGo",
    url: "https://duckduckgo.com",
    icon: "🦆",
    accent: "#de5833",
  },
  {
    id: "github",
    title: "GitHub",
    url: "https://github.com",
    icon: "🐙",
    accent: "#8b949e",
  },
  {
    id: "rust",
    title: "Rust Lang",
    url: "https://www.rust-lang.org",
    icon: "🦀",
    accent: "#f74c00",
  },
  {
    id: "wikipedia",
    title: "Wikipedia",
    url: "https://www.wikipedia.org",
    icon: "🌐",
    accent: "#6e8efb",
  },
];

let fallbackCounter = 2;

const acceptSnapshot = createSnapshotGate();
export function confirmUrl(url: string, force = false): void {
  currentUrl.set(url);
  if (force || !get(isAddressEditing)) addressInput.set(isNewTab(url) ? "" : url);
}

export function applyNavigationPatch(patch: NavigationPatch): void {
  const existing = get(tabs).find(tab => tab.id === patch.id);
  if (!existing || (existing.navigation?.revision ?? 0) >= patch.navigation.revision) return;
  tabs.update(items => items.map(tab => tab.id === patch.id ? { ...tab, ...patch } : tab));
  if (existing.is_active) confirmUrl(patch.url);
}

export function applyTabsSnapshot(packet: TabSnapshot): void {
  const incoming = acceptSnapshot(packet);
  if (!incoming?.length) return;
  splitLayout.set(Array.isArray(packet) ? null : packet.split_layout ?? null);
  const previous = get(tabs);
  const previousActive = previous.find(tab => tab.is_active)?.id;
  const byId = new Map(previous.map(tab => [tab.id, tab]));
  const snapshot = incoming.map(tab => {
    const old = byId.get(tab.id);
    return old && (old.navigation?.revision ?? 0) > (tab.navigation?.revision ?? 0)
      ? { ...tab, url: old.url, title: old.title, navigation: old.navigation, activity: old.activity } : tab;
  });
  tabs.set(dragOrder ? orderedTabs(snapshot, dragOrder.preview) : snapshot);
  const active = snapshot.find(tab => tab.is_active) ?? snapshot[0];
  activeWorkspaceId.set(active.workspace_id);
  confirmUrl(active.url, active.id !== previousActive);
}

export async function initializeTabs(): Promise<void> {
  const initial = await fetchTabs(NEW_TAB_URL, tr("newtab.title"));
  applyTabsSnapshot(initial);
}

export async function addTab(
  targetUrl?: string,
  workspaceId = get(activeWorkspaceId),
  side?: "left" | "right"
): Promise<void> {
  const urlToOpen = targetUrl?.trim() || NEW_TAB_URL;
  if (!isTauriRuntime()) {
    const resolved = await navigateToUrl(urlToOpen, get(defaultSearchEngine));
    const newId = `tab-${fallbackCounter++}`;
    const updated = get(tabs).map((t) => ({ ...t, is_active: false }));
    updated.push({
      id: newId,
      title: isNewTab(resolved) ? tr("newtab.title") : resolved.replace(/^https?:\/\//, ""),
      url: resolved,
      workspace_id: workspaceId,
      is_active: true,
      is_hibernated: false,
    });
    applyTabsSnapshot(updated);
    return;
  }
  const snapshot = await createNewTab(urlToOpen, get(defaultSearchEngine), side);
  applyTabsSnapshot(snapshot);
}

export async function selectTab(tabId: string): Promise<void> {
  if (!isTauriRuntime()) {
    const updated = get(tabs).map((t) => ({
      ...t,
      is_active: t.id === tabId,
      is_hibernated: t.id === tabId ? false : t.is_hibernated,
    }));
    applyTabsSnapshot(updated);
    return;
  }
  const snapshot = await switchActiveTab(tabId);
  applyTabsSnapshot(snapshot);
}

export async function removeTab(tabId: string, focusContent = false): Promise<void> {
  if (!isTauriRuntime()) {
    const filtered = get(tabs).filter((t) => t.id !== tabId);
    if (filtered.length === 0) {
      tabs.set([]);
      await addTab();
      return;
    }
    if (!filtered.some((t) => t.is_active)) {
      filtered[filtered.length - 1].is_active = true;
      filtered[filtered.length - 1].is_hibernated = false;
    }
    applyTabsSnapshot(filtered);
    return;
  }
  const snapshot = await closeExistingTab(tabId, get(defaultSearchEngine), focusContent);
  applyTabsSnapshot(snapshot);
}

export async function suspendTab(tabId: string): Promise<void> {
  if (!isTauriRuntime()) {
    const updated = get(tabs).map((t) =>
      t.id === tabId && !t.is_active ? { ...t, is_hibernated: true } : t
    );
    applyTabsSnapshot(updated);
    return;
  }
  const snapshot = await hibernateSingleTab(tabId);
  applyTabsSnapshot(snapshot);
}

export async function suspendInactiveTabs(mode?: AutoHibernateMode | 0): Promise<void> {
  if (!isTauriRuntime()) {
    const updated = get(tabs).map((t) =>
      !t.is_active ? { ...t, is_hibernated: true } : t
    );
    applyTabsSnapshot(updated);
    return;
  }
  const snapshot = mode === undefined || mode === 0
    ? await hibernateBackgroundTabs(0)
    : await autoHibernateBackgroundTabs(mode);
  applyTabsSnapshot(snapshot);
}

function orderedTabs(items: TabItem[], ids: string[]): TabItem[] {
  const byId = new Map(items.map(tab => [tab.id, tab]));
  return [...new Set([...ids, ...items.map(tab => tab.id)])]
    .flatMap(id => { const tab = byId.get(id); return tab ? [tab] : []; });
}

let dragOrder: { original: string[]; preview: string[] } | undefined;
let reorderQueue: Promise<void> = Promise.resolve();
let reorderRevision = 0;

export function beginTabReorder(): void {
  const ids = get(tabs).map(tab => tab.id);
  dragOrder = { original: ids, preview: ids };
}

export function previewTabReorder(ids: string[]): void {
  if (!dragOrder) return;
  const preview = orderedTabs(get(tabs), ids);
  dragOrder.preview = preview.map(tab => tab.id);
  tabs.set(preview);
}

export async function finishTabReorder(cancel = false): Promise<void> {
  const drag = dragOrder;
  dragOrder = undefined;
  if (!drag) return;
  if (cancel) { tabs.set(orderedTabs(get(tabs), drag.original)); return; }
  if (drag.preview.join("\0") === drag.original.join("\0")) return;
  await reorderTabsList(drag.preview, drag.original);
}

export async function reorderTabsList(ids: string[], rollback = get(tabs).map(tab => tab.id)): Promise<void> {
  const revision = ++reorderRevision;
  tabs.set(orderedTabs(get(tabs), ids));
  if (!isTauriRuntime()) return;
  const operation = reorderQueue.then(async () => {
    try {
      const snapshot = await reorderTabs(ids);
      if (revision === reorderRevision) applyTabsSnapshot(snapshot);
    } catch (error) {
      if (revision === reorderRevision) tabs.set(orderedTabs(get(tabs), rollback));
      throw error;
    }
  });
  reorderQueue = operation.catch(() => {});
  await operation;
}

export async function changeTabWorkspace(
  tabId: string,
  targetWorkspaceId: string
): Promise<void> {
  const currentTabs = get(tabs);
  const updated = currentTabs.map((t) =>
    t.id === tabId ? { ...t, workspace_id: targetWorkspaceId } : t
  );
  applyTabsSnapshot(updated);

  if (isTauriRuntime()) {
    try {
      const snapshot = await moveTabToWorkspace(tabId, targetWorkspaceId);
      applyTabsSnapshot(snapshot);
    } catch (err) {
      applyTabsSnapshot(currentTabs);
      throw err;
    }
  }
}

export async function moveActiveTabUp(workspaceId?: string): Promise<void> {
  const currentTabs = get(tabs);
  const active = currentTabs.find((t) => t.is_active);
  if (!active) return;

  const targetWs = workspaceId || active.workspace_id || "ws-personal";
  const wsTabs = focusedTabs(currentTabs, get(splitLayout), targetWs);
  const currentIds = wsTabs.map((t) => t.id);
  const idx = currentIds.indexOf(active.id);

  if (idx > 0) {
    const reordered = [...currentIds];
    const [moved] = reordered.splice(idx, 1);
    reordered.splice(idx - 1, 0, moved);
    await reorderTabsList(replaceGroupOrder(currentTabs, reordered));
  }
}

export async function moveActiveTabDown(workspaceId?: string): Promise<void> {
  const currentTabs = get(tabs);
  const active = currentTabs.find((t) => t.is_active);
  if (!active) return;

  const targetWs = workspaceId || active.workspace_id || "ws-personal";
  const wsTabs = focusedTabs(currentTabs, get(splitLayout), targetWs);
  const currentIds = wsTabs.map((t) => t.id);
  const idx = currentIds.indexOf(active.id);

  if (idx !== -1 && idx < currentIds.length - 1) {
    const reordered = [...currentIds];
    const [moved] = reordered.splice(idx, 1);
    reordered.splice(idx + 1, 0, moved);
    await reorderTabsList(replaceGroupOrder(currentTabs, reordered));
  }
}

export function toggleSidebar(): void {
  isSidebarOpen.update((open) => {
    const nextState = !open;
    sidebarWidth.set(nextState ? SIDEBAR_EXPANDED_WIDTH : SIDEBAR_COLLAPSED_WIDTH);
    return nextState;
  });
}

export async function submitNavigation(rawInput: string): Promise<void> {
  const trimmed = rawInput.trim();
  if (!trimmed) return;

  try {
    const resolvedUrl = await navigateToUrl(trimmed, get(defaultSearchEngine));
    if (!isTauriRuntime()) {
      currentUrl.set(resolvedUrl);
      addressInput.set(resolvedUrl);
    }
  } catch (error) {
    addressInput.set(get(currentUrl));
    throw error;
  }
}

export async function triggerBack(): Promise<void> {
  await navigateBack();
}

export async function triggerForward(): Promise<void> {
  await navigateForward();
}

export async function triggerReload(): Promise<void> {
  await reloadCurrentPage();
}

export async function toggleTabPin(tabId: string): Promise<void> {
  const tab = get(tabs).find(tab => tab.id === tabId);
  if (!tab) return;
  if (!isTauriRuntime()) { tabs.update(items => items.map(item => item.id === tabId ? { ...item, is_pinned: !item.is_pinned } : item)); return; }
  applyTabsSnapshot(await setTabPinned(tabId, !tab.is_pinned));
}
