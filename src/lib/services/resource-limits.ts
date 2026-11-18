import type { TabItem } from "../types/browser";

/**
 * The tabs to hibernate, in order, when the open tabs use more memory than the limit: the ones the user
 * went to longest ago first. The active tab, the tabs already asleep, and the ones with something going on
 * (a form being edited, media playing, a download) are never offered; the native side refuses the rest of
 * the protected ones, and the caller then tries the next.
 */
export function hibernationCandidates(tabs: readonly TabItem[]): TabItem[] {
  return tabs
    .filter(tab => !tab.is_active && !tab.is_hibernated && !tab.is_visible
      && !tab.activity?.dirty_form && !tab.activity?.playing_media && !tab.activity?.downloading && !tab.activity?.picture_in_picture && !tab.is_pinned)
    .sort((a, b) => (a.last_active_epoch ?? 0) - (b.last_active_epoch ?? 0));
}

/** The open tabs the user is not looking at: not the active one and not one shown in a split. */
export function backgroundTabIds(tabs: readonly TabItem[]): Set<string> {
  return new Set(tabs.filter(tab => !tab.is_active && !tab.is_visible).map(tab => tab.id));
}

/** Memory the open tabs use, in megabytes, from the per-tab measurements. */
export function usedMegabytes(memory: Record<string, number>): number {
  return Object.values(memory).reduce((total, bytes) => total + (bytes > 0 ? bytes : 0), 0) / (1024 * 1024);
}

/** Whether the memory is over the limit, with a margin so a reading close to it does not hibernate tabs back and forth. */
export function isOverLimit(usedMb: number, limitMb: number): boolean {
  return limitMb > 0 && usedMb > limitMb * 1.02;
}

/** A speed in bits per second, as people read network speeds ("340 kbps", "12.5 Mbps"). */
export function formatSpeed(bytesPerSecond: number): string {
  const bits = Math.max(0, bytesPerSecond) * 8;
  if (bits >= 1_000_000) return `${(bits / 1_000_000).toFixed(bits >= 10_000_000 ? 0 : 1)} Mbps`;
  return `${Math.round(bits / 1000)} kbps`;
}

/** A limit in megabytes as a label ("2 GB"). */
export function formatMemoryLimit(megabytes: number): string {
  return megabytes >= 1024 ? `${megabytes / 1024} GB` : `${megabytes} MB`;
}
