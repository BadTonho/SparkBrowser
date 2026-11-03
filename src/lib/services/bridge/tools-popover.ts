import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { ToolsPresentation, ToolsViewAction } from "../tools-presentation";

export interface ToolsPopoverEvent { sessionId: number; sequence: number; action: ToolsViewAction }
let queue: Promise<void> = Promise.resolve();
export function setToolsPopover(visible: boolean, sessionId: number, presentation?: ToolsPresentation, restoreFocus = false) {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_tools_popover", { visible, sessionId, presentation: presentation ?? null, restoreFocus }));
  return queue;
}
export const onToolsPopoverAction = (callback: (event: ToolsPopoverEvent) => void) =>
  isTauriRuntime() ? listen<ToolsPopoverEvent>("tools-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
