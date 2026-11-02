import { fetchSiteSearchShortcuts } from "./site-search";
import { resolvePreviewInput } from "../site-search.js";
import { invoke } from "./runtime";

import type { TabSnapshot, ViewportBounds } from "../../types/browser";
import type { SearchEngine } from "../../stores/searchEngineStore";
import type { AutoHibernateMode } from "../../stores/settingsStore";

import { isTauriRuntime } from "./runtime";

export async function navigateToUrl(
  input: string,
  searchEngine: SearchEngine = "google"
): Promise<string> {
  if(!isTauriRuntime()) {
    return resolvePreviewInput(input, await fetchSiteSearchShortcuts(), searchEngine);
  }
  return await invoke<string>("navigate_to", { input, searchEngine });
}

export async function navigateBack(): Promise<void> {
  if(!isTauriRuntime()) return;
  await invoke("go_back");
}

export async function navigateForward(): Promise<void> {
  if(!isTauriRuntime()) return;
  await invoke("go_forward");
}

export async function reloadCurrentPage(): Promise<void> {
  if(!isTauriRuntime()) return;
  await invoke("reload_page");
}

export async function syncViewportBounds(bounds: ViewportBounds): Promise<void> {
  if(!isTauriRuntime()) return;
  await invoke("sync_viewport_bounds", { bounds, splitRatio: bounds.split_ratio ?? null, splitKey: bounds.split_key ?? null });
}

export async function setResidentLimit(limit: number): Promise<TabSnapshot> {
  return isTauriRuntime() ? invoke<TabSnapshot>("set_resident_limit", { limit }) : [];
}
export async function setTabPinned(tabId: string, pinned: boolean): Promise<TabSnapshot> {
  return invoke<TabSnapshot>("set_tab_pinned", { tabId, pinned });
}

/**
 * Multi-Tab & Hibernation IPC commands
 */
export async function fetchTabs(
  fallbackUrl = "https://www.google.com/",
  fallbackTitle = "Google"
): Promise<TabSnapshot> {
  if(!isTauriRuntime()) {
    return [
      {
        id: "tab-1",
        title: fallbackTitle,
        url: fallbackUrl,
        workspace_id: "ws-personal",
        is_active: true,
        is_hibernated: false,
      },
    ];
  }
  return await invoke<TabSnapshot>("get_tabs");
}

export async function createNewTab(url?: string, searchEngine: SearchEngine = "google"): Promise<TabSnapshot> {
  if(!isTauriRuntime()) return [];
  return await invoke<TabSnapshot>("create_tab", { url: url ?? null, searchEngine });
}

export async function switchActiveTab(tabId: string): Promise<TabSnapshot> {
  if(!isTauriRuntime()) return [];
  return await invoke<TabSnapshot>("switch_tab", { tabId });
}

export async function closeExistingTab(
  tabId: string,
  searchEngine: SearchEngine = "google",
  focusContent = false
): Promise<TabSnapshot> {
  if(!isTauriRuntime()) return [];
  return await invoke<TabSnapshot>("close_tab", { tabId, searchEngine, focusContent });
}

export async function hibernateSingleTab(tabId: string): Promise<TabSnapshot> {
  if(!isTauriRuntime()) return [];
  return await invoke<TabSnapshot>("hibernate_tab", { tabId });
}

export async function hibernateBackgroundTabs(
  minIdleSecs?: number
): Promise<TabSnapshot> {
  if(!isTauriRuntime()) return [];
  return await invoke<TabSnapshot>("hibernate_inactive_tabs", {
    minIdleSecs: minIdleSecs ?? null,
  });
}

export async function autoHibernateBackgroundTabs(
  mode: AutoHibernateMode
): Promise<TabSnapshot> {
  if(!isTauriRuntime()) return [];
  return await invoke<TabSnapshot>("auto_hibernate_inactive_tabs", { mode: String(mode) });
}
export async function reorderTabs(tabIds: string[]): Promise<TabSnapshot> {
  if(!isTauriRuntime()) return [];
  return await invoke<TabSnapshot>("reorder_tabs", { tabIds });
}

export async function moveTabToWorkspace(
  tabId: string,
  workspaceId: string
): Promise<TabSnapshot> {
  if(!isTauriRuntime()) return [];
  return await invoke<TabSnapshot>("move_tab_to_workspace", { tabId, workspaceId });
}
