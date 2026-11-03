import assert from 'node:assert/strict';
import { withScenarioDatabase } from './persistence/database.mjs';

export async function checkSplitTabGroups(context) {
  const { invoke, until, fixture, click, script, request, sessionId, resizeWindow, restart, wait } = context;
  const packet = () => invoke('get_tabs');
  const key = split => `${split.workspace_id}:${split.left_tab_id}:${split.right_tab_id}`;
  const dragContext = p => ({ workspaceId:p.split_layout.workspace_id, activeTabId:p.tabs.find(t=>t.is_active).id, splitKey:key(p.split_layout), groupsKey:JSON.stringify([p.split_layout.left_tab_ids,p.split_layout.right_tab_ids]) });
  await invoke('navigate_to', {input:fixture.origin+'/page/groups-a'});
  const a = (await packet()).tabs.find(t=>t.is_active);
  const created = [];
  for (const name of ['b','c','d']) created.push((await invoke('create_tab',{url:fixture.origin+'/page/groups-'+name})).tabs.find(t=>t.is_active));
  const [b,c,d] = created;
  await until(()=>['a','b','c','d'].every(name=>fixture.reports.has('groups-'+name)), 'group fixture documents');
  await invoke('set_split_view',{leftTabId:a.id,rightTabId:b.id});
  let p=await packet();
  assert.deepEqual(p.split_layout.left_tab_ids,[a.id,c.id,d.id]);
  assert.deepEqual(p.split_layout.right_tab_ids,[b.id]);
  const checkLists = async () => until(async()=>{
    const current=await packet();
    const dom=await script('return ["left","right"].map(side=>[...document.querySelectorAll(`[data-testid=split-tabs-${side}] [data-tab-id]`)].map(tab=>tab.dataset.tabId))');
    return JSON.stringify(dom)===JSON.stringify([current.split_layout.left_tab_ids,current.split_layout.right_tab_ids]);
  },'each horizontal list matches its persisted group');
  await checkLists();
  await context.screenshot('split-tab-groups');
  await click(`[data-tab-id="${c.id}"] .tab-title`);
  await until(async()=>(await packet()).split_layout.left_tab_id===c.id,'left tab selection');
  assert.equal((await packet()).split_layout.right_tab_id,b.id);
  await click('[data-testid=new-tab-right]');
  const rightNew=await until(async()=>{const q=await packet();return q.tabs.find(t=>!p.tabs.some(old=>old.id===t.id));},'right plus creates a tab');
  await invoke('navigate_to',{input:fixture.origin+'/page/groups-new-right'});
  p=await packet(); assert.equal(p.split_layout.right_tab_id,rightNew.id); assert.equal(p.split_layout.left_tab_id,c.id);
  await click('[data-testid=new-tab-left]');
  const leftNew=await until(async()=>{const q=await packet();return q.tabs.find(t=>!p.tabs.some(old=>old.id===t.id));},'left plus creates a tab');
  await invoke('navigate_to',{input:fixture.origin+'/page/groups-new-left'});
  p=await packet(); assert.equal(p.split_layout.left_tab_id,leftNew.id); assert.equal(p.split_layout.right_tab_id,rightNew.id);
  await context.focusWindow();
  await request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'key',id:'groups-key',actions:[{type:'keyDown',value:'\uE009'},{type:'keyDown',value:'\uE004'},{type:'keyUp',value:'\uE004'},{type:'keyUp',value:'\uE009'}]}]});
  await until(async()=>(await packet()).split_layout.left_tab_id===a.id,'Ctrl+Tab wraps within the focused group');
  assert.equal((await packet()).split_layout.right_tab_id,rightNew.id);
  for (const width of [1160,680]) {
    await resizeWindow(width,780);
    for (const ratio of [.2,.5,.8]) {
      p=await packet(); await invoke('commit_split_ratio',{key:key(p.split_layout),ratio});
      await until(()=>script('const f=document.querySelector(".viewport-frame").getBoundingClientRect(),l=document.querySelector("[data-testid=split-tabs-left]").getBoundingClientRect(),r=document.querySelector("[data-testid=split-tabs-right]").getBoundingClientRect(),w=document.querySelector(".window-controls").getBoundingClientRect();const available=Math.max(200,Math.round(f.width)-8),left=Math.max(100,Math.min(available-100,available*arguments[0]));return Math.abs(l.left-Math.round(f.left))<=2&&Math.abs(l.width-left)<=2&&Math.abs(r.left-(Math.round(f.left)+left+8))<=2&&r.right<=w.left;',ratio),'tab bars align with native panes without overlapping window controls');
      if(width===680&&ratio===.8)await context.screenshot('split-tab-groups-small');
    }
  }
  await resizeWindow(1280,820);
  p=await packet();await invoke('commit_split_ratio',{key:key(p.split_layout),ratio:.5});
  await invoke('switch_tab',{tabId:c.id}); await invoke('switch_tab',{tabId:b.id});
  const point=id=>script('const e=document.querySelector(`[data-tab-id="${arguments[0]}"] .tab-title`),r=e.getBoundingClientRect();return {x:Math.round(r.left+Math.min(20,r.width/2)),y:Math.round(r.top+r.height/2)}',id);
  await context.focusWindow();await wait(200);
  const from=await point(c.id),to=await point(b.id);
  await request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'pointer',id:'groups-mouse',parameters:{pointerType:'mouse'},actions:[{type:'pointerMove',duration:0,...from},{type:'pointerDown',button:0},{type:'pointerMove',duration:60,x:from.x+8,y:from.y},{type:'pointerMove',duration:180,...to},{type:'pointerUp',button:0}]}]});
  await until(async()=>(await packet()).split_layout.right_tab_ids.includes(c.id),'real drag transfers a tab to the opposite bar');
  assert.equal((await packet()).split_layout.right_tab_id,c.id);
  await checkLists();
  p=await packet();
  await withScenarioDatabase(context.directory,async db=>{
    db.exec("CREATE TRIGGER fail_group_move BEFORE INSERT ON workspace_splits BEGIN SELECT RAISE(ABORT,'injected group failure'); END");
    try {
      await assert.rejects(invoke('move_tab_in_split',{tabId:d.id,side:'right',beforeId:b.id,context:dragContext(p)}),/injected group failure/);
      assert.deepEqual((await packet()).split_layout,p.split_layout);
      assert.deepEqual((await packet()).tabs.map(t=>t.id),p.tabs.map(t=>t.id));
    } finally {db.exec('DROP TRIGGER fail_group_move');}
  });
  await invoke('move_tab_in_split',{tabId:d.id,side:'right',beforeId:b.id,context:dragContext(p)});
  await assert.rejects(invoke('move_tab_in_split',{tabId:a.id,side:'right',beforeId:null,context:dragContext(p)}),/context changed/);
  await click('[data-testid=open-settings]');await click('[data-testid=settings-appearance_layout]');
  await script('document.querySelectorAll("#setting-browser-layout button")[1].click();document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}));');
  await until(()=>script('return Boolean(document.querySelector(".tab-row"))&&!document.querySelector("[data-testid=split-tabs-left]")'),'vertical layout uses one list');
  await invoke('switch_tab',{tabId:a.id});
  assert.equal((await packet()).split_layout.left_tab_id,a.id);
  assert.equal((await packet()).split_layout.right_tab_id,d.id);
  await click('[data-testid=open-settings]');await click('[data-testid=settings-appearance_layout]');
  await script('document.querySelectorAll("#setting-browser-layout button")[0].click();document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}));');
  await checkLists();
  for (const name of ['a','b','c','d']) {
    assert.equal(fixture.loads.get('groups-'+name),1,'group operations reuse resident documents');
    assert.equal(fixture.reports.get('groups-'+name).value,'artificial-test-draft');
  }
  const saved=(await packet()).split_layout;
  await restart();assert.deepEqual((await packet()).split_layout,saved);await checkLists();
  await invoke('close_tab',{tabId:d.id});
  p=await packet();assert.ok(p.split_layout,'closing a selected tab keeps a nonempty group');
  assert.equal(p.split_layout.right_tab_id,b.id,'selected removal chooses the next group neighbor');
  for(const id of [...p.split_layout.right_tab_ids])await invoke('close_tab',{tabId:id});
  assert.equal((await packet()).split_layout,null,'closing the last group member ends split');
}
