import { writable, get } from "svelte/store";
import type { WorkspaceItem } from "../types/browser";
import {
  fetchWorkspaces,
  fetchActiveWorkspace,
  switchWorkspace,
  isTauriRuntime,
  createWorkspaceProfile,
  updateWorkspaceProfile,
  deleteWorkspaceProfile,
} from "../services/tauri-bridge";
import { applyTabsSnapshot } from "./browserStore";

export const workspaces = writable<WorkspaceItem[]>([
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
]);

export const activeWorkspaceId = writable<string>("ws-personal");

/**
 * Applies the selected workspace's dynamic theme colors to CSS root variables.
 */
export function applyWorkspaceTheme(ws: WorkspaceItem): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.style.setProperty("--accent-primary", ws.accent_primary);
  root.style.setProperty("--accent-secondary", ws.accent_secondary);
  root.style.setProperty(
    "--accent-gradient",
    `linear-gradient(135deg, ${ws.accent_primary} 0%, ${ws.accent_secondary} 100%)`
  );
}

export async function initializeWorkspaces(): Promise<void> {
  const [wsList, currentId] = await Promise.all([
    fetchWorkspaces(),
    fetchActiveWorkspace(),
  ]);
  if (wsList.length > 0) {
    workspaces.set(wsList);
  }
  const found = wsList.find((w) => w.id === currentId) ?? wsList[0];
  activeWorkspaceId.set(found?.id ?? currentId);
  if (found) {
    applyWorkspaceTheme(found);
  }
}

export async function activateWorkspace(workspaceId: string): Promise<void> {
  activeWorkspaceId.set(workspaceId);
  const wsList = get(workspaces);
  const found = wsList.find((w) => w.id === workspaceId);
  if (found) {
    applyWorkspaceTheme(found);
  }
  if (typeof localStorage !== "undefined") {
    if (!isTauriRuntime()) {
      localStorage.setItem("veloce_active_workspace", workspaceId);
    }
  }

  if (isTauriRuntime()) {
    const updatedTabs = await switchWorkspace(workspaceId);
    applyTabsSnapshot(updatedTabs);
  }
}

export function createWorkspaceDraft(): WorkspaceItem {
  const randomId =
    typeof crypto !== "undefined" && "randomUUID" in crypto
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return {
    id: `ws-custom-${randomId}`,
    name: "New Workspace",
    icon: "📁",
    accent_primary: "#3b82f6",
    accent_secondary: "#8b5cf6",
  };
}

export async function saveWorkspaceProfile(
  workspace: WorkspaceItem,
  isNew: boolean
): Promise<WorkspaceItem> {
  const normalized = { ...workspace, name: workspace.name.trim() };
  if (!normalized.name) throw new Error("Workspace name cannot be empty.");
  const duplicate = get(workspaces).some(
    (item) =>
      item.id !== normalized.id &&
      item.name.trim().toLocaleLowerCase() === normalized.name.toLocaleLowerCase()
  );
  if (duplicate) throw new Error("A workspace with this name already exists.");

  const updated = isNew
    ? await createWorkspaceProfile(normalized)
    : await updateWorkspaceProfile(normalized);
  workspaces.set(updated);
  if (get(activeWorkspaceId) === normalized.id) {
    const active = updated.find((item) => item.id === normalized.id);
    if (active) applyWorkspaceTheme(active);
  }
  return normalized;
}

export async function removeWorkspaceProfile(
  workspaceId: string,
  action: "move" | "close",
  targetWorkspaceId?: string
): Promise<void> {
  if (workspaceId === "ws-personal") {
    throw new Error("The Personal workspace cannot be deleted.");
  }
  const result = await deleteWorkspaceProfile(
    workspaceId,
    action,
    targetWorkspaceId
  );
  workspaces.set(result.workspaces);
  activeWorkspaceId.set(result.active_workspace_id);
  if (result.tabs.length > 0) applyTabsSnapshot(result.tabs);
  if (typeof localStorage !== "undefined") {
    if (!isTauriRuntime()) {
      localStorage.setItem("veloce_active_workspace", result.active_workspace_id);
    }
  }
  const active = result.workspaces.find(
    (item) => item.id === result.active_workspace_id
  );
  if (active) applyWorkspaceTheme(active);
}

