import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { MediaSnapshot } from "../../types/media";
export const fetchTabMedia = () => isTauriRuntime() ? invoke<MediaSnapshot>("get_tab_media") : Promise.resolve({revision: 0, tabs: []});
export const setTabMuted = (tabId: string, muted: boolean) => invoke<MediaSnapshot>("set_tab_muted", {tabId, muted});
export const onTabMedia = (callback: (state: MediaSnapshot) => void) => isTauriRuntime() ? listen<MediaSnapshot>("tab-media-state", e => callback(e.payload)) : Promise.resolve(() => {});

export const refreshTabMedia = (tabId: string) => invoke<MediaSnapshot>("refresh_tab_media", {tabId});
export const setTabVolume = (tabId: string, volume: number, documentKey: string) => invoke<MediaSnapshot>("set_tab_volume", {tabId, volume, documentKey});

export const enterVideoPip = (tabId:string,context:string,document:string,video:string) => invoke<MediaSnapshot>("enter_video_pip",{tabId,context,document,video});
export const exitVideoPip = () => invoke<MediaSnapshot>("exit_video_pip");
