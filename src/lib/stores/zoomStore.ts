import { derived, get, writable } from "svelte/store";
import { currentUrl } from "./browserStore";
import { fetchSiteZoom, onSiteZoom, setSiteZoom } from "../services/bridge/zoom";
import { isTauriRuntime } from "../services/bridge/runtime";
import { describeError } from "../services/user-errors.js";
import type { ZoomSnapshot } from "../types/zoom";
export const zoomState = writable<ZoomSnapshot>({ revision: 0, sites: [] });
export const zoomError = writable("");
export const zoomBusy = writable(false);
export const zoomOrigin = derived(currentUrl, value => { try { const u = new URL(value); return /^https?:$/.test(u.protocol) ? u.origin : null; } catch { return null; } });
export const currentZoom = derived([zoomState, zoomOrigin], ([state, origin]) => state.sites.find(s => s.origin === origin)?.factor ?? 1);
export const ZOOM_LEVELS = [0.25, 0.33, 0.5, 0.67, 0.75, 0.8, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2, 2.5, 3, 4, 5];
function apply(state: ZoomSnapshot) { if (state.revision >= get(zoomState).revision) zoomState.set(state); }
export async function changeZoom(direction: -1 | 0 | 1, origin = get(zoomOrigin)) {
  if (!origin || !isTauriRuntime() || get(zoomBusy)) return;
  const value = get(zoomState).sites.find(s => s.origin === origin)?.factor ?? 1;
  const next = direction === 0 ? 1 : direction > 0 ? ZOOM_LEVELS.find(level => level > value + 0.001) ?? 5 : [...ZOOM_LEVELS].reverse().find(level => level < value - 0.001) ?? 0.25;
  zoomBusy.set(true); zoomError.set("");
  try { apply(await setSiteZoom(origin, next)); } catch (e) { zoomError.set(describeError(e)); }
  finally { zoomBusy.set(false); }
}
export function initializeZoom() {
  let disposed = false; let stop: (() => void) | undefined;
  void (async () => { stop = await onSiteZoom(s => { if (!disposed) apply(s); }); if (disposed) { stop(); return; } const s = await fetchSiteZoom(); if (!disposed) apply(s); })().catch(e => zoomError.set(describeError(e)));
  return () => { disposed = true; stop?.(); };
}
