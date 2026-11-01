import { invoke } from "./runtime";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { PrivacyStats, TabSnapshot, NavigationPatch } from "../../types/browser";

import { isTauriRuntime } from "./runtime";

export async function onContentUrlChanged(
  callback: (url: string) => void
): Promise<UnlistenFn> {
  if(!isTauriRuntime()) return () => { };
  return await listen<string>("content-url-changed", (event) => {
    callback(event.payload);
  });
}

export async function onTabsUpdated(
  callback: (tabs: TabSnapshot) => void
): Promise<UnlistenFn> {
  if(!isTauriRuntime()) return () => { };
  return await listen<TabSnapshot>("tabs-updated", (event) => {
    callback(event.payload);
  });
}

export async function onPrivacyStatsUpdated(
  callback: (stats: PrivacyStats) => void
): Promise<UnlistenFn> {
  if(!isTauriRuntime()) return () => { };
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
  if(!isTauriRuntime()) return () => { };
  return await listen<WorkspaceShortcutEvent>("workspace-shortcut-event", (event) => {
    callback(event.payload);
  });
}

export async function setKeyboardShortcuts(shortcuts: Record<string, unknown>): Promise<void> {
  if(!isTauriRuntime()) return;
  await invoke<void>("set_keyboard_shortcuts", { shortcuts });
}

export async function onBrowserShortcutAction(
  callback: (action: string, key?: string) => void
): Promise<UnlistenFn> {
  if(!isTauriRuntime()) return () => { };
  return await listen<{ action: string; key?: string }>("browser-shortcut-action", (event) => {
    callback(event.payload.action, event.payload.key);
  });
}

export async function onWorkspaceShortcutCancel(
  callback: () => void
): Promise<UnlistenFn> {
  if(!isTauriRuntime()) return () => { };
  return await listen("workspace-shortcut-cancel", callback);
}

export async function onWorkspaceSwitcherSelected(
  callback: (workspaceId: string) => void
): Promise<UnlistenFn> {
  if(!isTauriRuntime()) return () => { };
  return await listen<string>("workspace-switcher-selected", (event) => {
    callback(event.payload);
  });
}

export async function onWorkspaceSwitcherDismissed(
  callback: () => void
): Promise<UnlistenFn> {
  if(!isTauriRuntime()) return () => { };
  return await listen("workspace-switcher-dismissed", callback);
}
export async function onNavigationUpdated(callback: (patch: NavigationPatch) => void): Promise<UnlistenFn> {
  if(!isTauriRuntime()) return () => { };
  return listen<NavigationPatch>("tab-navigation-updated", event => callback(event.payload));
}
