import { writable, get } from "svelte/store";
import { spring } from "svelte/motion";
import type { QuickLink, TabItem } from "../types/browser";
import { defaultSearchEngine } from "./searchEngineStore";
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
  reorderTabs,
  moveTabToWorkspace,
  isTauriRuntime,
} from "../services/tauri-bridge";

export const SIDEBAR_EXPANDED_WIDTH = 276;
export const SIDEBAR_COLLAPSED_WIDTH = 0;

/**
 * Reactive state for tabs and active URL.
 */
export const tabs = writable<TabItem[]>([
  {
    id: "tab-1",
    title: "DuckDuckGo",
    url: "https://duckduckgo.com",
    workspace_id: "ws-personal",
    is_active: true,
    is_hibernated: false,
  },
]);

export const currentUrl = writable<string>("https://duckduckgo.com");
export const addressInput = writable<string>("https://duckduckgo.com");
export const isLoading = writable<boolean>(false);
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

export function applyTabsSnapshot(snapshot: TabItem[]): void {
  if (!snapshot || snapshot.length === 0) return;
  tabs.set(snapshot);
  const active = snapshot.find((t) => t.is_active) ?? snapshot[0];
  if (active) {
    currentUrl.set(active.url);
    addressInput.set(active.url);
  }
}

export async function initializeTabs(): Promise<void> {
  const initial = await fetchTabs();
  applyTabsSnapshot(initial);
}

export async function addTab(
  targetUrl?: string,
  workspaceId = "ws-personal"
): Promise<void> {
  const urlToOpen = targetUrl?.trim() || "https://duckduckgo.com";
  isLoading.set(true);
  try {
    if (!isTauriRuntime()) {
      const newId = `tab-${fallbackCounter++}`;
      const updated = get(tabs).map((t) => ({ ...t, is_active: false }));
      updated.push({
        id: newId,
        title: urlToOpen.replace(/^https?:\/\//, ""),
        url: urlToOpen,
        workspace_id: workspaceId,
        is_active: true,
        is_hibernated: false,
      });
      applyTabsSnapshot(updated);
      return;
    }
    const snapshot = await createNewTab(urlToOpen);
    applyTabsSnapshot(snapshot);
  } finally {
    setTimeout(() => isLoading.set(false), 300);
  }
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

export async function removeTab(tabId: string): Promise<void> {
  if (!isTauriRuntime()) {
    const filtered = get(tabs).filter((t) => t.id !== tabId);
    if (filtered.length === 0) {
      await addTab("https://duckduckgo.com");
      return;
    }
    if (!filtered.some((t) => t.is_active)) {
      filtered[filtered.length - 1].is_active = true;
      filtered[filtered.length - 1].is_hibernated = false;
    }
    applyTabsSnapshot(filtered);
    return;
  }
  const snapshot = await closeExistingTab(tabId);
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

export async function suspendInactiveTabs(minIdleSecs?: number): Promise<void> {
  if (!isTauriRuntime()) {
    const updated = get(tabs).map((t) =>
      !t.is_active ? { ...t, is_hibernated: true } : t
    );
    applyTabsSnapshot(updated);
    return;
  }
  const snapshot = await hibernateBackgroundTabs(minIdleSecs);
  applyTabsSnapshot(snapshot);
}

export async function reorderTabsList(orderedTabIds: string[]): Promise<void> {
  const currentTabs = get(tabs);
  const tabMap = new Map(currentTabs.map((t) => [t.id, t]));
  const reordered: TabItem[] = [];

  for (const id of orderedTabIds) {
    const item = tabMap.get(id);
    if (item) {
      reordered.push(item);
      tabMap.delete(id);
    }
  }

  for (const remaining of currentTabs) {
    if (tabMap.has(remaining.id)) {
      reordered.push(remaining);
    }
  }

  tabs.set(reordered);

  if (isTauriRuntime()) {
    try {
      const snapshot = await reorderTabs(orderedTabIds);
      applyTabsSnapshot(snapshot);
    } catch (err) {
      console.error("Failed to reorder tabs:", err);
    }
  }
}

export async function changeTabWorkspace(
  tabId: string,
  targetWorkspaceId: string
): Promise<void> {
  const currentTabs = get(tabs);
  const updated = currentTabs.map((t) =>
    t.id === tabId ? { ...t, workspace_id: targetWorkspaceId } : t
  );
  tabs.set(updated);

  if (isTauriRuntime()) {
    try {
      const snapshot = await moveTabToWorkspace(tabId, targetWorkspaceId);
      applyTabsSnapshot(snapshot);
    } catch (err) {
      console.error("Failed to move tab to workspace:", err);
    }
  }
}

export async function moveActiveTabUp(workspaceId?: string): Promise<void> {
  const currentTabs = get(tabs);
  const active = currentTabs.find((t) => t.is_active);
  if (!active) return;

  const targetWs = workspaceId || active.workspace_id || "ws-personal";
  const wsTabs = currentTabs.filter(
    (t) => !t.workspace_id || t.workspace_id === targetWs
  );
  const currentIds = wsTabs.map((t) => t.id);
  const idx = currentIds.indexOf(active.id);

  if (idx > 0) {
    const reordered = [...currentIds];
    const [moved] = reordered.splice(idx, 1);
    reordered.splice(idx - 1, 0, moved);
    await reorderTabsList(reordered);
  }
}

export async function moveActiveTabDown(workspaceId?: string): Promise<void> {
  const currentTabs = get(tabs);
  const active = currentTabs.find((t) => t.is_active);
  if (!active) return;

  const targetWs = workspaceId || active.workspace_id || "ws-personal";
  const wsTabs = currentTabs.filter(
    (t) => !t.workspace_id || t.workspace_id === targetWs
  );
  const currentIds = wsTabs.map((t) => t.id);
  const idx = currentIds.indexOf(active.id);

  if (idx !== -1 && idx < currentIds.length - 1) {
    const reordered = [...currentIds];
    const [moved] = reordered.splice(idx, 1);
    reordered.splice(idx + 1, 0, moved);
    await reorderTabsList(reordered);
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

  isLoading.set(true);
  try {
    const resolvedUrl = await navigateToUrl(trimmed, get(defaultSearchEngine));
    currentUrl.set(resolvedUrl);
    addressInput.set(resolvedUrl);
  } finally {
    setTimeout(() => isLoading.set(false), 350);
  }
}

export async function triggerBack(): Promise<void> {
  await navigateBack();
}

export async function triggerForward(): Promise<void> {
  await navigateForward();
}

export async function triggerReload(): Promise<void> {
  isLoading.set(true);
  try {
    await reloadCurrentPage();
  } finally {
    setTimeout(() => isLoading.set(false), 350);
  }
}
