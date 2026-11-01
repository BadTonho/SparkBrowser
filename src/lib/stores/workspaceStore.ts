import { safeStorage } from "../services/storage.js";
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
import {
  DEFAULT_WORKSPACES,
  NEW_WORKSPACE_DEFAULT_COLORS,
} from "../constants/workspaceDefaults";

export const workspaces = writable<WorkspaceItem[]>(
  DEFAULT_WORKSPACES.map((workspace) => ({ ...workspace }))
);

export { activeWorkspaceId } from "./workspaceSelectionStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";

activeWorkspaceId.subscribe((id) => {
  const workspace = get(workspaces).find((item) => item.id === id);
  if (workspace) applyWorkspaceTheme(workspace);
});

function getRelativeLuminance(color: string): number {
  const match = /^#([\da-f]{6})$/i.exec(color);
  if (!match) return 0.5;

  const channels = [0, 2, 4].map((offset) => {
    const value = Number.parseInt(match[1].slice(offset, offset + 2), 16) / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function getAccentForeground(primary: string, secondary: string): string {
  const backgroundLuminances = [
    getRelativeLuminance(primary),
    getRelativeLuminance(secondary),
  ];
  const nearBlackLuminance = getRelativeLuminance("#111111");
  const minimumContrast = (foregroundLuminance: number) =>
    Math.min(
      ...backgroundLuminances.map((backgroundLuminance) => {
        const lighter = Math.max(backgroundLuminance, foregroundLuminance);
        const darker = Math.min(backgroundLuminance, foregroundLuminance);
        return (lighter + 0.05) / (darker + 0.05);
      })
    );

  return minimumContrast(nearBlackLuminance) >= minimumContrast(1)
    ? "#111111"
    : "#ffffff";
}

/**
 * Applies the selected workspace's dynamic theme colors to CSS root variables.
 */
export function applyWorkspaceTheme(ws: WorkspaceItem): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.style.setProperty("--accent-primary", ws.accent_primary);
  root.style.setProperty("--accent-secondary", ws.accent_secondary);
  root.style.setProperty(
    "--accent-foreground",
    getAccentForeground(ws.accent_primary, ws.accent_secondary)
  );
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
  if (isTauriRuntime()) {
    applyTabsSnapshot(await switchWorkspace(workspaceId));
    return;
  }
  activeWorkspaceId.set(workspaceId);
  if (typeof safeStorage !== "undefined") safeStorage.setItem("veloce_active_workspace", workspaceId);
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
    ...NEW_WORKSPACE_DEFAULT_COLORS,
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
  applyTabsSnapshot(result.tabs);
  if (typeof safeStorage !== "undefined") {
    if (!isTauriRuntime()) {
      safeStorage.setItem("veloce_active_workspace", result.active_workspace_id);
    }
  }
  const active = result.workspaces.find(
    (item) => item.id === result.active_workspace_id
  );
  if (active) applyWorkspaceTheme(active);
}

