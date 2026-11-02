import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { MediaSnapshot } from "../../types/media";
export const fetchTabMedia = () => isTauriRuntime() ? invoke<MediaSnapshot>("get_tab_media") : Promise.resolve({revision: 0, tabs: []});
export const setTabMuted = (tabId: string, muted: boolean) => invoke<MediaSnapshot>("set_tab_muted", {tabId, muted});
export const onTabMedia = (callback: (state: MediaSnapshot) => void) => isTauriRuntime() ? listen<MediaSnapshot>("tab-media-state", e => callback(e.payload)) : Promise.resolve(() => {});

export const refreshTabMedia = (tabId: string) => invoke<MediaSnapshot>("refresh_tab_media", {tabId});
export const setTabVolume = (tabId: string, volume: number, documentKey: string, pipSession?: string, pipSequence?: number) => invoke<MediaSnapshot>("set_tab_volume", {tabId, volume, documentKey, pipSession, pipSequence});

export const enterVideoPip = (tabId:string,context:string,document:string,video:string,presentation?:{locale:string;labels:Record<string,string>}) => invoke<MediaSnapshot>("enter_video_pip",{tabId,context,document,video,presentation});
export const exitVideoPip = () => invoke<MediaSnapshot>("exit_video_pip");

export interface PipAction {session:string;sequence:number;tabId:string;documentKey:string;action:"volume"|"flush"|"return"|"close"|"capture-failed";value:number|null}
export const onPipAction = (callback:(action:PipAction)=>void) => listen<PipAction>("media-pip-action",event=>callback(event.payload));
export const onPipFallback = (callback:()=>void) => listen("media-pip-fallback",callback);
