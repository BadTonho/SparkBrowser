import { clearEngineData } from "./browser-data";
import { wipeBrowsingHistory } from "../stores/persistenceStore";
import { wipeDownloads } from "../stores/downloadsStore";

export type CleanItem = "cache" | "site" | "history" | "downloads";
export type CleanChoice = Record<CleanItem, boolean>;
export const CLEAN_ITEMS: readonly CleanItem[] = ["cache", "site", "history", "downloads"];

/** The engine could clear some of its stores but not all of them. */
export class PartialCleaning extends Error {}

export interface CleanOutcome {
  /** Items that were cleared. */
  done: CleanItem[];
  /** Items that could not be cleared, grouped the way they were attempted (cache and site data go together). */
  failed: { items: CleanItem[]; error: unknown }[];
}

/**
 * Clears the chosen items, all of what they hold or only what is older than `olderThanDays`. One failing
 * item never stops the others. Used by the cleaning page and by the automatic cleaning.
 */
export async function runCleaning(items: CleanChoice, olderThanDays: number | null): Promise<CleanOutcome> {
  const outcome: CleanOutcome = { done: [], failed: [] };
  const attempt = async (group: CleanItem[], action: () => Promise<unknown>) => {
    try { await action(); outcome.done.push(...group); }
    catch (error) { outcome.failed.push({ items: group, error }); }
  };
  if (items.history) await attempt(["history"], () => wipeBrowsingHistory(olderThanDays));
  if (items.downloads) await attempt(["downloads"], () => wipeDownloads(olderThanDays));
  if (items.cache || items.site) {
    const group = CLEAN_ITEMS.filter(item => (item === "cache" || item === "site") && items[item]);
    await attempt(group, async () => {
      const report = await clearEngineData({ cache: items.cache, siteData: items.site, olderThanDays });
      if (report.failed > 0) throw new PartialCleaning("Some data could not be cleared");
    });
  }
  return outcome;
}
