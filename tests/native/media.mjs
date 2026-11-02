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
