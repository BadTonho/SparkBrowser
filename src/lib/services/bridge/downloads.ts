import { invoke } from "./runtime";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { DownloadItem, DownloadTask, DownloadVisual, DownloadVisualSource } from "../../types/browser";

import { isTauriRuntime } from "./runtime";

export async function fetchDownloads(): Promise<DownloadItem[]> {
  if(!isTauriRuntime()) return [];
  return await invoke<DownloadItem[]>("get_downloads");
}

export async function enqueueDownloadUrl(url: string, expectedSha256: string | null = null): Promise<DownloadTask | null> {
  return await invoke<DownloadTask | null>("enqueue_download", { url, expectedSha256 });
}

export async function fetchDownloadTasks(): Promise<DownloadTask[]> {
  return isTauriRuntime() ? invoke<DownloadTask[]>("get_download_tasks") : [];
}
export const fetchDownloadVisual = (source: DownloadVisualSource): Promise<DownloadVisual> => isTauriRuntime()
  ? invoke("get_download_visual", { source }) : Promise.resolve({ size_bytes: null, thumbnail_png: null });
export async function cancelDownload(taskId: string): Promise<void> {
  await invoke("cancel_download", { taskId });
}
export async function onDownloadTaskUpdated(callback: (task: DownloadTask) => void): Promise<UnlistenFn> {
  return isTauriRuntime() ? listen<DownloadTask>("download-task-updated", event => callback(event.payload)) : () => { };
}
export async function onDownloadsUpdated(callback: () => void): Promise<UnlistenFn> {
  return isTauriRuntime() ? listen("downloads-updated", callback) : () => { };
}

export async function clearAllDownloads(): Promise<string[]> {
  if(!isTauriRuntime()) return [];
  return invoke<string[]>("clear_all_downloads");
}

export const controlDownload = (taskId: string, action: "pause" | "resume" | "restart" | "cancel" | "verify") => invoke<void>("control_download", { taskId, action });
export const setDownloadChecksum = (taskId: string, checksum: string | null) => invoke<void>("set_download_checksum", { taskId, checksum });
