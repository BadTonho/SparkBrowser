import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  BookmarkItem,
  DownloadItem,
  HistoryEntry,
  PrivacyStats,
  TabItem,
  ViewportBounds,
  WorkspaceDeleteResult,
  WorkspaceItem,
} from "../types/browser";
import type { SearchEngine } from "../stores/searchEngineStore";
import {
  DEFAULT_WORKSPACES,
  WORKSPACE_PALETTE_MIGRATION_VERSION,
  migrateLegacyDefaultWorkspaceColors,
} from "../constants/workspaceDefaults";

const WORKSPACE_PALETTE_MIGRATION_KEY = "veloce_workspace_palette_version";

/**
 * Checks whether the frontend is running inside the Tauri runtime.
 */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Navigates the active Rust child WebView to a URL or search query.
 */
export async function navigateToUrl(
  input: string,
  searchEngine: SearchEngine = "google"
): Promise<string> {
  if (!isTauriRuntime()) {
    return input.startsWith("http") ? input : `https://${input}`;
  }
  return await invoke<string>("navigate_to", { input, searchEngine });
}

export async function navigateBack(): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("go_back");
}

export async function navigateForward(): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("go_forward");
}

export async function reloadCurrentPage(): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("reload_page");
}

export async function syncViewportBounds(bounds: ViewportBounds): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("sync_viewport_bounds", { bounds });
}

/**
 * Multi-Tab & Hibernation IPC commands
 */
export async function fetchTabs(
  fallbackUrl = "https://www.google.com/",
  fallbackTitle = "Google"
): Promise<TabItem[]> {
  if (!isTauriRuntime()) {
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
  return await invoke<TabItem[]>("get_tabs");
}

export async function createNewTab(url?: string): Promise<TabItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<TabItem[]>("create_tab", { url: url ?? null });
}

export async function switchActiveTab(tabId: string): Promise<TabItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<TabItem[]>("switch_tab", { tabId });
}

export async function closeExistingTab(
  tabId: string,
  searchEngine: SearchEngine = "google"
): Promise<TabItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<TabItem[]>("close_tab", { tabId, searchEngine });
}

export async function hibernateSingleTab(tabId: string): Promise<TabItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<TabItem[]>("hibernate_tab", { tabId });
}

export async function hibernateBackgroundTabs(
  minIdleSecs?: number
): Promise<TabItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<TabItem[]>("hibernate_inactive_tabs", {
    minIdleSecs: minIdleSecs ?? null,
  });
}

export type OverlayId =
  | "command-bar"
  | "history"
  | "downloads"
  | "settings";

const activeOverlayIds = new Set<OverlayId>();
let overlaySyncQueue: Promise<void> = Promise.resolve();

/** Keeps the native content WebView hidden while any browser overlay is open. */
export async function setOverlayVisibility(
  visible: boolean,
  overlayId: OverlayId
): Promise<void> {
  if (visible) {
    activeOverlayIds.add(overlayId);
  } else {
    activeOverlayIds.delete(overlayId);
  }

  if (!isTauriRuntime()) return;

  const shouldHideContent = activeOverlayIds.size > 0;
  overlaySyncQueue = overlaySyncQueue
    .catch(() => undefined)
    .then(() => invoke("set_overlay_visibility", { visible: shouldHideContent }));
  await overlaySyncQueue;
}

let workspaceSwitcherSyncQueue: Promise<void> = Promise.resolve();

/** Shows the workspace picker in its transparent native overlay WebView. */
export async function setWorkspaceSwitcherOverlay(
  visible: boolean,
  workspaces: WorkspaceItem[],
  activeWorkspaceId: string,
  backdropBlurPx: number
): Promise<void> {
  if (!isTauriRuntime()) return;

  workspaceSwitcherSyncQueue = workspaceSwitcherSyncQueue
    .catch(() => undefined)
    .then(() =>
      invoke<void>("set_workspace_switcher_overlay", {
        visible,
        workspaces,
        activeWorkspaceId,
        backdropBlurPx,
      })
    );
  await workspaceSwitcherSyncQueue;
}

export async function reorderTabs(tabIds: string[]): Promise<TabItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<TabItem[]>("reorder_tabs", { tabIds });
}

export async function moveTabToWorkspace(
  tabId: string,
  workspaceId: string
): Promise<TabItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<TabItem[]>("move_tab_to_workspace", { tabId, workspaceId });
}

/**
 * Workspaces & Dynamic Themes IPC commands
 */
