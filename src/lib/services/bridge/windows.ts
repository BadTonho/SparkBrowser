import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { invoke, isTauriRuntime } from "./runtime";

/** The id of this browser window: `main`, or `window-<n>` for the others. */
export function currentWindowId(): string {
  if (!isTauriRuntime()) return "main";
  try { return getCurrentWebview().label; } catch { return "main"; }
}

/** Opens another browser window with a new-tab page and returns its id. */
export async function openBrowserWindow(): Promise<string | null> {
  if (!isTauriRuntime()) return null;
  return await invoke<string>("open_browser_window");
}

export interface BrowserWindowSummary {
  id: string;
  tab_count: number;
}

/** The open browser windows. */
export async function getBrowserWindows(): Promise<BrowserWindowSummary[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<BrowserWindowSummary[]>("get_browser_windows");
}

/** Sends a tab to another window; the page reloads there. */
export async function moveTabToWindow(tabId: string, windowId: string): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<void>("move_tab_to_window", { tabId, windowId });
}

/** Moves a tab into a new window placed at the screen position (physical pixels) where it was dropped. */
export async function detachTabToNewWindow(tabId: string, x: number, y: number): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<void>("detach_tab_to_new_window", { tabId, x, y });
}

/** A readable name for a window id: `main` is window 1, `window-<n>` is window n. */
export function windowNumber(id: string): number {
  const match = /^window-(\d+)$/.exec(id);
  return match ? Number(match[1]) : 1;
}

export async function minimizeWindow(): Promise<void> {
  if(!isTauriRuntime()) return;
  await getCurrentWindow().minimize();
}

export async function toggleMaximizeWindow(): Promise<void> {
  if(!isTauriRuntime()) return;
  await getCurrentWindow().toggleMaximize();
}

export async function closeWindow(): Promise<void> {
  if(!isTauriRuntime()) return;
  await getCurrentWindow().close();
}
