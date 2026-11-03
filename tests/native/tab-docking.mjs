import assert from 'node:assert/strict';
import { withScenarioDatabase } from './persistence/database.mjs';
import { inspectDragWindows } from './tab-drag-window.mjs';

export async function checkTabDocking(context) {
  const { invoke, until, fixture, script, click, request, sessionId, restart, resizeWindow, wait } = context;
  await invoke('navigate_to', { input: fixture.origin + '/page/dock-a' });
  await until(() => fixture.reports.has('dock-a'), 'first docking page');
  const a = (await invoke('get_tabs')).tabs.find(t => t.is_active);
  const b = (await invoke('create_tab', { url: fixture.origin + '/page/dock-b' })).tabs.find(t => t.is_active);
  await until(() => fixture.reports.has('dock-b'), 'second docking page');
  fixture.commands.set('dock-b', 'spa');
  await until(async () => (await invoke('get_tabs')).tabs.find(t => t.id === b.id).navigation.can_go_back, 'docking history');
  const c = (await invoke('create_tab', { url: fixture.origin + '/page/dock-c' })).tabs.find(t => t.is_active);
  await until(() => fixture.reports.has('dock-c'), 'third docking page');
  await invoke('switch_tab', { tabId: a.id });
  await until(() => script('return document.querySelector(".omnibox-input").value.endsWith("/page/dock-a")'), 'shell docking selection');
  await wait(250);
  const order = (await invoke('get_tabs')).tabs.map(t => t.id);
  const anchors = new Map(['dock-a','dock-b','dock-c'].map(name=>[name,fixture.reports.get(name).scrollAnchor]));
  await script(`window.dockingEvents=[];for(const type of ['pointerdown','pointerup','lostpointercapture','blur'])window.addEventListener(type,event=>{window.dockingEvents.push([type,event.clientX,event.clientY,event.pointerId]);if(window.dockingEvents.length>24)window.dockingEvents.shift();},true);`);
  const pointer = async actions => request('POST', `/session/${sessionId()}/actions`, { actions: [{ type: 'pointer', id: 'dock-mouse', parameters: { pointerType: 'mouse' }, actions }] });
  const box = () => script('const r=document.querySelector(".viewport-frame").getBoundingClientRect();return {left:r.left,top:r.top,width:r.width,height:r.height}');
  const point = id => script('const r=document.querySelector(`[data-tab-id="${arguments[0]}"] .tab-title`).getBoundingClientRect();return {x:Math.round(r.left+Math.min(25,r.width/2)),y:Math.round(r.top+r.height/2)}', id);
  const up = () => pointer([{ type: 'pointerUp', button: 0 }]);
  let previewSide;
  const holdAtEdge = async (id, side, viaTabId) => {
    previewSide=side;
    const p=await point(id),r=await box();
    // Keep the crossing in one W3C action sequence: EdgeDriver can release
    // native capture between separate actions requests targeting child WebViews.
    const via=viaTabId?await point(viaTabId):null;
    await pointer([{type:'pointerMove',duration:0,...p},{type:'pointerDown',button:0},
      {type:'pointerMove',duration:60,x:p.x+8,y:p.y},
      ...(via?[{type:'pointerMove',duration:180,...via}]:[]),
      {type:'pointerMove',duration:180,x:Math.round(r.left+r.width/2),y:Math.round(r.top+r.height/2)},
      {type:'pointerMove',duration:180,x:Math.round(side==='left'?r.left+12:r.left+r.width-12),y:Math.round(r.top+r.height/2)}]);
  };
  const checkPreview = async () => {
    if (process.platform === 'win32') {
      let windows;
      try { windows=await until(async()=>{const all=await inspectDragWindows();return all.some(w=>w.title==='Veloce split preview' && w.visible)?all:null;},'preview window visible'); }
      catch(error){console.log('Drag diagnostics',await script('return {focus:document.hasFocus(),drag:document.querySelector(".is-dragging")?.dataset.tabId,error:document.querySelector("[role=alert]")?.textContent}'));console.log('Native windows',await inspectDragWindows());throw error;}
      const preview=windows.find(w=>w.title==='Veloce split preview');
      assert.equal(preview.owner,true);assert.equal(preview.foreground,false);
      const r=await box();assert.ok(Math.abs(preview.height/preview.scale-r.height)<=2);
      const main=windows.find(window=>window.main);
      const available=Math.round(r.width)-8, left=available*((await invoke('get_tabs')).split_layout?.ratio??0.5);
      const width=previewSide==='left'?left:available-left;
      const x=Math.round(r.left)+(previewSide==='left'?0:left+8);
      assert.ok(Math.abs(preview.width/preview.scale-width)<=2,'preview width matches pane');
      assert.ok(Math.abs((preview.left-main.clientLeft)/preview.scale-x)<=2,'preview horizontal position matches pane');
      assert.ok(Math.abs((preview.top-main.clientTop)/preview.scale-Math.round(r.top))<=2,'preview vertical position matches page');
    }
  };
  const dock = async (id, side) => {
    await holdAtEdge(id, side); await checkPreview(); await up();
    try { await until(async () => (await invoke('get_tabs')).split_layout?.[side + '_tab_id'] === id, 'docked ' + side); }
    catch(error){console.log('Drop diagnostics',await script('return {events:window.dockingEvents,focus:document.hasFocus(),error:document.querySelector("[role=alert]")?.textContent}'));throw error;}
  };
  console.log('Docking: horizontal crossing native page');
  await context.focusWindow();
  await holdAtEdge(b.id,'left',a.id); await checkPreview();
  assert.equal((await invoke('get_tabs')).split_layout, null, 'preview does not split');
  assert.equal((await invoke('get_tabs')).tabs.find(t => t.is_active).id, a.id);
  await context.screenshot('dock-preview');
  await up();
  await until(async () => (await invoke('get_tabs')).split_layout?.left_tab_id === b.id, 'left drop');
  assert.equal((await invoke('get_tabs')).split_layout.right_tab_id, a.id);
  assert.equal((await invoke('get_tabs')).tabs.find(t => t.is_active).id, b.id);
  assert.deepEqual((await invoke('get_tabs')).tabs.map(t => t.id), order, 'docking does not reorder');
  const split = (await invoke('get_tabs')).split_layout;
  await invoke('commit_split_ratio', { key: `${split.workspace_id}:${split.left_tab_id}:${split.right_tab_id}`, ratio: 0.65 });
  await dock(c.id, 'right');
  assert.equal((await invoke('get_tabs')).split_layout.ratio, 0.65);
  assert.equal((await invoke('get_tabs')).split_layout.left_tab_id, b.id);
  assert.ok((await invoke('get_tabs')).tabs.some(t => t.id === a.id), 'replaced page stays open');
  await dock(b.id, 'right');
  assert.equal((await invoke('get_tabs')).split_layout.left_tab_id, c.id, 'paired tab swaps sides');
  assert.equal((await invoke('get_tabs')).split_layout.right_tab_id, b.id);
  assert.equal((await invoke('get_tabs')).tabs.find(t => t.id === b.id).navigation.can_go_back, true);

  const beforeCancel = (await invoke('get_tabs')).split_layout;
  await holdAtEdge(a.id,'left');
  await request('POST', `/session/${sessionId()}/actions`, { actions: [{ type: 'key', id: 'dock-key', actions: [{ type: 'keyDown', value: '\uE00C' }, { type: 'keyUp', value: '\uE00C' }] }] });
  await up(); assert.deepEqual((await invoke('get_tabs')).split_layout, beforeCancel);
  await until(() => script('return !document.querySelector(".is-dragging") && document.body.style.cursor!=="grabbing"'), 'drag cancelled');
  console.log('Docking: injected persistence failure');
  await invoke('get_history', { limit: 1 });
  await withScenarioDatabase(context.directory, async db => {
    const saved = db.prepare('SELECT * FROM workspace_splits').all();
    db.exec("CREATE TRIGGER fail_dock BEFORE INSERT ON workspace_splits BEGIN SELECT RAISE(ABORT,'injected docking failure'); END");
    try {
      await holdAtEdge(a.id,'left'); await checkPreview(); await up();
      await until(() => script('return document.querySelector("[role=alert]")?.textContent.length>0'), 'visible docking error');
      assert.deepEqual((await invoke('get_tabs')).split_layout, beforeCancel);
      assert.equal((await invoke('get_tabs')).tabs.find(t=>t.id===b.id).navigation.can_go_back,true);
      assert.deepEqual(db.prepare('SELECT * FROM workspace_splits').all(), saved);
    } finally { db.exec('DROP TRIGGER fail_dock'); }
  });
  await dock(a.id, 'left');
  console.log('Docking: vertical tabs and resize');
  await click('[data-testid=open-settings]'); await click('[data-testid=settings-appearance_layout]');
  await script('document.querySelectorAll("#setting-browser-layout button")[1].click();document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}));');
  await until(() => script('return Boolean(document.querySelector(".tab-row"))'), 'vertical tabs');
  await wait(300); await resizeWindow(1000, 740); await wait(300);
  await dock(c.id, 'right');
  assert.equal((await invoke('get_tabs')).split_layout.left_tab_id, a.id);
  assert.equal((await invoke('get_tabs')).split_layout.ratio, 0.65);
  for (const name of ['dock-a', 'dock-b', 'dock-c']) {
    assert.equal(fixture.loads.get(name), 1, name + ' retains its document');
    assert.equal(fixture.reports.get(name).value, 'artificial-test-draft');
    assert.ok(Math.abs(fixture.reports.get(name).scrollAnchor-anchors.get(name))<=1, name+' preserves its visible scroll anchor across reflow');
    assert.equal(fixture.reports.get(name).pageClicks,0,name+' receives no drag/drop click');
  }
  const saved = (await invoke('get_tabs')).split_layout;
  await restart(); assert.deepEqual((await invoke('get_tabs')).split_layout, saved);
  console.log('Docking: reorder and workspace transfer remain available');
  await context.focusWindow();
  await script(`window.reorderEvents=[];const root=document.querySelector('.browser-root');for(const type of ['pointerdown','pointermove','pointerup','lostpointercapture','blur'])window.addEventListener(type,e=>window.reorderEvents.push({type,x:e.clientX,y:e.clientY,buttons:e.buttons,capture:root.hasPointerCapture(e.pointerId||1),target:document.elementFromPoint(e.clientX||0,e.clientY||0)?.closest('[data-tab-id]')?.dataset.tabId,drag:document.querySelector('.is-dragging')?.dataset.tabId,order:[...document.querySelectorAll('.tab-row')].map(t=>t.dataset.tabId)}),true);`);
  const previousOrder=(await invoke('get_tabs')).tabs.map(tab=>tab.id);
  const dropOn = async (id,target) => {
    const p=await point(id);
    await pointer([{type:'pointerMove',duration:0,...p},{type:'pointerDown',button:0},
      {type:'pointerMove',duration:60,x:p.x+8,y:p.y},{type:'pointerMove',duration:180,x:Math.round(target.x),y:Math.round(target.y)},{type:'pointerUp',button:0}]);
  };
  await dropOn(b.id,await point(a.id));
  try { await until(async()=> (await invoke('get_tabs')).tabs.map(tab=>tab.id).join()!==previousOrder.join(),'normal tab reorder'); }
  catch(error){console.log('Reorder diagnostics',JSON.stringify(await script('return {events:window.reorderEvents,focus:document.hasFocus(),error:document.querySelector("[role=alert]")?.textContent}')));throw error;}
  const workspace=await script('const r=document.querySelector("[data-ws-id=ws-work]").getBoundingClientRect();return {x:r.left+r.width/2,y:r.top+r.height/2}');
  await dropOn(b.id,workspace);
  await until(async()=> (await invoke('get_tabs')).tabs.find(tab=>tab.id===b.id).workspace_id==='ws-work','workspace drag transfer');
  await invoke('set_split_view',{leftTabId:null,rightTabId:null});
  await invoke('switch_tab',{tabId:a.id});
  await until(()=>script('return document.querySelector(".tab-row.active")?.dataset.tabId===arguments[0]',a.id),'active drag source');
  await until(()=>script('return !document.querySelector(".split-divider") && document.querySelectorAll(".tab-row").length===2'),'transfer and split removal rendered');
  await wait(250);
  const other=(await invoke('get_tabs')).tabs.filter(tab=>tab.workspace_id==='ws-personal'&&tab.id!==a.id)
    .reduce((best,tab)=>!best||tab.last_active_epoch>best.last_active_epoch?tab:best,null);
  await dock(a.id,'right');
  assert.equal((await invoke('get_tabs')).split_layout.left_tab_id,other.id,'active drag uses recent other tab');
}
