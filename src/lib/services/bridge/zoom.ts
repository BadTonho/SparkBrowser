import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { ZoomSnapshot } from "../../types/zoom";
export const fetchSiteZoom = () => isTauriRuntime() ? invoke<ZoomSnapshot>("get_site_zoom") : Promise.resolve({ revision: 0, sites: [] });
export const setSiteZoom = (origin: string, factor: number) => invoke<ZoomSnapshot>("set_site_zoom", { origin, factor });
export const applyBackupZoom = (taskId: string) => invoke<ZoomSnapshot>("apply_backup_zoom", { taskId });
export const onSiteZoom = (callback: (s: ZoomSnapshot) => void) => isTauriRuntime() ? listen<ZoomSnapshot>("site-zoom-state", event => callback(event.payload)) : Promise.resolve(() => {});
