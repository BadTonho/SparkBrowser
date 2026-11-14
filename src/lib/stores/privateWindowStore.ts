import { writable } from "svelte/store";
import { fetchPrivateWindowState } from "../services/bridge/private-window";

export const isPrivateWindow = writable(false);

/**
 * Asks the native side whether this is a private window and marks the document, so styles (and the
 * badge) can make it obvious. The window has no native title bar, so nothing else would tell.
 */
export function initializePrivateWindow(): () => void {
  let stopped = false;
  void fetchPrivateWindowState()
    .then((value) => {
      if (stopped) return;
      isPrivateWindow.set(value);
      if (value) document.documentElement.dataset.private = "true";
    })
    .catch(() => {});
  return () => {
    stopped = true;
  };
}
