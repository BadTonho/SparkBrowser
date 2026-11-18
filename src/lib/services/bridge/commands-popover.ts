import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { PanelPresentation, PanelViewAction } from "../panel-presentation";

export interface CommandsPopoverEvent { sessionId: number; sequence: number; action: PanelViewAction }
let queue: Promise<void> = Promise.resolve();
export function setCommandsPopover(visible: boolean, sessionId: number, presentation?: PanelPresentation, restoreFocus = false) {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_commands_popover", { visible, sessionId, presentation: presentation ?? null, restoreFocus }));
  return queue;
}
export const onCommandsPopoverAction = (callback: (event: CommandsPopoverEvent) => void) =>
  isTauriRuntime() ? listen<CommandsPopoverEvent>("commands-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
