import { writable, get } from "svelte/store";
import type { WorkspaceItem } from "../types/browser";
import {
  fetchWorkspaces,
  fetchActiveWorkspace,
  switchWorkspace,
  isTauriRuntime,
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
  activeWorkspaceId.set(currentId);
  const found = wsList.find((w) => w.id === currentId) ?? wsList[0];
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

  if (isTauriRuntime()) {
    const updatedTabs = await switchWorkspace(workspaceId);
    applyTabsSnapshot(updatedTabs);
  }
}

