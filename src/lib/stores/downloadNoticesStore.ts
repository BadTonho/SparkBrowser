import { writable } from "svelte/store";
import { safeStorage } from "../services/storage.js";
import { createDownloadNotices } from "../services/download-notices";
import type { DownloadTask } from "../types/browser";

const ledger = createDownloadNotices(safeStorage);
export const downloadNotices = writable(ledger.summary());
export function observeDownloadNotices(tasks: readonly DownloadTask[], source: "event" | "snapshot") {
  downloadNotices.set(ledger.observe(tasks, source));
}
export function acknowledgeDownloadNotices() {
  downloadNotices.set(ledger.acknowledge());
}
