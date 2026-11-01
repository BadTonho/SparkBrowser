import { writable, get } from "svelte/store";
import type { DownloadItem, DownloadTask } from "../types/browser";
import {
  fetchDownloads,
  enqueueDownloadUrl,
  clearAllDownloads,
  setOverlayVisibility,
  isTauriRuntime,
  fetchDownloadTasks,
  onDownloadTaskUpdated,
  onDownloadsUpdated,
} from "../services/tauri-bridge";

export const downloads = writable<DownloadItem[]>([]);
export const downloadTasks = writable<DownloadTask[]>([]);
export const isDownloadsDrawerOpen = writable<boolean>(false);
let loadRevision = 0;

function applyTask(task: DownloadTask): void {
  downloadTasks.update(tasks => {
    const previous = tasks.find(item => item.id === task.id);
    if (previous && previous.revision >= task.revision) return tasks;
    return [task, ...tasks.filter(item => item.id !== task.id)].sort((a, b) => b.id.localeCompare(a.id)).slice(0, 164);
  });
}

export function initializeDownloadEvents(): () => void {
  let disposed = false;
  const stops: (() => void)[] = [];
  for (const promise of [onDownloadTaskUpdated(applyTask), onDownloadsUpdated(() => void loadDownloads())]) {
    void promise.then(stop => { if (disposed) stop(); else stops.push(stop); });
  }
  return () => { disposed = true; stops.forEach(stop => stop()); };
}

export async function loadDownloads(): Promise<void> {
  const revision = ++loadRevision;
  const [list, tasks] = await Promise.all([fetchDownloads(), fetchDownloadTasks()]);
  if (revision !== loadRevision) return;
  downloads.set(list);
  tasks.forEach(applyTask);
}

export async function startDownloadFromUrl(url: string): Promise<void> {
  const trimmed = url.trim();
  if (!trimmed) return;

  if (!isTauriRuntime()) throw new Error("Downloads require the desktop application.");

  applyTask(await enqueueDownloadUrl(trimmed));
}

export async function wipeDownloads(): Promise<void> {
  ++loadRevision;
  await clearAllDownloads();
  ++loadRevision;
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

