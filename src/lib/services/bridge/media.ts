import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { MediaSnapshot } from "../../types/media";
export const fetchTabMedia = () => isTauriRuntime() ? invoke<MediaSnapshot>("get_tab_media") : Promise.resolve({revision: 0, tabs: []});
export const setTabMuted = (tabId: string, muted: boolean) => invoke<MediaSnapshot>("set_tab_muted", {tabId, muted});
export const onTabMedia = (callback: (state: MediaSnapshot) => void) => isTauriRuntime() ? listen<MediaSnapshot>("tab-media-state", e => callback(e.payload)) : Promise.resolve(() => {});
