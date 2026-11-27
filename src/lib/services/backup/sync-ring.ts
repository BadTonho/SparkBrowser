/**
 * The ring around the sync card in Data & Backup, drawn like the download button's: green while a
 * sync runs (filling), green and full for a moment when it finished, yellow while sync is paused and
 * red after an error.
 */
export type SyncRing = "idle" | "syncing" | "error" | "done" | "paused";

export interface SyncRingInput {
  mode: "disabled" | "google_drive_web" | "folder" | "direct_webdav";
  /** "Send automatically" is on. */
  autoSave: boolean;
  /** Syncs in progress, manual or automatic. */
  running: number;
  /** The last sync ended without an error a moment ago. */
  finished: boolean;
  /** The last sync problem, if any. */
  problem: string | null;
  /** Google Drive waits for an account to be chosen or linked; its problem is guidance, not an error. */
  waitingForAccount: boolean;
  /** The person paused the automatic sync for a while. */
  pausedByUser?: boolean;
}

/** What the ring shows. A running sync wins over an older problem, which wins over the rest. */
export function summarizeSyncRing(input: SyncRingInput): SyncRing {
  if (input.mode === "disabled") return "idle";
  if (input.running > 0) return "syncing";
  if (input.problem && !input.waitingForAccount) return "error";
  if (input.finished) return "done";
  if (input.waitingForAccount || input.pausedByUser || !input.autoSave) return "paused";
  return "idle";
}
