import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { PanelPresentation, PanelViewAction } from "../panel-presentation";

export interface DownloadsPopoverEvent { sessionId: number; sequence: number; action: PanelViewAction }
let queue: Promise<void> = Promise.resolve();
export function setDownloadsPopover(visible: boolean, sessionId: number, presentation?: PanelPresentation, restoreFocus = false) {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_downloads_popover", { visible, sessionId, presentation: presentation ?? null, restoreFocus }));
  return queue;
}
export const onDownloadsPopoverAction = (callback: (event: DownloadsPopoverEvent) => void) =>
  isTauriRuntime() ? listen<DownloadsPopoverEvent>("downloads-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
