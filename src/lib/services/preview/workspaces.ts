import { safeStorage } from '../storage.js';
import type { TabItem, TabSnapshot, WorkspaceDeleteResult, WorkspaceItem } from '../../types/browser';
import {
  DEFAULT_WORKSPACES,
  WORKSPACE_PALETTE_MIGRATION_VERSION,
  migrateLegacyDefaultWorkspaceColors,
} from '../../constants/workspaceDefaults';
import { fetchTabs } from '../bridge/navigation';
const WORKSPACE_PALETTE_MIGRATION_KEY = 'spark_workspace_palette_version';
export async function fetchWorkspaces(): Promise<WorkspaceItem[]> {
  if(typeof safeStorage !== "undefined") {
    const saved = safeStorage.getItem("spark_workspaces");
    if(saved) {
      let parsed: unknown = undefined;
      try {
        parsed = JSON.parse(saved);
      } catch {
        safeStorage.removeItem("spark_workspaces");
      }

      if(Array.isArray(parsed) && parsed.length > 0 && parsed.every(isWorkspaceItem) && new Set(parsed.map(item => item.id)).size === parsed.length) {
        const savedWorkspaces = parsed as WorkspaceItem[];
        const paletteVersion = Number.parseInt(
          safeStorage.getItem(WORKSPACE_PALETTE_MIGRATION_KEY) ?? "0",
          10
        );
        if(
          !Number.isFinite(paletteVersion) ||
          paletteVersion < WORKSPACE_PALETTE_MIGRATION_VERSION
        ) {
          const migrated = migrateLegacyDefaultWorkspaceColors(savedWorkspaces);
          try {
            safeStorage.setItem("spark_workspaces", JSON.stringify(migrated));
            safeStorage.setItem(
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

export async function fetchActiveWorkspace(): Promise<string> {
  return typeof safeStorage === "undefined"
    ? "ws-personal"
    : safeStorage.getItem("spark_active_workspace") ?? "ws-personal";
}

export async function switchWorkspace(_workspaceId: string): Promise<TabSnapshot> {
  return [];
}

export async function createWorkspaceProfile(
  workspace: WorkspaceItem
): Promise<WorkspaceItem[]> {
  const next = [...(await fetchWorkspaces()), workspace];
  if(typeof safeStorage !== "undefined") {
    safeStorage.setItem("spark_workspaces", JSON.stringify(next));
  }
  return next;
}

export async function updateWorkspaceProfile(
  workspace: WorkspaceItem
): Promise<WorkspaceItem[]> {
  const next = (await fetchWorkspaces()).map((item) =>
    item.id === workspace.id ? workspace : item
  );
  if(typeof safeStorage !== "undefined") {
    safeStorage.setItem("spark_workspaces", JSON.stringify(next));
  }
  return next;
}

export async function deleteWorkspaceProfile(
  workspaceId: string,
  action: "move" | "close",
  targetWorkspaceId?: string,
  currentTabs?: TabItem[]
): Promise<WorkspaceDeleteResult> {
  const allWorkspaces = await fetchWorkspaces();
  if(workspaceId === "ws-personal") {
    throw new Error("The Personal workspace cannot be deleted");
  }
  const remaining = allWorkspaces.filter((item) => item.id !== workspaceId);
  const packet = currentTabs ?? await fetchTabs();
  const savedTabs = Array.isArray(packet) ? packet : packet.tabs;
  const activeId =
    typeof safeStorage === "undefined"
      ? "ws-personal"
      : safeStorage.getItem("spark_active_workspace") ?? "ws-personal";
  const deletingActive = activeId === workspaceId;
  const nextActiveId = deletingActive
    ? action === "move"
      ? targetWorkspaceId ?? "ws-personal"
      : "ws-personal"
    : activeId;
  let nextTabs = savedTabs;
  if(action === "move" && targetWorkspaceId) {
    nextTabs = savedTabs.map((tab) =>
      tab.workspace_id === workspaceId
        ? { ...tab, workspace_id: targetWorkspaceId }
        : tab
    );
  } else {
    nextTabs = savedTabs.filter((tab) => tab.workspace_id !== workspaceId);
    if(deletingActive && !nextTabs.some((tab) => tab.workspace_id === "ws-personal")) {
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
  if(typeof safeStorage !== "undefined") {
    safeStorage.setItem("spark_workspaces", JSON.stringify(remaining));
    safeStorage.setItem("spark_active_workspace", nextActiveId);
  }
  return {
    workspaces: remaining,
    tabs: nextTabs,
    active_workspace_id: nextActiveId,
  };
}

function isWorkspaceItem(value: unknown): value is WorkspaceItem {
  if(!value || typeof value !== "object") return false;
  const item = value as Partial<WorkspaceItem>;
  return [item.id, item.name, item.icon, item.accent_primary, item.accent_secondary]
    .every((field) => typeof field === "string")
    && /^[A-Za-z0-9_-]{1,128}$/.test(item.id!) && item.name!.trim().length > 0 && item.name!.length <= 128
    && item.icon!.length <= 64 && /^#[0-9a-f]{6}$/i.test(item.accent_primary!) && /^#[0-9a-f]{6}$/i.test(item.accent_secondary!);
}
