import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";

export interface PrivacyPopoverPresentation {
  tabId: string; url: string;
  anchor: { left: number; top: number; width: number; height: number };
  shield: boolean; autoSkip: boolean; busy: boolean; error: string;
  blocked: string; adblockLabel: string; autoSkipLabel: string; description: string;
  label: string; locale: string;
  theme: { background: string; border: string; accent: string };
}
export interface PrivacyPopoverAction {
  sessionId: number; tabId: string; url: string;
  action: "shield" | "auto-skip" | "dismiss"; value: boolean | null; reason: string;
}
let queue: Promise<void> = Promise.resolve();
export function setPrivacyPopover(visible: boolean, sessionId: number, presentation?: PrivacyPopoverPresentation, restoreFocus = false) {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_privacy_popover", { visible, sessionId, presentation: presentation ?? null, restoreFocus }));
  return queue;
}
export const onPrivacyPopoverAction = (callback: (action: PrivacyPopoverAction) => void) =>
  isTauriRuntime() ? listen<PrivacyPopoverAction>("privacy-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
