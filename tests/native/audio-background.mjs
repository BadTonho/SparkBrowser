import assert from 'node:assert/strict';
import {release} from 'node:os';
import {newTabDriver} from './new-tab-driver.mjs';

export async function checkAudioBackground(context){
  const {invoke,until,fixture,script,click,wait,restart,withWebview}=context;
  const {inside,ready}=newTabDriver(context);
  const source=(await invoke('create_tab',{url:fixture.origin+'/page/media-equalizer'})).tabs.find(t=>t.is_active).id;
  await until(async()=>(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===source)?.audible,'equalizer source audible');
  const target=(await invoke('create_tab')).tabs.find(t=>t.is_active).id;await ready();
  await inside(()=>script('window.audioFrames=[];window.addEventListener("spark-audio-frame",e=>{window.audioFrames.push(e.detail);if(window.audioFrames.length>60)window.audioFrames.shift()})'));
  await inside(()=>click('#widget-settings'));await inside(()=>click('#menu-item-wallpaper'));
  await inside(()=>script('const n=document.querySelector("#audio-background-enabled");n.checked=true;n.dispatchEvent(new Event("change"))'));
  await inside(()=>click('#settings-close'));
  if(process.platform==='win32'&&Number(release().split('.')[2])<20348){
    await inside(()=>until(()=>script('return window.audioFrames.some(f=>f.status==="audio_unavailable")'),'unsupported Windows is explained'));return;
  }
  await inside(()=>until(()=>script('return window.audioFrames.some(f=>f.status==="ready"&&Math.max(...f.bands)>0.05)'),'real browser audio reaches the FFT',30000));
  await inside(()=>until(()=>script('return !document.querySelector("#audio-background").hidden'),'equalizer visible'));
  const frequencies=await inside(()=>script('const f=window.audioFrames.findLast(f=>Math.max(...f.bands)>0.05);return f.bands'));
  const peak=frequencies.indexOf(Math.max(...frequencies));
  assert.ok(peak>=12&&peak<=16,'440 Hz tone peaks in the expected logarithmic band');
  await inside(()=>context.screenshot('audio-background'));
  await invoke('set_tab_muted',{tabId:source,muted:true});
  await inside(()=>until(()=>script('return document.querySelector("#audio-background").hidden'),'mute fades to silence'));
  await invoke('set_tab_muted',{tabId:source,muted:false});
  await inside(()=>until(()=>script('return !document.querySelector("#audio-background").hidden'),'unmute resumes the equalizer'));
  const media=(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===source);
  await invoke('set_tab_volume',{tabId:source,volume:0,documentKey:media.document_key});
  await inside(()=>until(()=>script('return document.querySelector("#audio-background").hidden'),'zero volume clears the equalizer'));
  const latest=(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===source);
  await invoke('set_tab_volume',{tabId:source,volume:100,documentKey:latest.document_key});
  await inside(()=>until(()=>script('return !document.querySelector("#audio-background").hidden'),'volume restored'));
  await invoke('switch_tab',{tabId:source});await wait(500);
  const before=await inside(()=>script('return window.audioFrames.length'),target);await wait(500);
  assert.equal(await inside(()=>script('return window.audioFrames.length'),target),before,'hidden new tab receives no spectrum');
  const stillPlaying=await withWebview('/page/media-equalizer',()=>script('return !document.querySelector("audio").paused&&!document.querySelector("audio").muted'));
  assert.equal(stillPlaying,true,'analysis never interrupts playback');
  await invoke('switch_tab',{tabId:target});await ready();
  await inside(()=>until(()=>script('return !document.querySelector("#audio-background").hidden'),'returning to the new tab resumes capture'));
  await restart();await ready();
  await inside(()=>{return script('window.audioRestored=false;window.addEventListener("spark-new-tab-widgets",e=>window.audioRestored=e.detail.settings.audio_background.enabled);window.__sparkRefreshWidgets()');});
  await inside(()=>until(()=>script('return window.audioRestored'),'equalizer preference persists'));
  await inside(()=>click('#widget-settings'));await inside(()=>click('#menu-item-wallpaper'));
  await inside(()=>script('const n=document.querySelector("#audio-background-enabled");n.checked=false;n.dispatchEvent(new Event("change"))'));
  await inside(()=>until(()=>script('return document.querySelector("#audio-background").hidden'),'disabled effect stops rendering'));
  await inside(()=>click('#settings-close'));
}
