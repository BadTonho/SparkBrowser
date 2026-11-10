import { surfaceCoordinator } from "../services/surface-coordinator.js";
import { get, writable } from "svelte/store";
import { tabs } from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { setPrivacyPopover } from "../services/bridge/privacy-popover";
import { privacyError } from "./privacyStore";
import { describeError } from "../services/user-errors.js";

interface Session { id: number; tabId: string; url: string; workspaceId: string; anchor: HTMLButtonElement }
export const privacyPopover = writable<Session | null>(null);
let revision = 0;
export async function openPrivacy(anchor: HTMLButtonElement) {
  if (get(privacyPopover)) return closePrivacy(true);
  const id = ++revision;
  const currentClaim = await surfaceCoordinator.claim("privacy");
  if (!currentClaim()) return;
  const tab = get(tabs).find(tab => tab.is_active);
  if (!tab || id !== revision || !anchor.isConnected) return;
  privacyError.set("");
  privacyPopover.set({ id, tabId: tab.id, url: tab.url, workspaceId: get(activeWorkspaceId), anchor });
}
export async function closePrivacy(restoreFocus = false) {
  ++revision;
  const session = get(privacyPopover);
  if (!session) return;
  privacyPopover.set(null);
  try {
    await setPrivacyPopover(false, session.id, undefined, restoreFocus);
    if (restoreFocus && !get(privacyPopover) && session.anchor.isConnected) session.anchor.focus({ preventScroll: true });
  } catch (error) { privacyError.set(describeError(error)); }
}

surfaceCoordinator.register("privacy", () => closePrivacy());
