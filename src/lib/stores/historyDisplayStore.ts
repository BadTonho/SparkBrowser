/**
 * Where History opens: the floating panel ("popup") or its own tab ("tab", `spark://history`).
 * Saved on this computer, like the settings display.
 */
import { get, writable } from "svelte/store";
import { safeStorage } from "../services/storage.js";
import { isTauriRuntime } from "../services/bridge/runtime";
import { addTab, selectTab, tabs } from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";

export type HistoryDisplayMode = "popup" | "tab";
export const HISTORY_URL = "spark://history";
const KEY = "spark_history_display";

export const historyDisplayMode = writable<HistoryDisplayMode>(safeStorage.getItem(KEY) === "tab" ? "tab" : "popup");
if (typeof window !== "undefined") window.addEventListener("storage", (event) => {
  if (event.key === KEY) historyDisplayMode.set(event.newValue === "tab" ? "tab" : "popup");
});

export function setHistoryDisplayMode(mode: HistoryDisplayMode): void {
  safeStorage.setItem(KEY, mode);
  historyDisplayMode.set(mode);
}

/** History opens in a tab (the panel is used in the browser preview, which has no tabs of its own). */
export const historyOpensInTab = (): boolean => get(historyDisplayMode) === "tab" && isTauriRuntime();

/** Shows the history tab of this workspace, opening one when there is none. */
export async function openHistoryTab(): Promise<void> {
  const workspace = get(activeWorkspaceId);
  const existing = get(tabs).find((tab) => tab.url === HISTORY_URL && !tab.is_ephemeral && (!tab.workspace_id || tab.workspace_id === workspace));
  if (existing) await selectTab(existing.id);
  else await addTab(HISTORY_URL);
}