export async function fetchWorkspaces(): Promise<WorkspaceItem[]> {
  if (!isTauriRuntime()) {
    if (typeof localStorage !== "undefined") {
      const saved = localStorage.getItem("veloce_workspaces");
      if (saved) {
        let parsed: unknown = undefined;
        try {
          parsed = JSON.parse(saved);
        } catch {
          localStorage.removeItem("veloce_workspaces");
        }

        if (Array.isArray(parsed) && parsed.every(isWorkspaceItem)) {
          const savedWorkspaces = parsed as WorkspaceItem[];
          const paletteVersion = Number.parseInt(
            localStorage.getItem(WORKSPACE_PALETTE_MIGRATION_KEY) ?? "0",
            10
          );
          if (
            !Number.isFinite(paletteVersion) ||
            paletteVersion < WORKSPACE_PALETTE_MIGRATION_VERSION
          ) {
            const migrated = migrateLegacyDefaultWorkspaceColors(savedWorkspaces);
            try {
              localStorage.setItem("veloce_workspaces", JSON.stringify(migrated));
              localStorage.setItem(
                WORKSPACE_PALETTE_MIGRATION_KEY,
                String(WORKSPACE_PALETTE_MIGRATION_VERSION)
              );
            } catch {
              // Keep the migrated values in memory and retry persistence next launch.
            }
            return migrated;
          }
          return savedWorkspaces;
        }
      }
    }
    return DEFAULT_WORKSPACES.map((workspace) => ({ ...workspace }));
  }
  return await invoke<WorkspaceItem[]>("get_workspaces");
}

export async function fetchActiveWorkspace(): Promise<string> {
  if (!isTauriRuntime()) {
    return typeof localStorage === "undefined"
      ? "ws-personal"
      : localStorage.getItem("veloce_active_workspace") ?? "ws-personal";
  }
  return await invoke<string>("get_active_workspace");
}

export async function switchWorkspace(workspaceId: string): Promise<TabItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<TabItem[]>("switch_workspace", { workspaceId });
}

export async function createWorkspaceProfile(
  workspace: WorkspaceItem
): Promise<WorkspaceItem[]> {
  if (!isTauriRuntime()) {
    const next = [...(await fetchWorkspaces()), workspace];
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("veloce_workspaces", JSON.stringify(next));
    }
    return next;
  }
  return await invoke<WorkspaceItem[]>("create_workspace", {
    id: workspace.id,
    name: workspace.name,
    icon: workspace.icon,
    accentPrimary: workspace.accent_primary,
    accentSecondary: workspace.accent_secondary,
  });
}

export async function updateWorkspaceProfile(
  workspace: WorkspaceItem
): Promise<WorkspaceItem[]> {
  if (!isTauriRuntime()) {
    const next = (await fetchWorkspaces()).map((item) =>
      item.id === workspace.id ? workspace : item
    );
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("veloce_workspaces", JSON.stringify(next));
    }
    return next;
  }
  return await invoke<WorkspaceItem[]>("update_workspace", {
    id: workspace.id,
    name: workspace.name,
    icon: workspace.icon,
    accentPrimary: workspace.accent_primary,
    accentSecondary: workspace.accent_secondary,
  });
}

export async function deleteWorkspaceProfile(
  workspaceId: string,
  action: "move" | "close",
  targetWorkspaceId?: string
): Promise<WorkspaceDeleteResult> {
  if (isTauriRuntime()) {
    return await invoke<WorkspaceDeleteResult>("delete_workspace", {
      workspaceId,
      action,
      targetWorkspaceId: targetWorkspaceId ?? null,
    });
  }

  const allWorkspaces = await fetchWorkspaces();
  if (workspaceId === "ws-personal") {
    throw new Error("The Personal workspace cannot be deleted");
  }
  const remaining = allWorkspaces.filter((item) => item.id !== workspaceId);
  const savedTabs = await fetchTabs();
  const activeId =
    typeof localStorage === "undefined"
      ? "ws-personal"
      : localStorage.getItem("veloce_active_workspace") ?? "ws-personal";
  const deletingActive = activeId === workspaceId;
  const nextActiveId = deletingActive
    ? action === "move"
      ? targetWorkspaceId ?? "ws-personal"
      : "ws-personal"
    : activeId;
  let nextTabs = savedTabs;
  if (action === "move" && targetWorkspaceId) {
    nextTabs = savedTabs.map((tab) =>
      tab.workspace_id === workspaceId
        ? { ...tab, workspace_id: targetWorkspaceId }
        : tab
    );
  } else {
    nextTabs = savedTabs.filter((tab) => tab.workspace_id !== workspaceId);
    if (deletingActive && !nextTabs.some((tab) => tab.workspace_id === "ws-personal")) {
      nextTabs.push({
        id: `tab-${Date.now()}`,
        title: "DuckDuckGo",
        url: "https://duckduckgo.com",
        workspace_id: "ws-personal",
        is_active: false,
        is_hibernated: false,
      });
    }
  }
  const activeTab = deletingActive
    ? [...nextTabs].reverse().find((tab) => tab.workspace_id === nextActiveId)
    : nextTabs.find((tab) => tab.is_active);
  nextTabs = nextTabs.map((tab) => ({
    ...tab,
    is_active: tab.id === activeTab?.id,
  }));
  if (typeof localStorage !== "undefined") {
    localStorage.setItem("veloce_workspaces", JSON.stringify(remaining));
    localStorage.setItem("veloce_active_workspace", nextActiveId);
  }
  return {
    workspaces: remaining,
    tabs: nextTabs,
    active_workspace_id: nextActiveId,
  };
}

