import { invoke, isTauriRuntime } from "./runtime";
import { listen } from "@tauri-apps/api/event";
import type { PermissionRequest, SitePermission, PermissionSnapshot } from "../../types/browser";
export const fetchSitePermissions = () => isTauriRuntime() ? invoke<PermissionSnapshot>("get_site_permissions") : Promise.resolve<PermissionSnapshot>({ supported: false, message: "Site permissions require Windows desktop WebView2.", settings: [], requests: [] });
export const setSitePermission = (origin: string, kind: SitePermission["kind"], state: SitePermission["state"]) => invoke<void>("set_site_permission", { origin, kind, state });
export const respondSitePermission = (id: string, decision: string) => invoke<void>("respond_site_permission", { id, decision });
export async function onPermissionRequests(callback: (requests: PermissionRequest[]) => void) {
  return isTauriRuntime() ? listen<PermissionRequest[]>("permission-requests-updated", event => callback(event.payload)) : () => {};
}
