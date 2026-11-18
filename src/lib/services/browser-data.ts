import { invoke, isTauriRuntime } from "./bridge/runtime";
import { logIgnored } from "./ignored-error";

/** `olderThanDays`: clear only what is older than that many days (Windows); absent clears everything. */
export interface EngineDataRequest { cache: boolean; siteData: boolean; olderThanDays?: number | null }
export interface EngineDataReport { cleared: number; failed: number }

/**
 * Clears the web engine's cache and/or the data sites store, all of it or only what is older than some days.
 *
 * The browser's own interface keeps its settings (theme, palette, toolbar buttons...) in the same local
 * storage as the sites when the data is shared, so clearing site data would also wipe them. They are read
 * before and written back after, so only the sites lose their data.
 */
export async function clearEngineData(request: EngineDataRequest): Promise<EngineDataReport> {
  if (!isTauriRuntime()) return { cleared: 0, failed: 0 };
  const saved: [string, string][] = [];
  if (request.siteData) {
    try {
      for (let index = 0; index < localStorage.length; index++) {
        const key = localStorage.key(index);
        const value = key === null ? null : localStorage.getItem(key);
        if (key !== null && value !== null) saved.push([key, value]);
      }
    } catch (error) { logIgnored(error); }
  }
  try {
    return await invoke<EngineDataReport>("clear_browser_data", { request: { cache: request.cache, site_data: request.siteData, older_than_days: request.olderThanDays ?? null } });
  } finally {
    for (const [key, value] of saved) {
      try { localStorage.setItem(key, value); } catch (error) { logIgnored(error); }
    }
  }
}
