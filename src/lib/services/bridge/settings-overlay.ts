import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { PanelPresentation, PanelViewAction } from "../panel-presentation";

let queue: Promise<void> = Promise.resolve();
export function setSettingsOverlay(visible: boolean, sessionId: number, presentation?: PanelPresentation, restoreFocus = false): Promise<void> {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_settings_overlay", { visible, sessionId, presentation: presentation ?? null, restoreFocus }));
  return queue;
}
export function onSettingsOverlayAction(callback: (event: { sessionId: number; sequence: number; action: PanelViewAction }) => void): Promise<UnlistenFn> {
  return isTauriRuntime() ? listen("settings-popover-action", event => callback(event.payload as { sessionId: number; sequence: number; action: PanelViewAction })) : Promise.resolve(() => {});
}
