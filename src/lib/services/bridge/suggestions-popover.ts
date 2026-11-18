import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { PanelPresentation, PanelViewAction } from "../panel-presentation";

export interface SuggestionsPopoverEvent { sessionId: number; sequence: number; action: PanelViewAction }
let queue: Promise<void> = Promise.resolve();
export function setSuggestionsPopover(visible: boolean, sessionId: number, presentation?: PanelPresentation, restoreFocus = false) {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_suggestions_popover", { visible, sessionId, presentation: presentation ?? null, restoreFocus }));
  return queue;
}
export const onSuggestionsPopoverAction = (callback: (event: SuggestionsPopoverEvent) => void) =>
  isTauriRuntime() ? listen<SuggestionsPopoverEvent>("suggestions-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
