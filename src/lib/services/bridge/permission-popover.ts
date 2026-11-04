import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";

export interface PermissionPopoverPresentation {
  mode: "settings" | "request";
  tabId: string;
  url: string;
  origin: string;
  anchor: { left: number; top: number; width: number; height: number };
  request: null | { id: string; tabId: string; origin: string; topOrigin: string; frameOrigin: string | null; kind: string; displayOrigin: string; details: string[] };
  pendingCount: number;
  settings: Record<string, string>;
  labels: Record<string, unknown>;
  theme: Record<string, string>;
  locale: string;
  error: string;
  busy: boolean;
  confirmingReload: boolean;
}
export interface PermissionPopoverAction {
  sessionId: number; tabId: string; url: string; origin: string; requestId: string | null;
  action: string; kind: string; value: string; decision: string; reason: string;
}
let queue: Promise<void> = Promise.resolve();
export function setPermissionPopover(visible: boolean, sessionId: number, presentation?: PermissionPopoverPresentation) {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_permission_popover", { visible, sessionId, presentation: presentation ?? null }));
  return queue;
}
export const onPermissionPopoverAction = (callback: (action: PermissionPopoverAction) => void) =>
  isTauriRuntime() ? listen<PermissionPopoverAction>("site-permission-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
