import { surfaceCoordinator } from "../services/surface-coordinator.js";
import { derived, get, writable } from "svelte/store";
import { tabs } from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { mediaState, mediaError, refreshMedia, flushVolume } from "./mediaStore";
import { setMediaPopover } from "../services/bridge/media-popover";
import { describeError } from "../services/user-errors.js";

interface MediaPopoverSession {
  id: number;
  tabId: string;
  documentKey: string;
  url: string;
  workspaceId: string;
  anchor: HTMLButtonElement;
}

export const mediaPopover = writable<MediaPopoverSession | null>(null);
export const isMediaOpen = derived(mediaPopover, session => session !== null);
let revision = 0;

export async function openMedia(anchor: HTMLButtonElement) {
  if (get(mediaPopover)) return closeMedia(true);
  const request = ++revision;
  const currentClaim = await surfaceCoordinator.claim("media");
  if (!currentClaim()) return;
  const active = get(tabs).find(tab => tab.is_active);
  const workspaceId = get(activeWorkspaceId);
  if (!active) return;
  mediaError.set("");
  const residentMediaTabs = get(tabs).filter(tab => !tab.is_hibernated);
  await Promise.all(residentMediaTabs.map(tab => refreshMedia(tab.id, true)));
  const current = get(tabs).find(tab => tab.is_active);
  if (!currentClaim() || request !== revision || !anchor.isConnected || current?.id !== active.id || current.url !== active.url || workspaceId !== get(activeWorkspaceId)) return;
  mediaPopover.set({ id: request, tabId: active.id, documentKey: get(mediaState).tabs.find(tab => tab.tab_id === active.id)?.document_key ?? "", url: active.url, workspaceId, anchor });
}

export async function closeMedia(restoreFocus = false) {
  ++revision;
  const session = get(mediaPopover);
  if (!session) return;
  mediaPopover.set(null);
  try {
    await setMediaPopover(false, session.id, undefined, restoreFocus);
    if (restoreFocus && !get(mediaPopover) && session.anchor.isConnected) session.anchor.focus({ preventScroll: true });
  } catch (error) { mediaError.set(describeError(error)); }
  finally { await flushVolume(); }
}

surfaceCoordinator.register("media", () => closeMedia());
