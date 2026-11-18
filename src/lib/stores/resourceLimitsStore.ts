import { get, writable } from "svelte/store";
import { safeStorage } from "../services/storage.js";
import { invoke, isTauriRuntime } from "../services/bridge/runtime";
import { logIgnored } from "../services/ignored-error";

/** The limits the user chose. `0` is "no limit" in every field. */
export interface ResourceLimits {
  /** Memory the open tabs may use together, in megabytes; above it the least recently used tabs are hibernated. */
  ramMb: number;
  /** Share of the processor the web engine may use, in percent. */
  cpuPercent: number;
  /** Speed of the pages' network, in kilobits per second. */
  networkKbps: number;
}

/** What each slider can pick: the slider moves over this list, and one position past its end means "no limit". */
export interface LimitRange {
  values: readonly number[];
  /** What can be typed instead: any whole number from `min` to `max`, in the limit's own unit (MB, percent, kbps). */
  min: number;
  max: number;
}
const steps = (from: number, to: number, by: number) => Array.from({ length: (to - from) / by + 1 }, (_, index) => from + index * by);
export const LIMIT_RANGES: Record<keyof ResourceLimits, LimitRange> = {
  ramMb: { values: steps(512, 16384, 512), min: 256, max: 131072 },
  cpuPercent: { values: steps(10, 90, 5), min: 1, max: 95 },
  // Fine steps at low speeds and coarse ones at high speeds, so 5, 25 and 500 Mbps are both easy to pick (1 Mbps = 1000 kbps).
  networkKbps: { values: [...steps(1000, 9000, 1000), ...steps(10000, 45000, 5000), ...steps(50000, 90000, 10000), ...steps(100000, 1000000, 100000)], min: 100, max: 1000000 },
};

/** The slider position of a limit; "no limit" is the position after the last value. */
export function limitToSlider(range: LimitRange, limit: number): number {
  if (limit === 0) return range.values.length;
  // A typed value between two steps puts the thumb on the nearest one.
  let nearest = 0;
  range.values.forEach((value, index) => { if (Math.abs(value - limit) < Math.abs(range.values[nearest] - limit)) nearest = index; });
  return nearest;
}

/** The limit a slider position means; anything past the last value is "no limit". */
export function sliderToLimit(range: LimitRange, position: number): number {
  return range.values[Math.round(position)] ?? 0;
}

const STORAGE_KEY = "spark_resource_limits";
const NONE: ResourceLimits = { ramMb: 0, cpuPercent: 0, networkKbps: 0 };

/** Anything that is a value the panel cannot give falls back to "no limit", so a damaged value never limits the browser. */
export function normalizeLimits(value: unknown): ResourceLimits {
  const saved = (typeof value === "object" && value !== null ? value : {}) as Record<string, unknown>;
  const pick = (given: unknown, { min, max }: LimitRange) => (typeof given === "number" && Number.isInteger(given) && given >= min && given <= max ? given : 0);
  return {
    ramMb: pick(saved.ramMb, LIMIT_RANGES.ramMb),
    cpuPercent: pick(saved.cpuPercent, LIMIT_RANGES.cpuPercent),
    networkKbps: pick(saved.networkKbps, LIMIT_RANGES.networkKbps),
  };
}

function load(): ResourceLimits {
  try { return normalizeLimits(JSON.parse(safeStorage.getItem(STORAGE_KEY) ?? "null")); }
  catch { return { ...NONE }; }
}

export const resourceLimits = writable<ResourceLimits>(load());

/** Tells the native side what the processor and the network limits are. Memory is enforced by the interface. */
export async function applyNativeLimits(limits = get(resourceLimits)): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<void>("set_resource_limits", { limits: { cpu_percent: limits.cpuPercent, network_kbps: limits.networkKbps } });
}

export async function setResourceLimit<K extends keyof ResourceLimits>(key: K, value: ResourceLimits[K]): Promise<void> {
  const next = normalizeLimits({ ...get(resourceLimits), [key]: value });
  resourceLimits.set(next);
  safeStorage.setItem(STORAGE_KEY, JSON.stringify(next));
  if (key !== "ramMb") await applyNativeLimits(next).catch(logIgnored);
}