function isWorkspaceItem(value: unknown): value is WorkspaceItem {
  if (!value || typeof value !== "object") return false;
  const item = value as Partial<WorkspaceItem>;
  return [item.id, item.name, item.icon, item.accent_primary, item.accent_secondary]
    .every((field) => typeof field === "string");
}

/**
 * Download Manager IPC commands
 */
export async function fetchDownloads(): Promise<DownloadItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<DownloadItem[]>("get_downloads");
}

export async function enqueueDownloadUrl(url: string): Promise<DownloadItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<DownloadItem[]>("enqueue_download", { url });
}

export async function clearAllDownloads(): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("clear_all_downloads");
}

/**
 * SQLite Persistence IPC commands (History & Bookmarks)
 */
export async function fetchHistory(limit = 50): Promise<HistoryEntry[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<HistoryEntry[]>("get_history", { limit });
}

export async function clearAllHistory(): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("clear_browsing_history");
}

export async function fetchBookmarks(): Promise<BookmarkItem[]> {
  if (!isTauriRuntime()) {
    return [
      { id: 1, title: "DuckDuckGo", url: "https://duckduckgo.com", icon: "🦆" },
      { id: 2, title: "GitHub", url: "https://github.com", icon: "🐙" },
      { id: 3, title: "Rust Lang", url: "https://www.rust-lang.org", icon: "🦀" },
      { id: 4, title: "Wikipedia", url: "https://www.wikipedia.org", icon: "🌐" },
    ];
  }
  return await invoke<BookmarkItem[]>("get_bookmarks");
}

export async function toggleBookmarkItem(
  title: string,
  url: string,
  icon?: string
): Promise<BookmarkItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<BookmarkItem[]>("toggle_bookmark", {
    title,
    url,
    icon: icon ?? null,
  });
}

/**
 * Privacy & Native AdBlock Shield IPC commands
 */
export async function fetchPrivacyStats(): Promise<PrivacyStats> {
  if (!isTauriRuntime()) {
    return {
      shield_enabled: true,
      blocked_count: 0,
      hibernated_tabs_count: 0,
    };
  }
  return await invoke<PrivacyStats>("get_privacy_stats");
}

export async function toggleAdblockShield(): Promise<PrivacyStats> {
  if (!isTauriRuntime()) {
    return {
      shield_enabled: true,
      blocked_count: 0,
      hibernated_tabs_count: 0,
    };
  }
  return await invoke<PrivacyStats>("toggle_adblock_shield");
}

/**
 * Event Listeners from Rust Core
 */
export async function onContentUrlChanged(
  callback: (url: string) => void
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) return () => {};
  return await listen<string>("content-url-changed", (event) => {
    callback(event.payload);
  });
}

export async function onTabsUpdated(
  callback: (tabs: TabItem[]) => void
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) return () => {};
  return await listen<TabItem[]>("tabs-updated", (event) => {
    callback(event.payload);
  });
}

export async function onPrivacyStatsUpdated(
  callback: (stats: PrivacyStats) => void
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) return () => {};
  return await listen<PrivacyStats>("privacy-stats-updated", (event) => {
    callback(event.payload);
  });
}

export interface WorkspaceShortcutEvent {
  type: "alt-down" | "alt-up" | "alt-cancel" | "select-index";
  index?: number;
}

export async function onWorkspaceShortcutEvent(
  callback: (event: WorkspaceShortcutEvent) => void
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) return () => {};
  return await listen<WorkspaceShortcutEvent>("workspace-shortcut-event", (event) => {
    callback(event.payload);
  });
}

export async function setKeyboardShortcuts(shortcuts: Record<string, unknown>): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<void>("set_keyboard_shortcuts", { shortcuts });
}

export async function onBrowserShortcutAction(
  callback: (action: string, key?: string) => void
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) return () => {};
  return await listen<{ action: string; key?: string }>("browser-shortcut-action", (event) => {
    callback(event.payload.action, event.payload.key);
  });
}

export async function onWorkspaceShortcutCancel(
  callback: () => void
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) return () => {};
  return await listen("workspace-shortcut-cancel", callback);
}

export async function onWorkspaceSwitcherSelected(
  callback: (workspaceId: string) => void
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) return () => {};
  return await listen<string>("workspace-switcher-selected", (event) => {
    callback(event.payload);
  });
}

export async function onWorkspaceSwitcherDismissed(
  callback: () => void
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) return () => {};
  return await listen("workspace-switcher-dismissed", callback);
}


/**
 * Window control helpers for custom frameless titlebar.
 */
export async function minimizeWindow(): Promise<void> {
  if (!isTauriRuntime()) return;
  await getCurrentWindow().minimize();
}

export async function toggleMaximizeWindow(): Promise<void> {
  if (!isTauriRuntime()) return;
  await getCurrentWindow().toggleMaximize();
}

export async function closeWindow(): Promise<void> {
  if (!isTauriRuntime()) return;
  await getCurrentWindow().close();
}
