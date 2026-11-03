import { get, writable } from "svelte/store";
import { tabs } from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { tabLayout } from "./settingsStore";
import { zoomOrigin, zoomError, flushZoom } from "./zoomStore";
import { closeMedia } from "./mediaPopoverStore";
import { closePrivacy } from "./privacyPopoverStore";
import { setZoomPopover } from "../services/bridge/zoom-popover";
import { describeError } from "../services/user-errors.js";

interface Session { id: number; tabId: string; url: string; origin: string; workspaceId: string; layout: string; anchor: HTMLButtonElement }
export const zoomPopover = writable<Session | null>(null);
let revision = 0;
export async function openZoom(anchor: HTMLButtonElement) {
  if (get(zoomPopover)) return closeZoom(true);
  const id = ++revision;
  await closeMedia();
  await closePrivacy();
  const tab = get(tabs).find(tab => tab.is_active), origin = get(zoomOrigin);
  if (!tab || !origin || id !== revision || !anchor.isConnected) return;
  zoomPopover.set({ id, tabId: tab.id, url: tab.url, origin, workspaceId: get(activeWorkspaceId), layout: get(tabLayout), anchor });
}
export async function closeZoom(restoreFocus = false) {
  ++revision;
  const session = get(zoomPopover);
  if (!session) return;
  zoomPopover.set(null);
  try {
    await setZoomPopover(false, session.id, undefined, restoreFocus);
    if (restoreFocus && !get(zoomPopover) && session.anchor.isConnected && session.anchor.getClientRects().length) session.anchor.focus({ preventScroll: true });
  } catch (error) { zoomError.set(describeError(error)); }
  finally { await flushZoom(); }
}
