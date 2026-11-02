import assert from 'node:assert/strict';
import {writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {withFloatingPlayer} from './pip-player.mjs';
import {samplePipFrames} from './pip-performance.mjs';

// Network-dependent, explicitly invoked only; excluded from the local fixture suite.
export async function checkYoutubePip(context) {
  const {invoke,until,script,wait,screenshot,directory}=context;
  const id=(await invoke('create_tab',{url:'https://www.youtube.com/watch?v=WKc0w7t6mzE'})).tabs.find(t=>t.is_active).id;
  await until(async()=>{
    const state=(await invoke('refresh_tab_media',{tabId:id})).tabs.find(t=>t.tab_id===id);
    const candidates=state.frames.flatMap(frame=>frame.videos.filter(v=>v.eligible).map(video=>({frame,video}))).sort((a,b)=>b.video.width*b.video.height-a.video.width*a.video.height);
    if(!candidates.length)return false;
    const {frame,video}=candidates[0];
    try{await invoke('enter_video_pip',{tabId:id,context:frame.context,document:frame.document,video:video.id});return true;}
    catch(error){if(/media.stale|media.pip_unavailable/.test(String(error)))return false;throw error;}
  },'YouTube eligible source',60000);
  await withFloatingPlayer(context,async()=>{
    await until(()=>script('return document.querySelector("video").readyState>=2;'),'YouTube mirrored image');
    await script('const s=window.opener.document.querySelector("video");window.__pipTestTransitions=0;const changed=()=>window.__pipTestTransitions++;s.addEventListener("loadedmetadata",changed);window.addEventListener("pagehide",()=>s.removeEventListener("loadedmetadata",changed),{once:true});');
    await until(()=>script('return getComputedStyle(window.opener.document.querySelector("video")).visibility==="hidden";'),'YouTube original image hidden');
    assert.equal(await script('return document.querySelector("video").srcObject.getAudioTracks().length'),0);
  });
  const samples=[];let stable=0;
  for(let attempt=0;attempt<8&&stable<2;attempt++) {
    await wait(2000);
    const sample=await samplePipFrames(context);
    samples.push(sample);writeFileSync(resolve(directory,'artifacts','youtube-fps.json'),JSON.stringify(samples,null,2));
    console.log('YouTube PiP FPS',JSON.stringify(sample));
    if(sample.unstable){stable=0;continue;}
    assert.equal(sample.track,'live');assert.ok(sample.sourceFPS>5,'source must be delivering frames');
    assert.ok(sample.presentedFPS>=sample.sourceFPS*.9,'copy retains at least 90% of delivered source FPS');
    assert.ok(sample.maxGapMs<=250,`stable YouTube sample stalled for ${sample.maxGapMs} ms`);stable++;
  }
  assert.equal(stable,2,'two stabilized samples after any advertisement/source transition');
  await withFloatingPlayer(context,async()=>{
    console.log('YouTube observed source transitions',await script('return window.__pipTestTransitions;'));
    await script('document.querySelector("[data-testid=pip-speaker]").click();document.dispatchEvent(new PointerEvent("pointermove"));');
    await wait(200);await screenshot('youtube-compact-volume');
    await script('const s=document.querySelector("[data-testid=pip-volume]");s.value="65";s.dispatchEvent(new Event("input",{bubbles:true}));s.dispatchEvent(new Event("change",{bubbles:true}));');
  });
  await until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id)?.applied_volume===65,'YouTube source volume');
  await invoke('exit_video_pip');
  await context.withWebview('/watch',()=>until(()=>script('return getComputedStyle(document.querySelector("video")).visibility!=="hidden"&&!document.querySelector("video").paused;'),'YouTube image/playback restored'));
}
