import { writable } from "svelte/store";
import { chooseDownloadLocation, fetchDownloadLocation, resetDownloadLocation, type DownloadLocation } from "../services/bridge/downloads";
import { logIgnored } from "../services/ignored-error";
import { safeStorage } from "../services/storage.js";

/** Where downloads go when Spark does not ask (null until the backend answered). */
export const downloadLocation = writable<DownloadLocation | null>(null);
/** The last attempt to read or change the folder failed. */
export const downloadLocationError = writable(false);

const SHOW_WHEN_DONE_KEY = "spark_show_downloads_when_done";
/** Open the downloads panel when a download finishes. On unless the person turned it off. */
export const showDownloadsWhenDone = writable(safeStorage.getItem(SHOW_WHEN_DONE_KEY) !== "false");
export function setShowDownloadsWhenDone(enabled: boolean): void {
  showDownloadsWhenDone.set(enabled);
  safeStorage.setItem(SHOW_WHEN_DONE_KEY, String(enabled));
}

export async function loadDownloadLocation(): Promise<void> {
  try { downloadLocation.set(await fetchDownloadLocation()); downloadLocationError.set(false); }
  catch (error) { logIgnored(error); downloadLocationError.set(true); }
}

/** Opens the folder picker; a cancelled picker changes nothing. */
export async function pickDownloadLocation(): Promise<void> {
  try {
    const chosen = await chooseDownloadLocation();
    if (chosen) downloadLocation.set(chosen);
    downloadLocationError.set(false);
  } catch (error) { logIgnored(error); downloadLocationError.set(true); }
}

export async function useDefaultDownloadLocation(): Promise<void> {
  try { downloadLocation.set(await resetDownloadLocation()); downloadLocationError.set(false); }
  catch (error) { logIgnored(error); downloadLocationError.set(true); }
}

// Read from storage each time, so a change made in another window (or by a test) counts at once.
export const isShowingDownloadsWhenDone = () => safeStorage.getItem(SHOW_WHEN_DONE_KEY) !== "false";
