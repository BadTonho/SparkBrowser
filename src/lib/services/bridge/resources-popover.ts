import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { PanelPresentation, PanelViewAction } from "../panel-presentation";

export interface ResourcesPopoverEvent { sessionId: number; sequence: number; action: PanelViewAction }
let queue: Promise<void> = Promise.resolve();
export function setResourcesPopover(visible: boolean, sessionId: number, presentation?: PanelPresentation, restoreFocus = false): Promise<void> {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_resources_popover", { visible, sessionId, presentation: presentation ?? null, restoreFocus }));
  return queue;
}
export const onResourcesPopoverAction = (callback: (event: ResourcesPopoverEvent) => void): Promise<UnlistenFn> =>
  isTauriRuntime() ? listen<ResourcesPopoverEvent>("resources-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
