import { get, writable } from "svelte/store";
import { tick } from "svelte";
import { notifyOpening } from "../services/bridge/overlays";
import { setVideoDownloadPopover } from "../services/bridge/video-download";

export interface VideoDownloadSession { id: number; anchor: HTMLButtonElement }

export const isVideoDownloadOpen = writable(false);
export const videoDownloadSession = writable<VideoDownloadSession | null>(null);
let revision = 0;

export async function openVideoDownload(anchor?: HTMLButtonElement): Promise<void> {
  if (get(isVideoDownloadOpen)) {
    await closeVideoDownload(true);
    return;
  }
  const request = ++revision;
  await tick();
  if (request !== revision) return;
  const visible = (button: HTMLButtonElement | null | undefined) => button?.isConnected && button.getClientRects().length ? button : undefined;
  const reference = visible(anchor) ?? visible(document.querySelector<HTMLButtonElement>('.titlebar-actions [data-action="video"]'));
  if (!reference) return;
  notifyOpening("video-download");
  if (request !== revision || get(isVideoDownloadOpen)) return;
  videoDownloadSession.set({ id: request, anchor: reference });
  isVideoDownloadOpen.set(true);
}

export async function closeVideoDownload(restoreFocus = false): Promise<void> {
  ++revision;
  const session = get(videoDownloadSession);
  if (!session) return;
  videoDownloadSession.set(null);
  isVideoDownloadOpen.set(false);
  await setVideoDownloadPopover(false, session.id, undefined, restoreFocus);
  if (restoreFocus && !get(videoDownloadSession) && session.anchor.isConnected) session.anchor.focus({ preventScroll: true });
}
