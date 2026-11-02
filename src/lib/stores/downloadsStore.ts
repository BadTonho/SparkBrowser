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
const clearedTasks = new Set<string>();
let taskSequence = 0;
const changedAt = new Map<string, number>();

function applyTask(task: DownloadTask): void {
  if (clearedTasks.has(task.id)) return;
  changedAt.set(task.id, ++taskSequence);
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
  const startedAt = taskSequence;
  const [list, tasks] = await Promise.all([fetchDownloads(), fetchDownloadTasks()]);
  if (revision !== loadRevision) return;
  downloads.set(list);
  const ids = new Set(tasks.map(task => task.id));
  downloadTasks.update(current => {
    const fetched = tasks.map(task => {
      const live = current.find(item => item.id === task.id);
      return live && live.revision > task.revision ? live : task;
    });
    const newer = current.filter(task =>
      !ids.has(task.id) && (changedAt.get(task.id) ?? 0) > startedAt);
    return [...fetched, ...newer].filter(task => !clearedTasks.has(task.id));
  });
}

export async function startDownloadFromUrl(url: string, expectedSha256: string | null = null): Promise<void> {
  const trimmed = url.trim();
  if (!trimmed) return;

  if (!isTauriRuntime()) throw new Error("Downloads require the desktop application.");

  applyTask(await enqueueDownloadUrl(trimmed, expectedSha256));
}

export async function wipeDownloads(): Promise<void> {
  ++loadRevision;
  const removed = await clearAllDownloads();
  removed.forEach(id => clearedTasks.add(id));
  downloadTasks.update(tasks => tasks.filter(task => !clearedTasks.has(task.id)));
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

