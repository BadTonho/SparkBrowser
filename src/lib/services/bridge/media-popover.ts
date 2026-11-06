import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";

export interface MediaPopoverPresentation {
  tabId: string;
  documentKey: string;
  anchor: { left: number; top: number; width: number; height: number };
  volume: number;
  disabled: boolean;
  label: string;
  locale: string;
  theme: { background: string; border: string; accent: string };
  items: Array<{ tabId: string; tabTitle: string; workspaceName: string; frameLabel: string; playing: boolean; audible: boolean; muted: boolean; muteAvailable: boolean }>;
  frameTrackingSupported: boolean;
  labels: { title: string; playing: string; audible: string; mute: string; unmute: string; noMedia: string; frameLimit: string };
}

export interface MediaPopoverAction {
  sessionId: number;
  tabId: string;
  documentKey: string;
  action: "volume" | "dismiss" | "select" | "mute";
  value: number | null;
  reason: string;
  targetTabId: string | null;
}

let syncQueue: Promise<void> = Promise.resolve();
export function setMediaPopover(visible: boolean, sessionId: number, presentation?: MediaPopoverPresentation, restoreFocus = false): Promise<void> {
  if (!isTauriRuntime()) return Promise.resolve();
  syncQueue = syncQueue.catch(() => undefined).then(() =>
    invoke<void>("set_media_popover", { visible, sessionId, presentation: presentation ?? null, restoreFocus })
  );
  return syncQueue;
}

export const onMediaPopoverAction = (callback: (action: MediaPopoverAction) => void) =>
  isTauriRuntime() ? listen<MediaPopoverAction>("media-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
