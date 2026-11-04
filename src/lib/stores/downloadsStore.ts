import { writable, get, derived } from "svelte/store";
import { tick } from "svelte";
import { t } from "../i18n";
import { summarizeDownloads } from "../services/download-progress";
import { downloadNotices, observeDownloadNotices, acknowledgeDownloadNotices } from "./downloadNoticesStore";
export { acknowledgeDownloadNotices } from "./downloadNoticesStore";
import { describeError } from "../services/user-errors.js";
import { notifyOpening, settleOverlayVisibility } from "../services/bridge/overlays";
import { setDownloadsPopover } from "../services/bridge/downloads-popover";
import { isZenMode } from "./settingsStore";
import type { DownloadItem, DownloadTask } from "../types/browser";
import {
  fetchDownloads,
  enqueueDownloadUrl,
  clearAllDownloads,
  isTauriRuntime,
  fetchDownloadTasks,
  onDownloadTaskUpdated,
  onDownloadsUpdated,
} from "../services/tauri-bridge";

export const downloads = writable<DownloadItem[]>([]);
export const downloadTasks = writable<DownloadTask[]>([]);
export const isDownloadsDrawerOpen = writable<boolean>(false);
export const downloadsSession = writable<{ id: number; anchor: HTMLButtonElement } | null>(null);
export const downloadsLoadError = writable("");
export const downloadProgress = derived([downloadTasks, downloadNotices], ([tasks, notices]) => summarizeDownloads(tasks, notices));
export const downloadProgressText = derived([downloadProgress, t], ([progress, tr]) => {
  if (progress.kind === "idle") return "";
  const key = progress.kind === "error" ? "downloads.progress_error" : progress.kind === "completed" ? "downloads.progress_completed" : progress.kind === "paused" ? "downloads.progress_paused" : progress.percent === null ? "downloads.progress_pending" : "downloads.progress_known";
  return tr(key, { count: String(progress.count), percent: String(Math.round(progress.percent ?? 0)) });
});
let openRevision = 0;
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
    observeDownloadNotices([task], "event");
    return [task, ...tasks.filter(item => item.id !== task.id)].sort((a, b) => b.id.localeCompare(a.id)).slice(0, 164);
  });
}

export function initializeDownloadEvents(): () => void {
  let disposed = false;
  const stops: (() => void)[] = [];
  const subscriptions = [onDownloadTaskUpdated(task => { if (!disposed) applyTask(task); }), onDownloadsUpdated(() => { if (!disposed) void loadDownloads().catch(() => {}); })];
  void Promise.all(subscriptions.map(promise => promise.then(stop => { if (disposed) stop(); else stops.push(stop); })))
    .then(() => { if (!disposed) return loadDownloads(); }).catch(() => {});
  return () => { disposed = true; stops.forEach(stop => stop()); };
}

export async function loadDownloads(): Promise<void> {
  const revision = ++loadRevision;
  const startedAt = taskSequence;
  let list: DownloadItem[], tasks: DownloadTask[];
  try { [list, tasks] = await Promise.all([fetchDownloads(), fetchDownloadTasks()]); }
  catch (error) { if (revision === loadRevision) downloadsLoadError.set(describeError(error)); throw error; }
  if (revision !== loadRevision) return;
  downloadsLoadError.set("");
  downloads.set(list);
  const ids = new Set(tasks.map(task => task.id));
  downloadTasks.update(current => {
    const fetched = tasks.map(task => {
      const live = current.find(item => item.id === task.id);
      return live && live.revision > task.revision ? live : task;
    });
    const newer = current.filter(task =>
      !ids.has(task.id) && (changedAt.get(task.id) ?? 0) > startedAt);
    const merged = [...fetched, ...newer].filter(task => !clearedTasks.has(task.id));
    observeDownloadNotices(merged, "snapshot");
    return merged;
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
  observeDownloadNotices(get(downloadTasks), "snapshot");
  ++loadRevision;
  downloads.set([]);
}

export async function openDownloadsDrawer(anchor?: HTMLButtonElement): Promise<void> {
  if (get(isDownloadsDrawerOpen)) { acknowledgeDownloadNotices(); if (anchor) await closeDownloadsDrawer(true); return; }
  const request = ++openRevision;
  isZenMode.set(false);
  await tick();
  if (request !== openRevision) return;
  const visible = (button: HTMLButtonElement | null | undefined) => button?.isConnected && button.getClientRects().length ? button : undefined;
  const reference = visible(anchor) ?? visible(document.querySelector<HTMLButtonElement>('.titlebar-actions [data-action="downloads"]')) ?? visible(document.querySelector<HTMLButtonElement>('[data-testid="open-titlebar-menu"]'));
  if (!reference) return;
  notifyOpening("downloads");
  await settleOverlayVisibility();
  if (request !== openRevision || !visible(reference)) return;
  acknowledgeDownloadNotices();
  downloadsSession.set({ id: request, anchor: reference });
  isDownloadsDrawerOpen.set(true);
  void loadDownloads().catch(() => {});
}

export async function closeDownloadsDrawer(restoreFocus: unknown = false): Promise<void> {
  ++openRevision;
  const session = get(downloadsSession);
  downloadsSession.set(null);
  isDownloadsDrawerOpen.set(false);
  if (!session) return;
  await setDownloadsPopover(false, session.id, undefined, restoreFocus === true);
  if (restoreFocus === true && !get(isDownloadsDrawerOpen) && session.anchor.isConnected && session.anchor.getClientRects().length) session.anchor.focus({ preventScroll: true });
}

export async function toggleDownloadsDrawer(): Promise<void> {
  if (get(isDownloadsDrawerOpen)) {
    acknowledgeDownloadNotices();
    await closeDownloadsDrawer(true);
  } else {
    await openDownloadsDrawer();
  }
}

