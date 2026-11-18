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

export const RAM_CHOICES_MB = [1024, 2048, 4096, 8192] as const;
export const CPU_CHOICES_PERCENT = [25, 50, 75] as const;
export const NETWORK_CHOICES_KBPS = [1000, 5000, 10000, 25000] as const;

const STORAGE_KEY = "spark_resource_limits";
const NONE: ResourceLimits = { ramMb: 0, cpuPercent: 0, networkKbps: 0 };

/** Anything that is not one of the offered choices falls back to "no limit", so a damaged value never limits the browser. */
export function normalizeLimits(value: unknown): ResourceLimits {
  const saved = (typeof value === "object" && value !== null ? value : {}) as Record<string, unknown>;
  const pick = (given: unknown, allowed: readonly number[]) => (typeof given === "number" && allowed.includes(given) ? given : 0);
  return {
    ramMb: pick(saved.ramMb, RAM_CHOICES_MB),
    cpuPercent: pick(saved.cpuPercent, CPU_CHOICES_PERCENT),
    networkKbps: pick(saved.networkKbps, NETWORK_CHOICES_KBPS),
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
