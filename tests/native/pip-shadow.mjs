import assert from 'node:assert/strict';
import {openControlledPip} from './pip-continuity.mjs';
import {withFloatingPlayer} from './pip-player.mjs';

export async function checkPipShadow(context) {
  const {invoke,script,until,wait,click,restart,fixture,floatingWindow,withWebview}=context;
  const setting='[data-testid=setting-pip-shadow]';
  async function openSetting(search=false) {
    await context.openSettings();
    if(search){
      await script('const s=document.querySelector("#settings-search-input");s.value="sombra";s.dispatchEvent(new Event("input",{bubbles:true}));');
      await until(()=>script('return [...document.querySelectorAll(".settings-search-result")].some(b=>b.textContent.includes("Sombra"));'),'translated shadow search result');
      await script('[...document.querySelectorAll(".settings-search-result")].find(b=>b.textContent.includes("Sombra")).click();');
    }else await click('[data-testid=settings-appearance_layout]');
    await until(()=>script('return document.querySelector(arguments[0])?.disabled===false;',setting),'shadow preference ready');
  }
  const close=()=>context.closeSettings();
  async function change(value){await click(setting);await until(()=>script('const b=document.querySelector(arguments[0]);return !b.disabled&&b.getAttribute("aria-pressed")===arguments[1];',setting,String(value)),'shadow setting confirms');}
  await openSetting(true);
  assert.equal(await script('return document.querySelector(arguments[0]).getAttribute("aria-pressed")',setting),'false');
  await close();
  const id=await openControlledPip(context),path='/page/media-pip-continuity';
  await withWebview(path,()=>script('document.documentElement.style.background="#777";document.body.style.background="#777";'));
  const owner=async()=>(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id).frames.find(f=>f.pip_session).pip_session;
  const session=await owner();
  const off=await floatingWindow('Capture','pip-shadow-off');
  const original=await withFloatingPlayer(context,()=>script('return {width:innerWidth,height:innerHeight,frames:document.querySelector("video").getVideoPlaybackQuality().totalVideoFrames};'));
  await openSetting();await change(true);await close();
  await wait(200);
  const on=await floatingWindow();assert.equal(on.handle,off.handle);assert.equal(await owner(),session);
  const dimensions=await withFloatingPlayer(context,()=>script('return {width:innerWidth,height:innerHeight};'));
  assert.deepEqual(dimensions,{width:original.width,height:original.height},'live shadow preserves client size');
  assert.equal(on.left,off.left);assert.equal(on.top,off.top);
  await floatingWindow('Capture','pip-shadow-on');
  const beforeMinimize=await floatingWindow();
  await withFloatingPlayer(context,()=>script('document.querySelector("[data-testid=pip-speaker]").click();const s=document.querySelector("[data-testid=pip-volume]");s.value="64";s.dispatchEvent(new Event("input",{bubbles:true}));s.dispatchEvent(new Event("change",{bubbles:true}));'));
  await until(async()=>(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id).applied_volume===64,'source volume');
  await withFloatingPlayer(context,()=>script('document.querySelector("[data-testid=pip-minimize]").click();'));
  const peer=(await invoke('create_tab',{url:fixture.origin+'/page/pip-shadow-peer'})).tabs.find(t=>t.is_active).id;
  await invoke('create_workspace',{id:'pip-shadow-workspace',name:'Shadow test',icon:'T',accentPrimary:'#888888',accentSecondary:'#999999'});
  await invoke('move_tab_to_workspace',{tabId:peer,workspaceId:'pip-shadow-workspace'});await invoke('switch_workspace',{workspaceId:'pip-shadow-workspace'});
  await openSetting();await change(false);await close();
  assert.equal((await floatingWindow()).minimized,true,'shadow change preserves minimized state');
  await floatingWindow('Restore');assert.equal(await owner(),session);
  const restored=await floatingWindow();
  assert.equal(restored.left,beforeMinimize.left);assert.equal(restored.top,beforeMinimize.top);
  assert.deepEqual(await withFloatingPlayer(context,()=>script('return {width:innerWidth,height:innerHeight};')),{width:original.width,height:original.height},'restoring after a minimized shadow change preserves the video area');
  assert.equal((await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id).applied_volume,64);
  await withFloatingPlayer(context,()=>until(()=>script('return document.querySelector("video").getVideoPlaybackQuality().totalVideoFrames>arguments[0];',original.frames),'playback continues'));
  await openSetting();await change(true);await close();await invoke('exit_video_pip');await restart();
  await openSetting();assert.equal(await script('return document.querySelector(arguments[0]).getAttribute("aria-pressed");',setting),'true');await close();
  const restoredId=await openControlledPip(context);
  await invoke('exit_video_pip');await openSetting();await change(false);await close();
  assert.equal(await script('return localStorage.getItem("veloce_pip_window_shadow");'),'false');
  assert.ok((await invoke('get_tabs')).tabs.some(t=>t.id===restoredId));
}
