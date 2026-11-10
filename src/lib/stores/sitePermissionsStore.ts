import { createSharedLifecycle } from "../services/lifecycle.js";
import { get, writable } from "svelte/store";
import type { PermissionRequest } from "../types/browser";
import { onPermissionRequests, fetchSitePermissions } from "../services/bridge/site-permissions";
import { setOverlayVisibility } from "../services/bridge/overlays";
const confirmedPermissionRequests = writable<PermissionRequest[]>([]);
export const permissionRequests = { subscribe: confirmedPermissionRequests.subscribe };
export const isSitePermissionsOpen = writable(false);
export async function openSitePermissions() {
  if (get(permissionRequests).length) return;
  await setOverlayVisibility(true, "site-permissions");
  isSitePermissionsOpen.set(true);
}
export async function closeSitePermissions() { isSitePermissionsOpen.set(false); await setOverlayVisibility(false, "site-permissions"); }
function startPermissionEvents(): () => void {
  let revision = 0;
  let disposed = false; let stop: (() => void) | undefined;
  function apply(requests: PermissionRequest[]) {
    if (disposed) return;
    confirmedPermissionRequests.set(requests);
    if (requests.length && get(isSitePermissionsOpen)) {
      isSitePermissionsOpen.set(false);
      void setOverlayVisibility(false, "site-permissions");
    }
    void setOverlayVisibility(requests.length > 0, "permission-prompt");
  }
  void onPermissionRequests(requests => { revision++; apply(requests); }).then(async unlisten => { if (disposed) { unlisten(); return; } stop = unlisten; const observed = revision; const snapshot = await fetchSitePermissions(); if (revision === observed) apply(snapshot.requests); }).catch(() => {});
  return () => { disposed = true; stop?.(); };
}

export const initializePermissionEvents = createSharedLifecycle(scope => scope.own(startPermissionEvents()));
