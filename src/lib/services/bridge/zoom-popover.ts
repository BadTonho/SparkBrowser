import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";

export interface ZoomPopoverPresentation {
  tabId: string; url: string; origin: string;
  anchor: { left: number; top: number; width: number; height: number };
  factor: number; levels: number[]; pending: boolean; error: string;
  label: string; inLabel: string; outLabel: string; resetLabel: string; locale: string;
  theme: { background: string; border: string; accent: string };
}
export interface ZoomPopoverAction {
  sessionId: number; tabId: string; url: string; origin: string;
  action: "zoom" | "dismiss"; value: number | null; reason: string;
}
let queue: Promise<void> = Promise.resolve();
export function setZoomPopover(visible: boolean, sessionId: number, presentation?: ZoomPopoverPresentation, restoreFocus = false) {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_zoom_popover", { visible, sessionId, presentation: presentation ?? null, restoreFocus }));
  return queue;
}
export const onZoomPopoverAction = (callback: (action: ZoomPopoverAction) => void) =>
  isTauriRuntime() ? listen<ZoomPopoverAction>("zoom-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
