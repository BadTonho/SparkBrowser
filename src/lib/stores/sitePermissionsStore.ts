import { writable } from "svelte/store";
import type { PermissionRequest } from "../types/browser";
import { onPermissionRequests, fetchSitePermissions } from "../services/bridge/site-permissions";
import { setOverlayVisibility } from "../services/bridge/overlays";
export const permissionRequests = writable<PermissionRequest[]>([]);
export const isSitePermissionsOpen = writable(false);
export async function openSitePermissions() { await setOverlayVisibility(true, "site-permissions"); isSitePermissionsOpen.set(true); }
export async function closeSitePermissions() { isSitePermissionsOpen.set(false); await setOverlayVisibility(false, "site-permissions"); }
export function initializePermissionEvents(): () => void {
  let disposed = false; let stop: (() => void) | undefined;
  function apply(requests: PermissionRequest[]) { if (disposed) return; permissionRequests.set(requests); void setOverlayVisibility(requests.length > 0, "permission-prompt"); }
  void onPermissionRequests(apply).then(async unlisten => { if (disposed) { unlisten(); return; } stop = unlisten; const snapshot = await fetchSitePermissions(); apply(snapshot.requests); }).catch(() => {});
  return () => { disposed = true; stop?.(); };
}
