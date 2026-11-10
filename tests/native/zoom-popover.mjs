import assert from 'node:assert/strict';
import { withScenarioDatabase } from './persistence/database.mjs';

export function zoomUi(context) {
  const { click, script, until, withWebview } = context;
  const expanded = value => until(() => script('const button=document.querySelector("[data-testid=open-zoom]");return button?button.getAttribute("aria-expanded")===arguments[0]:arguments[0]==="false";', String(value)), 'zoom expanded '+value);
  const open = async () => {
    await until(()=>context.focusWindow().then(()=>true,()=>false),'isolated zoom test window receives focus');
    await until(()=>script('const e=document.querySelector("[data-testid=open-zoom]"),r=e?.getBoundingClientRect();return e&&!e.disabled&&r.width>0&&r.left>=0&&r.right<=innerWidth&&r.bottom<=innerHeight&&innerHeight>=480;'),'zoom trigger is visible after native window changes');
    await click('[data-testid=open-zoom]'); await expanded(true);
  };
  const inside = action => withWebview('/zoom-popover.html', async () => {
    await until(() => script('return !document.querySelector("#zoom-popover").hidden'), 'native zoom panel');
    return action();
  });
  const close = async () => { await inside(()=>script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))')); await expanded(false); };
  const percentage = async value => {
    await open();
    await inside(()=>until(()=>script('return document.querySelector("#zoom-percentage").textContent===arguments[0]',value+'%'),'panel percentage '+value));
    await close();
  };
  return { open, inside, close, percentage, expanded };
}

