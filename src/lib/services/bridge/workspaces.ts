import { invoke } from '@tauri-apps/api/core';
import { isTauriRuntime } from './runtime';
import type { TabItem, TabSnapshot, WorkspaceDeleteResult, WorkspaceItem } from '../../types/browser';
import * as preview from '../preview/workspaces';
export async function fetchWorkspaces(): Promise<WorkspaceItem[]> {
  if(!isTauriRuntime()) return preview.fetchWorkspaces();
  return await invoke<WorkspaceItem[]>("get_workspaces");
}

export async function fetchActiveWorkspace(): Promise<string> {
  if(!isTauriRuntime()) return preview.fetchActiveWorkspace();
  return await invoke<string>("get_active_workspace");
}

export async function switchWorkspace(workspaceId: string): Promise<TabSnapshot> {
  if(!isTauriRuntime()) return preview.switchWorkspace(workspaceId);
  return await invoke<TabSnapshot>("switch_workspace", { workspaceId });
}

export async function createWorkspaceProfile(
  workspace: WorkspaceItem
): Promise<WorkspaceItem[]> {
  if(!isTauriRuntime()) return preview.createWorkspaceProfile(workspace);
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
  if(!isTauriRuntime()) return preview.updateWorkspaceProfile(workspace);
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
  targetWorkspaceId?: string,
  currentTabs?: TabItem[]
): Promise<WorkspaceDeleteResult> {
  if(!isTauriRuntime()) return preview.deleteWorkspaceProfile(workspaceId, action, targetWorkspaceId, currentTabs);
  return await invoke<WorkspaceDeleteResult>("delete_workspace", {
    workspaceId,
    action,
    targetWorkspaceId: targetWorkspaceId ?? null,
  });
}
