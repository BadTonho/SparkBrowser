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
  WorkspaceItem,
} from "../types/browser";

/**
 * Checks whether the frontend is running inside the Tauri runtime.
 */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Navigates the active Rust child WebView to a URL or search query.
 */
export async function navigateToUrl(input: string): Promise<string> {
  if (!isTauriRuntime()) {
    return input.startsWith("http") ? input : `https://${input}`;
  }
  return await invoke<string>("navigate_to", { input });
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
export async function fetchTabs(): Promise<TabItem[]> {
  if (!isTauriRuntime()) {
    return [
      {
        id: "tab-1",
        title: "DuckDuckGo",
        url: "https://duckduckgo.com",
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

export async function closeExistingTab(tabId: string): Promise<TabItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<TabItem[]>("close_tab", { tabId });
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

export async function setOverlayVisibility(visible: boolean): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("set_overlay_visibility", { visible });
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
    return [
      {
        id: "ws-personal",
        name: "Personal",
        icon: "🌟",
        accent_primary: "#6e8efb",
        accent_secondary: "#a777e3",
      },
      {
        id: "ws-work",
        name: "Work",
        icon: "💼",
        accent_primary: "#38bdf8",
        accent_secondary: "#6366f1",
      },
      {
        id: "ws-study",
        name: "Study",
        icon: "📚",
        accent_primary: "#34d399",
        accent_secondary: "#059669",
      },
      {
        id: "ws-gaming",
        name: "Gaming",
        icon: "🎮",
        accent_primary: "#f43f5e",
        accent_secondary: "#d946ef",
      },
    ];
  }
  return await invoke<WorkspaceItem[]>("get_workspaces");
}

export async function fetchActiveWorkspace(): Promise<string> {
  if (!isTauriRuntime()) return "ws-personal";
  return await invoke<string>("get_active_workspace");
}

export async function switchWorkspace(workspaceId: string): Promise<TabItem[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<TabItem[]>("switch_workspace", { workspaceId });
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
