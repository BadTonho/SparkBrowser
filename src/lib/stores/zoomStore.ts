import { derived, get, writable } from "svelte/store";
import { currentUrl } from "./browserStore";
import { fetchSiteZoom, onSiteZoom, setSiteZoom } from "../services/bridge/zoom";
import { isTauriRuntime } from "../services/bridge/runtime";
import { describeError } from "../services/user-errors.js";
import type { ZoomSnapshot } from "../types/zoom";
import { createZoomWriter } from "../services/zoom-writer.js";
export const zoomState = writable<ZoomSnapshot>({ revision: 0, sites: [] });
export const zoomError = writable("");
export const zoomBusy = writable(false);
export const zoomDrafts = writable<Record<string, number>>({});
export const zoomErrors = writable<Record<string, string>>({});
export const zoomOrigin = derived(currentUrl, value => { try { const u = new URL(value); return /^https?:$/.test(u.protocol) ? u.origin : null; } catch { return null; } });
export const currentZoom = derived([zoomState, zoomOrigin], ([state, origin]) => state.sites.find(s => s.origin === origin)?.factor ?? 1);
export const ZOOM_LEVELS = [0.25, 0.33, 0.5, 0.67, 0.75, 0.8, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2, 2.5, 3, 4, 5];
function apply(state: ZoomSnapshot) { if (state.revision >= get(zoomState).revision) zoomState.set(state); }
const writer = createZoomWriter(setSiteZoom, (state: ZoomSnapshot, origin: string) => {
  apply(state);
  zoomErrors.update(errors => { const next = { ...errors }; delete next[origin]; return next; });
}, fetchSiteZoom, (error: unknown, origin: string) => {
  const message = describeError(error);
  zoomError.set(message);
  zoomErrors.update(errors => ({ ...errors, [origin]: message }));
}, (origin: string, pending: boolean) => {
  if (!pending) zoomDrafts.update(drafts => { const next = { ...drafts }; delete next[origin]; return next; });
  zoomBusy.set(Object.keys(get(zoomDrafts)).length > 0);
});
export const flushZoom = () => writer.flush();
export async function setZoomFactor(factor: number, origin = get(zoomOrigin)) {
  if (!origin || !isTauriRuntime() || !Number.isFinite(factor) || factor < 0.25 || factor > 5) return;
  const current = get(zoomDrafts)[origin] ?? get(zoomState).sites.find(site => site.origin === origin)?.factor ?? 1;
  if (Math.abs(current - factor) < 0.00001 && !get(zoomErrors)[origin]) return flushZoom();
  zoomError.set("");
  zoomErrors.update(errors => { const next = { ...errors }; delete next[origin]; return next; });
  zoomDrafts.update(drafts => ({ ...drafts, [origin]: factor }));
  writer.schedule(origin, factor);
  await flushZoom();
}
export async function changeZoom(direction: -1 | 0 | 1, origin = get(zoomOrigin)) {
  if (!origin || !isTauriRuntime()) return;
  const value = get(zoomDrafts)[origin] ?? get(zoomState).sites.find(s => s.origin === origin)?.factor ?? 1;
  const next = direction === 0 ? 1 : direction > 0 ? ZOOM_LEVELS.find(level => level > value + 0.001) ?? 5 : [...ZOOM_LEVELS].reverse().find(level => level < value - 0.001) ?? 0.25;
  await setZoomFactor(next, origin);
}
export function initializeZoom() {
  let disposed = false; let stop: (() => void) | undefined;
  void (async () => { stop = await onSiteZoom(s => { if (!disposed) apply(s); }); if (disposed) { stop(); return; } const s = await fetchSiteZoom(); if (!disposed) apply(s); })().catch(e => zoomError.set(describeError(e)));
  return () => { disposed = true; stop?.(); };
}
