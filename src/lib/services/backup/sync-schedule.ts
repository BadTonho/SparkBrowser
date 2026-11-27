/**
 * When the automatic sync sends: the chosen frequency, a pause, and whether there is anything new.
 * The store calls this on a one-minute tick; "auto" keeps the original behavior (a folder follows each
 * change, Google Drive and WebDAV send after a quiet moment and at most every minute).
 */
import type { SyncInterval, SyncOptions } from "../../types/backup";

type Mode = "disabled" | "google_drive_web" | "folder" | "direct_webdav";

const INTERVAL_MS: Record<Exclude<SyncInterval, "auto" | "on_exit">, number> = {
  five_minutes: 5 * 60_000,
  fifteen_minutes: 15 * 60_000,
};

/** The automatic sync is paused right now. */
export function syncPaused(options: Pick<SyncOptions, "paused_until">, nowMs: number): boolean {
  return options.paused_until !== null && options.paused_until * 1000 > nowMs;
}

/** Each change schedules a send after a short wait (only with the automatic frequency). */
export function followsEachChange(interval: SyncInterval): boolean {
  return interval === "auto";
}

/** Whether the one-minute tick should send now. */
export function tickShouldSync(input: { mode: Mode; options: SyncOptions; pending: boolean; sinceLastMs: number; nowMs: number }): boolean {
  const { mode, options, pending } = input;
  if (mode === "disabled" || !pending || syncPaused(options, input.nowMs)) return false;
  if (options.interval === "on_exit") return false;
  // A folder already follows each change; the tick only backs up the network targets.
  if (options.interval === "auto") return mode === "google_drive_web" || mode === "direct_webdav";
  return input.sinceLastMs >= INTERVAL_MS[options.interval];
}

/** Unix seconds for "pause for an hour" and "pause until tomorrow" (the next local midnight). */
export function pauseUntil(choice: "hour" | "tomorrow", now: Date): number {
  if (choice === "hour") return Math.floor(now.getTime() / 1000) + 3600;
  const midnight = new Date(now);
  midnight.setHours(24, 0, 0, 0);
  return Math.floor(midnight.getTime() / 1000);
}