export async function checkZoomPopover(context, a, b) {
  const { invoke, script, click, fixture, until, request, sessionId, resizeWindow, wait } = context;
  const ui=zoomUi(context);
  const factor=()=>invoke('get_site_zoom').then(s=>s.sites.find(row=>row.origin===fixture.origin)?.factor??1);
  const settled=Date.now();
  await until(()=>fixture.reports.get('a')?.visibility==='visible'&&fixture.reports.get('a').reportedAt>=settled,'settled active fixture before opening zoom');
  const initial=fixture.reports.get('a');
  assert.equal(await script('return document.querySelector("[data-testid=open-zoom]").textContent.trim()==="" && !document.querySelector(".omnibox-wrapper [data-testid=zoom-reset]")'),true,'address contains only the zoom icon');
  await ui.open();
  const anchor=await script('const r=document.querySelector("[data-testid=open-zoom]").getBoundingClientRect();return {left:r.left,bottom:r.bottom,width:r.width}');
  await ui.inside(async()=>{
    const rect=await script('const r=document.querySelector("#zoom-popover").getBoundingClientRect();return {left:r.left,top:r.top,width:r.width}');
    assert.equal(rect.width,240); assert.ok(Math.abs(rect.top-anchor.bottom-6)<=1);
    assert.ok(Math.abs(rect.left+120-anchor.left-anchor.width/2)<=1);
    await context.screenshot('zoom-popover');
    assert.equal(await script('return document.querySelector("#zoom-percentage").textContent'),'100%');
    await script('location.href="veloce-internal://zoom-popover/zoom?token=invalid&sequence=999&value=5"');
  });
  await wait(150);assert.equal(await factor(),1,'unauthenticated adjustment rejected');
  await ui.close();
  assert.equal(fixture.reports.get('a').hiddenCount,initial.hiddenCount,'opening never hides native pages');
  assert.equal(fixture.reports.get('a').scroll,initial.scroll,'opening preserves scrolling');

  await ui.open();
  await ui.inside(async()=>{
    const r=await script('const r=document.querySelector("#page-zoom").getBoundingClientRect();return {left:r.left+8,width:r.width-16,y:Math.round(r.top+r.height/2)}');
    await request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'pointer',id:'zoom-slider-mouse',parameters:{pointerType:'mouse'},actions:[
      {type:'pointerMove',duration:0,x:Math.round(r.left+r.width*7/16),y:r.y},{type:'pointerDown',button:0},
      {type:'pointerMove',duration:180,x:Math.round(r.left+r.width*10/16),y:r.y}]}]});
  });
  await until(async()=>await factor()===1.5,'slider applies while mouse remains pressed');
  await ui.inside(()=>request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'pointer',id:'zoom-slider-mouse',parameters:{pointerType:'mouse'},actions:[{type:'pointerUp',button:0}]}]}));
  await ui.close();
  await until(()=>Math.abs(fixture.reports.get('a').pixelRatio/initial.pixelRatio-1.5)<.01,'live page zoom');

  await invoke('set_site_zoom',{origin:fixture.origin,factor:1.37});
  await ui.percentage(137);assert.equal(await factor(),1.37,'opening does not round a legacy factor');
  await invoke('set_site_zoom',{origin:fixture.origin,factor:1.5});
  await ui.open();
  await withScenarioDatabase(context.directory,async db=>{
    db.exec("CREATE TRIGGER fail_zoom BEFORE INSERT ON site_zoom BEGIN SELECT RAISE(ABORT,'injected zoom failure'); END");
    try {
      await ui.inside(()=>click('[data-testid=zoom-in]'));
      await ui.inside(()=>until(()=>script('return !document.querySelector("#zoom-error").hidden && document.querySelector("#zoom-percentage").textContent==="150%"'),'write failure restores confirmed percentage'));
      assert.equal(await factor(),1.5);
    } finally { db.exec('DROP TRIGGER fail_zoom'); }
  });
  await ui.inside(()=>click('[data-testid=zoom-in]'));
  await until(async()=>await factor()===1.75,'failed zoom can retry');
  await ui.inside(()=>click('[data-testid=zoom-reset]'));
  await until(async()=>await factor()===1,'reset restores 100%');
  await ui.close();

  for (const reason of ['outside','trigger','tab']) {
    console.log('Zoom dismissal: '+reason);
    await ui.open();
    const trigger=await script('const r=document.querySelector("[data-testid=open-zoom]").getBoundingClientRect();return {x:r.left+r.width/2,y:r.top+r.height/2}');
    await ui.inside(()=>reason==='tab'
      ?script('document.querySelector("#zoom-reset").focus();window.dispatchEvent(new KeyboardEvent("keydown",{key:"Tab",bubbles:true,cancelable:true}))')
      :script('document.dispatchEvent(new PointerEvent("pointerdown",{bubbles:true,clientX:arguments[0],clientY:arguments[1]}))',reason==='outside'?2:trigger.x,reason==='outside'?300:trigger.y));
    await ui.expanded(false);
  }
  await invoke('set_split_view',{leftTabId:a.id,rightTabId:b.id});
  await ui.open();await invoke('switch_tab',{tabId:b.id});await ui.expanded(false);
  await ui.open();await script('document.querySelector("[data-testid=open-media]").click()');await ui.expanded(false);
  await invoke('set_zoom_popover',{visible:true,sessionId:0,presentation:null,restoreFocus:false});
  await context.withWebview('/media-popover.html',()=>script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))'));
  await ui.open();await script('document.querySelector("[data-testid=open-privacy]").click()');await ui.expanded(false);
  await context.withWebview('/privacy-popover.html',()=>script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))'));
  await ui.open();
  await invoke('set_media_popover',{visible:true,sessionId:0,presentation:null,restoreFocus:false});
  await wait(100);assert.equal(await script('return document.querySelector("[data-testid=open-zoom]").getAttribute("aria-expanded")'),'true','stale media presentation cannot close zoom');
  await script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"z",ctrlKey:true,shiftKey:true,bubbles:true,cancelable:true}))');
  await ui.expanded(false);
  await until(()=>script('return !document.querySelector(".titlebar")'),'Zen hides zoom trigger');
  await script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"z",ctrlKey:true,shiftKey:true,bubbles:true,cancelable:true}))');
  await until(()=>script('return Boolean(document.querySelector("[data-testid=open-zoom]"))'),'leaving Zen restores zoom trigger');
  await ui.open();await context.openSettings();await ui.expanded(false);
  await click('[data-testid=settings-appearance_layout]');
  await script('document.querySelectorAll("#setting-browser-layout button")[1].click();');
  await context.closeSettings();
  await ui.open(); await resizeWindow(680,740);
  await ui.inside(()=>until(()=>script('const r=document.querySelector("#zoom-popover").getBoundingClientRect();return r.left>=8&&r.right<=innerWidth-8&&r.top<80'),'vertical minimum-width placement'));
  await ui.close();await resizeWindow(1280,820);
  await context.openSettings();await click('[data-testid=settings-appearance_layout]');
  await script('document.querySelectorAll("#setting-browser-layout button")[0].click();');await context.closeSettings();
  await ui.open();await invoke('switch_workspace',{workspaceId:'ws-work'});await ui.expanded(false);
  await invoke('switch_workspace',{workspaceId:a.workspace_id});
  await invoke('switch_tab',{tabId:a.id});
  await invoke('set_split_view',{leftTabId:null,rightTabId:null});
  await until(()=>Math.abs(fixture.reports.get('a').pixelRatio/initial.pixelRatio-1)<.01,'helper restores default zoom');
  assert.equal(fixture.loads.get('a'),1);assert.equal(fixture.loads.get('b'),1);
  assert.equal(fixture.reports.get('a').value,'artificial-test-draft');
  assert.equal((await invoke('get_tabs')).tabs.find(t=>t.id===a.id).navigation.can_go_back,true);
}
