import { setOverlayVisibility } from "../services/bridge/overlays";
import { isTauriRuntime } from "../services/bridge/runtime";
import { createMediaVolumeWriter } from "../services/media-volume-writer.js";
import { get, writable } from "svelte/store";
import { fetchTabMedia, onTabMedia, setTabMuted, refreshTabMedia, setTabVolume, enterVideoPip, exitVideoPip } from "../services/bridge/media";
import { describeError, errorDetails } from "../services/user-errors.js";
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

export const isMediaOpen = writable(false);
export const volumeDrafts = writable<Record<string,number>>({});
export async function openMedia() { mediaError.set(""); await setOverlayVisibility(true,"media"); isMediaOpen.set(true); }
export async function closeMedia() { isMediaOpen.set(false); await volumeWriter.flush(); await setOverlayVisibility(false,"media"); }
let refreshing = false;
export async function refreshMedia(id: string) {
  if (refreshing || !isTauriRuntime() || !get(mediaState).tabs.find(t=>t.tab_id===id)?.mute_available) return; refreshing=true;
  try { applyMedia(await refreshTabMedia(id)); } catch(e) {if(errorDetails(e).code!=="media.stale")mediaError.set(describeError(e));} finally {refreshing=false;}
}
const volumeWriter = createMediaVolumeWriter(
  (item: {id:string;document:string;value:number})=>setTabVolume(item.id,item.value,item.document),
  applyMedia,
  (e:unknown,item:{id:string;document:string})=>{if(errorDetails(e).code!=="media.stale" || get(mediaState).tabs.find(t=>t.tab_id===item.id)?.document_key===item.document)mediaError.set(describeError(e));},
  (item:{id:string;value:number},pending:boolean)=>{if(!pending)volumeDrafts.update(d=>{if(d[item.id]!==item.value)return d;const next={...d};delete next[item.id];return next;});}
);
export function queueVolume(id:string,document:string,value:number) {mediaError.set("");volumeDrafts.update(d=>({...d,[id]:value}));volumeWriter.schedule({id,document,value});}

export const pipBusy=writable(false);
export async function enterVideo(id:string,context:string,document:string,video:string) {
  if(get(pipBusy))return;pipBusy.set(true);mediaError.set("");
  try{applyMedia(await enterVideoPip(id,context,document,video));}catch(e){mediaError.set(describeError(e));}finally{pipBusy.set(false);}
}
export async function exitVideo() {
  if(get(pipBusy))return;pipBusy.set(true);mediaError.set("");
  try{applyMedia(await exitVideoPip());}catch(e){mediaError.set(describeError(e));}finally{pipBusy.set(false);}
}
