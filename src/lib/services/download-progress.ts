import type { DownloadTask } from "../types/browser";

export interface DownloadProgress {
  kind: "idle" | "determinate" | "indeterminate" | "paused";
  percent: number | null;
  count: number;
}
function percent(tasks: DownloadTask[]): number | null {
  if (tasks.some(task => !Number.isFinite(task.total) || !task.total || task.total <= 0)) return null;
  const total = tasks.reduce((sum, task) => sum + task.total!, 0);
  const received = tasks.reduce((sum, task) => sum + Math.min(task.total!, Math.max(0, Number.isFinite(task.received) ? task.received : 0)), 0);
  return Math.max(0, Math.min(100, received / total * 100));
}
export function summarizeDownloads(tasks: DownloadTask[]): DownloadProgress {
  const running = tasks.filter(task => task.status === "Downloading");
  if (running.length) {
    const value = percent(running);
    return { kind: value === null ? "indeterminate" : "determinate", percent: value, count: running.length };
  }
  const processing = tasks.filter(task => ["Queued", "Verifying", "Finalizing"].includes(task.status));
  if (processing.length) return { kind: "indeterminate", percent: null, count: processing.length };
  const paused = tasks.filter(task => ["Paused", "Pausing"].includes(task.status));
  if (paused.length) return { kind: "paused", percent: percent(paused), count: paused.length };
  return { kind: "idle", percent: null, count: 0 };
}
