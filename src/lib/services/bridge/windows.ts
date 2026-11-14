import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { invoke, isTauriRuntime } from "./runtime";

/** The id of this browser window: `main`, or `window-<n>` for the others. */
export function currentWindowId(): string {
  if (!isTauriRuntime()) return "main";
  try { return getCurrentWebview().label; } catch { return "main"; }
}

/** Opens another browser window with a new-tab page. */
export async function openBrowserWindow(): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<string>("open_browser_window");
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
