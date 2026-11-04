import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { PanelPresentation, PanelViewAction } from "../panel-presentation";

export interface SettingsQuickPopoverEvent {
  sessionId: number;
  sequence: number;
  action: PanelViewAction;
}

let queue: Promise<void> = Promise.resolve();
export function setSettingsQuickPopover(visible: boolean, sessionId: number, presentation?: PanelPresentation, restoreFocus = false): Promise<void> {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_quick_settings_popover", { visible, sessionId, presentation: presentation ?? null, restoreFocus }));
  return queue;
}

export const onSettingsQuickPopoverAction = (callback: (event: SettingsQuickPopoverEvent) => void): Promise<UnlistenFn> =>
  isTauriRuntime() ? listen<SettingsQuickPopoverEvent>("quick-settings-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
