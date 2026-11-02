import assert from 'node:assert/strict';

export async function checkMediaMute({invoke,until,fixture,click,restart,script}) {
  if (process.platform !== 'win32') return;
  const packet = await invoke('create_tab', {url: fixture.origin+'/page/media-a'});
  const a = packet.tabs.find(t=>t.is_active).id;
  await until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===a)?.audible, 'native audio activity');
  const next = await invoke('create_tab', {url:fixture.origin+'/page/media-b'});
  const b = next.tabs.find(t=>t.is_active).id;
  await until(()=>script('return Boolean(document.querySelector(`[data-media-tab="${arguments[0]}"]`))', a),'background mute control');
  await click(`[data-media-tab="${a}"]`);
  await until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===a)?.muted,'background tab muted');
  assert.equal((await invoke('get_tabs')).tabs.find(t=>t.is_active).id,b,'mute does not select/reorder the tab');
  await invoke('switch_tab',{tabId:a});
  await invoke('navigate_to',{input:fixture.origin+'/page/media-c'});
  await until(()=>fixture.reports.has('media-c'),'navigation');
  assert.equal((await invoke('get_tab_media')).tabs.find(t=>t.tab_id===a).muted,true);
  await invoke('set_tab_muted',{tabId:a,muted:false});
  await until(async()=> !(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===a)?.muted,'unmuted');
  await invoke('set_tab_muted',{tabId:a,muted:true});
  await restart();
  await until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===a)?.mute_available,'restored view');
  assert.equal((await invoke('get_tab_media')).tabs.find(t=>t.tab_id===a).muted,false,'mute is not persisted');
}

export async function checkMediaVolume({invoke,until,fixture,click,script,restart}) {
  if(process.platform!=='win32')return;
  const packet=await invoke('create_tab',{url:fixture.origin+'/page/media-volume'});
  const id=packet.tabs.find(t=>t.is_active).id;
  await until(()=>fixture.reports.has('media-volume'),'audio fixture');
  await until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id)?.audible,'audio ready before gain checks');
  const read=async()=> (await invoke('refresh_tab_media',{tabId:id})).tabs.find(t=>t.tab_id===id);
  let state=await until(async()=>{const s=await read();return s.document_key&&s.frames.some(f=>f.count)?s:null;},'media documents');
  const set=async volume=>{state=(await invoke('set_tab_volume',{tabId:id,volume,documentKey:state.document_key})).tabs.find(t=>t.tab_id===id);assert.equal(state.applied_volume,volume);};
  await set(50);await until(()=>fixture.reports.get('media-volume')?.nativeVolume===0.5,'native attenuation');
  assert.equal(fixture.reports.get('media-volume').playerVolume,1,'player volume stays independent');
  await set(200);await until(()=>fixture.reports.get('media-volume')?.audioRms>0.005,'amplified waveform');
  const doubled=fixture.reports.get('media-volume').audioRms;
  await set(150);await until(()=>fixture.reports.get('media-volume')?.audioGain===1.5,'gain 150');
  const ratio=fixture.reports.get('media-volume').audioRms/doubled;
  assert.ok(ratio>0.65&&ratio<0.85,`waveform gain ratio ${ratio}`);
  await set(0);assert.equal(state.muted,false,'zero volume is independent of mute');
  await invoke('set_tab_muted',{tabId:id,muted:true});await set(100);assert.equal(state.muted,true);await invoke('set_tab_muted',{tabId:id,muted:false});
  await click('[data-testid="open-media"]');await until(()=>script('return document.querySelector("[data-testid=media-volume]")?.value==="100"'),'media panel');
  await script('const el=document.querySelector("[data-testid=media-volume]");for(const value of [20,40,60,80]){el.value=value;el.dispatchEvent(new Event("input",{bubbles:true}));}');
  await until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id)?.applied_volume===80,'latest slider value');
  await script('document.querySelector("[role=dialog]").dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}));');
  await until(()=>script('return document.activeElement?.matches("[data-testid=open-media]")'),'focus returns');
  const old=state.document_key;
  await invoke('navigate_to',{input:fixture.origin+'/page/media-next'});await until(()=>fixture.reports.has('media-next'),'next document');
  await assert.rejects(()=>invoke('set_tab_volume',{tabId:id,volume:150,documentKey:old}),/media.stale/);
  state=await read();await set(100);
  fixture.commands.set('media-next','cross-audio');await until(()=>fixture.reports.get('media-next')?.reportedAt>0,'cross source');
  await new Promise(r=>setTimeout(r,400));state=await read();
  const limited=(await invoke('set_tab_volume',{tabId:id,volume:200,documentKey:state.document_key})).tabs.find(t=>t.tab_id===id);
  assert.equal(limited.applied_volume,100);assert.equal(limited.volume_status,'limited');
  await until(()=>fixture.reports.get('media-next')?.nativeMuted===false,'unsupported audio preserved');
  await restart();const restored=(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id);assert.equal(restored.volume,100);
}
