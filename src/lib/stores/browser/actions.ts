import { mergeSessionObservations, mergeNavigationObservation } from "../../services/session-reconciliation.js";
import { NEW_TAB_URL, isNewTab } from "../../services/new-tab";
import { tr } from "../../i18n";
import { splitLayout } from "../splitViewStore";
import { focusedTabs, replaceGroupOrder } from "../../services/split-tab-groups.js";
import { get } from "svelte/store";
import { activeWorkspaceId } from "../workspaceSelectionStore";
import { createSnapshotGate } from "../../services/tab-snapshots.js";
import type { QuickLink, TabItem, TabSnapshot, NavigationPatch } from "../../types/browser";
import type { AutoHibernateMode } from "../settingsStore";
import {
  defaultSearchEngine,
} from "../searchEngineStore";
import {
  navigateToUrl,
  navigateBack,
  navigateForward,
  reloadCurrentPage,
  fetchTabs,
  createNewTab,
  createNewEphemeralTab,
  switchActiveTab,
  closeExistingTab,
  hibernateSingleTab,
  hibernateBackgroundTabs,
  autoHibernateBackgroundTabs,
  reorderTabs,
  moveTabToWorkspace,
  isTauriRuntime,
  setTabPinned,
  currentWindowId,
} from "../../services/tauri-bridge";

import { tabs, currentUrl, addressInput, isAddressEditing, isSidebarOpen,
  confirmedTabs, previewTabOrder, replaceTabs, updateTabs, setConfirmedUrl, setSidebarOpen } from "./state";
import { applyConfirmedSplit } from "../splitViewStore";
import { confirmWorkspaceSelection } from "../workspaceSelectionStore";
import { reanchorPreviewDividers } from "../tabDividersStore";
export { tabs, currentUrl, addressInput, isAddressEditing, activeNavigation, isLoading,
  isSidebarOpen, sidebarWidth, SIDEBAR_EXPANDED_WIDTH, SIDEBAR_COLLAPSED_WIDTH,
  setSidebarOpen } from "./state";

export const quickLinks: QuickLink[] = [
  {
    id: "duckduckgo",
    title: "DuckDuckGo",
    url: "https://duckduckgo.com",
    icon: "search",
    accent: "#de5833",
  },
  {
    id: "github",
    title: "GitHub",
    url: "https://github.com",
    icon: "code",
    accent: "#8b949e",
  },
  {
    id: "rust",
    title: "Rust Lang",
    url: "https://www.rust-lang.org",
    icon: "box",
    accent: "#f74c00",
  },
  {
    id: "wikipedia",
    title: "Wikipedia",
    url: "https://www.wikipedia.org",
    icon: "globe",
    accent: "#6e8efb",
  },
];

let fallbackCounter = 2;

const acceptSnapshot = createSnapshotGate();
export function confirmUrl(url: string, force = false): void {
  setConfirmedUrl(url);
  if (force || !get(isAddressEditing)) addressInput.set(isNewTab(url) ? "" : url);
}

export function applyNavigationPatch(patch: NavigationPatch): void {
  const existing = get(tabs).find(tab => tab.id === patch.id);
  if (!existing || (existing.navigation?.revision ?? 0) >= patch.navigation.revision) return;
  updateTabs(items => mergeNavigationObservation(items, patch));
  if (existing.is_active) confirmUrl(patch.url);
}

export function applyTabsSnapshot(packet: TabSnapshot): void {
  // Every window is sent its own tabs; ignore what another window shows.
  if (!Array.isArray(packet) && packet.window_id && packet.window_id !== currentWindowId()) return;
  const incoming = acceptSnapshot(packet);
  if (!incoming?.length) return;
  applyConfirmedSplit(Array.isArray(packet) ? null : packet.split_layout ?? null);
  const previous = get(tabs);
  const previousActive = previous.find(tab => tab.is_active)?.id;
  const snapshot = mergeSessionObservations(previous, incoming);
  replaceTabs(snapshot);
  const active = snapshot.find(tab => tab.is_active) ?? snapshot[0];
  confirmWorkspaceSelection(active.workspace_id);
  confirmUrl(active.url, active.id !== previousActive);
}

export async function initializeTabs(active = () => true): Promise<void> {
  const initial = await fetchTabs(NEW_TAB_URL, tr("newtab.title"));
  if (active()) applyTabsSnapshot(initial);
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

export async function addEphemeralTab(targetUrl?: string): Promise<void> {
  const urlToOpen = targetUrl?.trim() || NEW_TAB_URL;
  if (!isTauriRuntime()) {
    const resolved = await navigateToUrl(urlToOpen, get(defaultSearchEngine));
    const newId = `tab-${fallbackCounter++}`;
    const updated = get(tabs).map((t) => ({ ...t, is_active: false }));
    updated.push({
      id: newId,
      title: isNewTab(resolved) ? tr("newtab.title") : resolved.replace(/^https?:\/\//, ""),
      url: resolved,
      workspace_id: get(activeWorkspaceId),
      is_active: true,
      is_hibernated: false,
      is_ephemeral: true,
    });
    applyTabsSnapshot(updated);
    return;
  }
  const snapshot = await createNewEphemeralTab(urlToOpen, get(defaultSearchEngine));
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
    const current = get(tabs);
    const index = current.findIndex(tab => tab.id === tabId);
    if (index !== -1) {
      const workspaceId = current[index].workspace_id;
      const previous = current.slice(0, index).findLast(tab => tab.workspace_id === workspaceId);
      reanchorPreviewDividers(tabId, previous?.id ?? "");
    }
    const filtered = current.filter((t) => t.id !== tabId);
    if (filtered.length === 0) {
      replaceTabs([]);
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
  previewTabOrder(preview.map(tab => tab.id));
}

export async function finishTabReorder(cancel = false): Promise<void> {
  const drag = dragOrder;
  dragOrder = undefined;
  if (!drag) return;
  if (cancel) { previewTabOrder(null); return; }
  if (drag.preview.join("\0") === drag.original.join("\0")) { previewTabOrder(null); return; }
  await reorderTabsList(drag.preview);
}

export async function reorderTabsList(ids: string[]): Promise<void> {
  const revision = ++reorderRevision;
  previewTabOrder(ids);
  if (!isTauriRuntime()) {
    replaceTabs(orderedTabs(get(confirmedTabs), ids));
    previewTabOrder(null);
    return;
  }
  const operation = reorderQueue.then(async () => {
    try {
      const snapshot = await reorderTabs(ids);
      if (revision === reorderRevision) { applyTabsSnapshot(snapshot); previewTabOrder(null); }
    } catch (error) {
      if (revision === reorderRevision) previewTabOrder(null);
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
  if (isTauriRuntime()) {
    applyTabsSnapshot(await moveTabToWorkspace(tabId, targetWorkspaceId));
    return;
  }
  applyTabsSnapshot(get(tabs).map(tab => tab.id === tabId
    ? { ...tab, workspace_id: targetWorkspaceId } : tab));
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
  setSidebarOpen(!get(isSidebarOpen));
}

export async function submitNavigation(rawInput: string): Promise<void> {
  const trimmed = rawInput.trim();
  if (!trimmed) return;

  try {
    const resolvedUrl = await navigateToUrl(trimmed, get(defaultSearchEngine));
    if (!isTauriRuntime()) {
      setConfirmedUrl(resolvedUrl);
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
  if (!isTauriRuntime()) { updateTabs(items => items.map(item => item.id === tabId ? { ...item, is_pinned: !item.is_pinned } : item)); return; }
  applyTabsSnapshot(await setTabPinned(tabId, !tab.is_pinned));
}
