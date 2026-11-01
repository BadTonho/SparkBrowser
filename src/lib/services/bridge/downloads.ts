import { invoke } from "./runtime";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { DownloadItem, DownloadTask } from "../../types/browser";

import { isTauriRuntime } from "./runtime";

export async function fetchDownloads(): Promise<DownloadItem[]> {
  if(!isTauriRuntime()) return [];
  return await invoke<DownloadItem[]>("get_downloads");
}

export async function enqueueDownloadUrl(url: string): Promise<DownloadTask> {
  return await invoke<DownloadTask>("enqueue_download", { url });
}

export async function fetchDownloadTasks(): Promise<DownloadTask[]> {
  return isTauriRuntime() ? invoke<DownloadTask[]>("get_download_tasks") : [];
}
export async function cancelDownload(taskId: string): Promise<void> {
  await invoke("cancel_download", { taskId });
}
export async function onDownloadTaskUpdated(callback: (task: DownloadTask) => void): Promise<UnlistenFn> {
  return isTauriRuntime() ? listen<DownloadTask>("download-task-updated", event => callback(event.payload)) : () => { };
}
export async function onDownloadsUpdated(callback: () => void): Promise<UnlistenFn> {
  return isTauriRuntime() ? listen("downloads-updated", callback) : () => { };
}

export async function clearAllDownloads(): Promise<void> {
  if(!isTauriRuntime()) return;
  await invoke("clear_all_downloads");
}
