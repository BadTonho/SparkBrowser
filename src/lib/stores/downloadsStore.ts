import { writable, get } from "svelte/store";
import type { DownloadItem } from "../types/browser";
import {
  fetchDownloads,
  enqueueDownloadUrl,
  clearAllDownloads,
  setOverlayVisibility,
  isTauriRuntime,
} from "../services/tauri-bridge";

export const downloads = writable<DownloadItem[]>([]);
export const isDownloadsDrawerOpen = writable<boolean>(false);

export async function loadDownloads(): Promise<void> {
  const list = await fetchDownloads();
  downloads.set(list);
}

export async function startDownloadFromUrl(url: string): Promise<void> {
  const trimmed = url.trim();
  if (!trimmed) return;

  if (!isTauriRuntime()) {
    const filename = trimmed.split("/").pop() || "downloaded-file.bin";
    downloads.update((list) => [
      {
        id: Date.now(),
        file_name: filename,
        url: trimmed,
        save_path: `~/Downloads/${filename}`,
        status: "Completed",
        created_at: Math.floor(Date.now() / 1000),
      },
      ...list,
    ]);
    return;
  }

  const updated = await enqueueDownloadUrl(trimmed);
  downloads.set(updated);
}

export async function wipeDownloads(): Promise<void> {
  await clearAllDownloads();
  downloads.set([]);
}

export async function openDownloadsDrawer(): Promise<void> {
  await loadDownloads();
  isDownloadsDrawerOpen.set(true);
  await setOverlayVisibility(true, "downloads");
}

export async function closeDownloadsDrawer(): Promise<void> {
  isDownloadsDrawerOpen.set(false);
  await setOverlayVisibility(false, "downloads");
}

export async function toggleDownloadsDrawer(): Promise<void> {
  if (get(isDownloadsDrawerOpen)) {
    await closeDownloadsDrawer();
  } else {
    await openDownloadsDrawer();
  }
}

