import { get, writable } from "svelte/store";
import { safeStorage } from "../services/storage.js";
import { isTauriRuntime } from "../services/bridge/runtime";
import { invoke } from "../services/bridge/runtime";
import { logIgnored } from "../services/ignored-error";
import { CLEAN_ITEMS, runCleaning, type CleanChoice } from "../services/cleaning";

/**
 * The automatic cleaning. It is off by default. When it is on, the browser clears the chosen items, only what
 * is older than `days` days, when it opens and then about once a day. It always has a period: clearing
 * everything by itself (signing you out of sites, for one) is never automatic.
 */
export interface AutoClean {
  enabled: boolean;
  items: CleanChoice;
  days: number;
  /** When it last ran, in milliseconds since 1970; `0` if never. */
  lastRun: number;
}

export const AUTO_CLEAN_DAYS = [5, 10, 20, 30, 90] as const;
const STORAGE_KEY = "spark_auto_clean";
const DAY_MS = 24 * 60 * 60 * 1000;
const FIRST_CHECK_MS = 60_000;
const CHECK_EVERY_MS = 60 * 60 * 1000;

const OFF: AutoClean = { enabled: false, items: { cache: true, site: false, history: false, downloads: false }, days: 30, lastRun: 0 };

/** A damaged or half-written value never turns the cleaning on: it needs a known period and something to clear. */
export function normalizeAutoClean(value: unknown): AutoClean {
  const saved = (typeof value === "object" && value !== null ? value : {}) as Record<string, unknown>;
  const savedItems = (typeof saved.items === "object" && saved.items !== null ? saved.items : {}) as Record<string, unknown>;
  const items = Object.fromEntries(CLEAN_ITEMS.map(item => [item, savedItems[item] === true])) as CleanChoice;
  const days = typeof saved.days === "number" && (AUTO_CLEAN_DAYS as readonly number[]).includes(saved.days) ? saved.days : 0;
  const lastRun = typeof saved.lastRun === "number" && Number.isFinite(saved.lastRun) && saved.lastRun > 0 ? saved.lastRun : 0;
  const usable = days > 0 && Object.values(items).some(Boolean);
  return { enabled: saved.enabled === true && usable, items, days: days || OFF.days, lastRun };
}

function load(): AutoClean {
  try { return normalizeAutoClean(JSON.parse(safeStorage.getItem(STORAGE_KEY) ?? "null")); }
  catch { return { ...OFF }; }
}

export const autoClean = writable<AutoClean>(load());

/** Saves what the panel chose. It turns itself off when there is no period or nothing chosen. */
export function setAutoClean(change: Partial<AutoClean>): void {
  const next = normalizeAutoClean({ ...get(autoClean), ...change });
  autoClean.set(next);
  try { safeStorage.setItem(STORAGE_KEY, JSON.stringify(next)); } catch (error) { logIgnored(error); }
}

/** Whether a cleaning is due: it is on and it has not run in the last day. */
export function isCleaningDue(settings: AutoClean, now: number): boolean {
  return settings.enabled && now - settings.lastRun >= DAY_MS;
}

let running = false;

/** Runs the automatic cleaning if it is due. The time is saved first, so a failure is not retried in a loop. */
export async function runAutoCleanIfDue(now = Date.now()): Promise<boolean> {
  const settings = get(autoClean);
  if (running || !isCleaningDue(settings, now)) return false;
  running = true;
  try {
    setAutoClean({ lastRun: now });
    const outcome = await runCleaning(settings.items, settings.days);
    for (const failure of outcome.failed) {
      const reason = failure.error as { message?: string } | undefined;
      void invoke("report_frontend_error", {
        summary: `Automatic cleaning could not clear ${failure.items.join(", ")}: ${reason?.message ?? String(failure.error)}`,
        details: "", level: "warning",
      }).catch(() => {});
    }
    return outcome.done.length > 0;
  } finally {
    running = false;
  }
}

/** Starts the schedule: a check a minute after the browser opens, then every hour. Returns the way to stop it. */
export function startAutoClean(): () => void {
  if (!isTauriRuntime()) return () => {};
  const first = setTimeout(() => void runAutoCleanIfDue().catch(logIgnored), FIRST_CHECK_MS);
  const timer = setInterval(() => void runAutoCleanIfDue().catch(logIgnored), CHECK_EVERY_MS);
  return () => { clearTimeout(first); clearInterval(timer); };
}
