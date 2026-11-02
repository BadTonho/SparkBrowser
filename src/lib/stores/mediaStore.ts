import { isTauriRuntime } from "../services/bridge/runtime";
import { createMediaVolumeWriter } from "../services/media-volume-writer.js";
import { createMediaRefresher } from "../services/media-refresher.js";
import { get, writable } from "svelte/store";
import { setTabMuted, refreshTabMedia, setTabVolume, enterVideoPip, exitVideoPip } from "../services/bridge/media";
import { describeError, errorDetails } from "../services/user-errors.js";
import type { MediaSnapshot } from "../types/media";
import { locale, tr } from "../i18n";
export const mediaState = writable<MediaSnapshot>({revision: 0, tabs: []});
export const mediaError = writable("");
export const mediaBusy = writable<Set<string>>(new Set());
export function applyMedia(s: MediaSnapshot) { if (s.revision >= get(mediaState).revision) { mediaState.set(s);for(const [id,pip] of pipVolumeEdits)if(!s.tabs.some(tab=>tab.tab_id===id&&tab.frames.some(frame=>frame.pip_session===pip.session)))pipVolumeEdits.delete(id); } }
export async function toggleTabMute(id: string) {
  if (get(mediaBusy).has(id)) return;
  mediaBusy.update(v => new Set(v).add(id)); mediaError.set("");
  try { applyMedia(await setTabMuted(id, !get(mediaState).tabs.find(t => t.tab_id === id)?.muted)); }
  catch(e) { mediaError.set(describeError(e)); }
  finally { mediaBusy.update(v => { const n = new Set(v); n.delete(id); return n; }); }
}
export const volumeDrafts = writable<Record<string,number>>({});
export const flushVolume = () => volumeWriter.flush();
const refreshReport = createMediaRefresher(refreshTabMedia, applyMedia);
export async function refreshMedia(id: string, silent = false): Promise<MediaSnapshot | undefined> {
  if (!isTauriRuntime() || !get(mediaState).tabs.find(tab => tab.tab_id === id)?.mute_available) return;
  try { return await refreshReport(id); }
  catch (error) { if (!silent && errorDetails(error).code !== "media.stale") mediaError.set(describeError(error)); }
}
const volumeWriter = createMediaVolumeWriter(
  (item: {id:string;document:string;value:number})=>{const pip=pipVolumeEdits.get(item.id);return setTabVolume(item.id,item.value,item.document,pip?.session,pip?.sequence);},
  applyMedia,
  (e:unknown,item:{id:string;document:string})=>{if(errorDetails(e).code!=="media.stale" || get(mediaState).tabs.find(t=>t.tab_id===item.id)?.document_key===item.document)mediaError.set(describeError(e));},
  (item:{id:string;value:number},pending:boolean)=>{if(!pending)volumeDrafts.update(d=>{if(d[item.id]!==item.value)return d;const next={...d};delete next[item.id];return next;});}
);
const pipVolumeEdits = new Map<string,{session:string;sequence:number}>();
export function queueVolume(id:string,document:string,value:number,pip?:{session:string;sequence:number}) {if(pip)pipVolumeEdits.set(id,pip);mediaError.set("");volumeDrafts.update(d=>({...d,[id]:value}));volumeWriter.schedule({id,document,value});}

export const pipBusy=writable(false);
export async function enterVideo(id:string,context:string,document:string,video:string) {
  if(get(pipBusy))return;pipBusy.set(true);mediaError.set("");
  try{applyMedia(await enterVideoPip(id,context,document,video,{locale:get(locale),labels:{play:tr("media.play"),pause:tr("media.pause"),back:tr("media.back"),forward:tr("media.forward"),progress:tr("media.progress"),volume:tr("media.volume"),return:tr("media.return"),close:tr("media.close"),fallback:tr("media.pip_native_fallback")}}));}catch(e){mediaError.set(describeError(e));}finally{pipBusy.set(false);}
}
export async function exitVideo() {
  if(get(pipBusy))return;pipBusy.set(true);mediaError.set("");
  try{await flushVolume();applyMedia(await exitVideoPip());}catch(e){mediaError.set(describeError(e));}finally{pipBusy.set(false);}
}
