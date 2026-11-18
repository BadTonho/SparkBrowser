/** What a cleaning would free, as the native side measured it (bytes). `null`: this system cannot measure it. */
export interface CleanupEstimate {
  cache_bytes: number | null;
  site_bytes: number | null;
  history_bytes: number;
  downloads_bytes: number;
  /** The measuring of a very large folder stopped early: the real numbers are at least these. */
  partial: boolean;
}

export type CleanupItem = "cache" | "site" | "history" | "downloads";

const FIELD: Record<CleanupItem, keyof Omit<CleanupEstimate, "partial">> = {
  cache: "cache_bytes", site: "site_bytes", history: "history_bytes", downloads: "downloads_bytes",
};

/** What one item would free; `null` when it cannot be measured. */
export function itemBytes(estimate: CleanupEstimate, item: CleanupItem): number | null {
  return estimate[FIELD[item]];
}

/** The total of the chosen items, and whether any chosen item could not be measured (it counts as nothing). */
export function chosenBytes(estimate: CleanupEstimate, chosen: Record<CleanupItem, boolean>): { total: number; unknown: boolean } {
  let total = 0;
  let unknown = false;
  for (const item of Object.keys(FIELD) as CleanupItem[]) {
    if (!chosen[item]) continue;
    const bytes = itemBytes(estimate, item);
    if (bytes === null) unknown = true; else total += bytes;
  }
  return { total, unknown };
}

/** A size the way people read it ("0 B", "512 B", "1.5 KB", "312 MB", "1.2 GB"). */
export function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = Math.max(0, bytes);
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) { value /= 1024; unit++; }
  const text = unit === 0 || value >= 100 ? String(Math.round(value)) : value.toFixed(value >= 10 ? 0 : 1).replace(/\.0$/, "");
  return `${text} ${units[unit]}`;
}
