import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { PanelPresentation, PanelViewAction } from "../panel-presentation";
import { errorDetails } from "../user-errors.js";

let queue: Promise<void> = Promise.resolve();
export function setSettingsOverlay(visible: boolean, sessionId: number, presentation?: PanelPresentation, restoreFocus = false, animateClose = false): Promise<void> {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_settings_overlay", { visible, sessionId, presentation: presentation ?? null, restoreFocus, animateClose }));
  return queue;
}
export function onSettingsOverlayAction(callback: (event: { sessionId: number; sequence: number; action: PanelViewAction }) => void): Promise<UnlistenFn> {
  return isTauriRuntime() ? listen("settings-popover-action", event => callback(event.payload as { sessionId: number; sequence: number; action: PanelViewAction })) : Promise.resolve(() => {});
}

export async function setSettingsTab(tabId: string, visible: boolean, sessionId: number, presentation?: PanelPresentation, active: () => boolean = () => true): Promise<void> {
  if (!visible) return;
  let lastError: unknown;
  for (let attempt = 0; attempt < 80; attempt++) {
    if (!active()) return;
    try {
      await invoke<void>("set_settings_overlay", { visible: true, sessionId, presentation: presentation ?? null, restoreFocus: false, animateClose: false, tabId });
      return;
    } catch (error) {
      lastError = error;
      if (errorDetails(error).code !== "settings_tab.not_ready") throw error;
      await new Promise(resolve => setTimeout(resolve, 100));
    }
  }
  if (active()) throw lastError;
}
export function onSettingsTabAction(tabId: string, callback: (event: { sessionId: number; sequence: number; action: PanelViewAction }) => void): Promise<UnlistenFn> {
  return listen<{ tabId: string; sessionId: number; sequence: number; action: PanelViewAction }>("settings-tab-action", event => {
    if (event.payload.tabId === tabId) callback(event.payload);
  });
}
export function onSettingsTabLoaded(callback: (tabId: string) => void): Promise<UnlistenFn> {
  return isTauriRuntime() ? listen<string>("settings-tab-loaded", event => callback(event.payload)) : Promise.resolve(() => {});
}
