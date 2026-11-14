import type { DownloadTask } from "../types/browser";

export interface DownloadNotices { completed: number; error: number }
interface Entry { id: string; revision: number; status: string; notice: "completed" | "error" | null }
interface Storage { getItem(key: string): string | null; setItem(key: string, value: string): unknown }
const KEY = "spark_download_notices";
const LIMIT = 4096;
const statuses = new Set(["Queued", "Downloading", "Pausing", "Paused", "Interrupted", "RestartRequired", "Verifying", "VerificationFailed", "Finalizing", "Cancelling", "Completed", "Cancelled", "Failed"]);
const result = (status: string) => status === "Completed" ? "completed" : ["Failed", "VerificationFailed"].includes(status) ? "error" : null;

/** Stores only task identities and observed states, never file metadata or bridge credentials. */
export function createDownloadNotices(storage: Storage) {
  const entries = new Map<string, Entry>();
  let initialized = false;
  try {
    const raw = storage.getItem(KEY);
    if (raw && raw.length <= 1_000_000) {
      const saved = JSON.parse(raw);
      if (saved?.version === 1 && typeof saved.initialized === "boolean" && Array.isArray(saved.entries) && saved.entries.length <= LIMIT) {
        const restored = new Map<string, Entry>();
        for (const entry of saved.entries) {
          if (!entry || typeof entry.id !== "string" || !entry.id.length || entry.id.length > 128 || !Number.isSafeInteger(entry.revision) || entry.revision < 0 || !statuses.has(entry.status) ||
            ![null, "completed", "error"].includes(entry.notice) || restored.has(entry.id)) throw new Error("Invalid download notice preference");
          restored.set(entry.id, { id: entry.id, revision: entry.revision, status: entry.status, notice: entry.notice });
        }
        initialized = saved.initialized;
        for (const [id, entry] of restored) entries.set(id, entry);
      }
    }
  } catch { /* Invalid preferences start with an empty baseline. */ }
  function persist() {
    try { storage.setItem(KEY, JSON.stringify({ version: 1, initialized, entries: [...entries.values()] })); }
    catch { /* The in-memory ledger remains authoritative for this session. */ }
  }
  function summary(): DownloadNotices {
    let completed = 0, error = 0;
    for (const entry of entries.values()) {
      if (entry.notice === "completed") completed++;
      if (entry.notice === "error") error++;
    }
    return { completed, error };
  }
  return {
    summary,
    observe(tasks: readonly DownloadTask[], source: "event" | "snapshot") {
      let changed = false;
      if (source === "snapshot") {
        const ids = new Set(tasks.map(task => task.id));
        for (const id of entries.keys()) if (!ids.has(id)) { entries.delete(id); changed = true; }
      }
      for (const task of tasks) {
        const previous = entries.get(task.id);
        if (previous && previous.revision >= task.revision) continue;
        const terminal = result(task.status);
        const notice = terminal && (previous ? previous.status !== task.status : source === "event" || initialized) ? terminal : previous?.notice ?? null;
        changed ||= !previous || previous.status !== task.status || previous.notice !== notice;
        entries.set(task.id, { id: task.id, revision: task.revision, status: task.status, notice });
      }
      while (entries.size > LIMIT) { entries.delete(entries.keys().next().value!); changed = true; }
      if (source === "snapshot" && !initialized) { initialized = true; changed = true; }
      if (changed) persist();
      return summary();
    },
    acknowledge() {
      for (const entry of entries.values()) entry.notice = null;
      persist();
      return summary();
    },
  };
}
