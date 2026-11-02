import { get, writable } from "svelte/store";
import { fetchTabMedia, onTabMedia, setTabMuted } from "../services/bridge/media";
import { describeError } from "../services/user-errors.js";
import type { MediaSnapshot } from "../types/media";
export const mediaState = writable<MediaSnapshot>({revision: 0, tabs: []});
export const mediaError = writable("");
export const mediaBusy = writable<Set<string>>(new Set());
export function applyMedia(s: MediaSnapshot) { if (s.revision >= get(mediaState).revision) mediaState.set(s); }
export async function toggleTabMute(id: string) {
  if (get(mediaBusy).has(id)) return;
  mediaBusy.update(v => new Set(v).add(id)); mediaError.set("");
  try { applyMedia(await setTabMuted(id, !get(mediaState).tabs.find(t => t.tab_id === id)?.muted)); }
  catch(e) { mediaError.set(describeError(e)); }
  finally { mediaBusy.update(v => { const n = new Set(v); n.delete(id); return n; }); }
}
export function initializeMedia() {
  let disposed = false; let stop: (() => void) | undefined;
  void (async () => { stop = await onTabMedia(s => { if (!disposed) applyMedia(s); }); if (disposed) { stop(); return; } const s = await fetchTabMedia(); if (!disposed) applyMedia(s); })().catch(e => mediaError.set(describeError(e)));
  return () => { disposed = true; stop?.(); };
}
