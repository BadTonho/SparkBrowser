import assert from 'node:assert/strict';

export async function checkVolumePopover(context, id) {
  const { invoke, until, fixture, click, script, withWebview, screenshot, request, sessionId, resizeWindow } = context;
  const hiddenBefore = fixture.reports.get('media-volume').hiddenCount;
  const button = await script('const r=document.querySelector("[data-testid=open-media]").getBoundingClientRect();return {left:r.left,top:r.top,width:r.width,height:r.height};');
  await click('[data-testid="open-media"]');
  await until(()=>script('return document.querySelector("[data-testid=open-media]")?.getAttribute("aria-expanded")==="true"'),'volume popover opens');
  await withWebview('/media-popover.html', async()=>{
    await until(()=>script('return !document.querySelector("#media-popover").hidden && document.querySelector("[data-testid=media-volume]")?.value==="100"'),'native slider ready');
    const ui=await script('const p=document.querySelector("#media-popover"),r=p.getBoundingClientRect(),s=document.querySelector("input"),v=document.querySelector("#volume-percentage"),sr=s.getBoundingClientRect(),vr=v.getBoundingClientRect();return {width:r.width,height:r.height,left:r.left,top:r.top,text:p.textContent.trim(),inputs:p.querySelectorAll("input").length,buttons:p.querySelectorAll("button").length,dialog:!!document.querySelector("[role=dialog]"),focused:document.activeElement===s,label:s.getAttribute("aria-label"),min:s.min,max:s.max,step:s.step,percentageBelow:vr.top>=sr.bottom,percentageCentered:Math.abs(vr.left+vr.width/2-sr.left-sr.width/2)<1,percentageFont:getComputedStyle(v).fontSize};');
    assert.equal(ui.width,320);assert.ok(ui.height>=64 && ui.height<=440);
    assert.ok(Math.abs(ui.top-(button.top+button.height+6))<=1,`six pixel gap below trigger: ${JSON.stringify({ui,button})}`);
    assert.ok(Math.abs(ui.left-(button.left+button.width/2-160))<=1,`centered beneath trigger: ${JSON.stringify({ui,button})}`);
    assert.ok(ui.text.includes('100%'));assert.equal(ui.inputs,1);assert.equal(ui.dialog,false);assert.equal(ui.focused,true);assert.ok(ui.label);
    assert.equal(ui.percentageBelow,true);assert.equal(ui.percentageCentered,true);assert.equal(ui.percentageFont,'12px');
    assert.deepEqual([ui.min,ui.max,ui.step],['0','200','1']);
    await screenshot('volume-popover');
  });
  const applied = async value => until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id)?.applied_volume===value,`slider applies ${value}%`);
  for (const value of [0,100,200]) {
    const displayed=await withWebview('/media-popover.html',()=>script('const s=document.querySelector("input");s.value=arguments[0];s.dispatchEvent(new Event("input",{bubbles:true}));return document.querySelector("#volume-percentage").textContent;',value));
    assert.equal(displayed,`${value}%`,'percentage follows input immediately');
    await applied(value);
  }
  const documentKey=(await invoke('get_tab_media')).tabs.find(tab=>tab.tab_id===id).document_key;
  for (const value of [100,200]) {
    await invoke('set_tab_volume',{tabId:id,volume:value,documentKey});
    await withWebview('/media-popover.html',()=>until(()=>script('return document.querySelector("#volume-percentage").textContent===arguments[0] && document.querySelector("input").value+"%"===arguments[0];',`${value}%`),'percentage follows applied native volume'));
  }
  await withWebview('/media-popover.html',async()=>{
    const element=await request('POST',`/session/${sessionId()}/element`,{using:'css selector',value:'input'});
    await request('POST',`/session/${sessionId()}/element/${element['element-6066-11e4-a52e-4f735466cecf']}/value`,{text:'\uE012',value:['\uE012']});
    assert.equal(await script('return document.querySelector("#volume-percentage").textContent;'),'199%','percentage follows arrow keys');
  });
  await applied(199);
  for (const value of [0,200]) {
    await withWebview('/media-popover.html',async()=>{
      const rect=await script('const r=document.querySelector("input").getBoundingClientRect();return {left:r.left,right:r.right,y:r.top+r.height/2};');
      await request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'pointer',id:'volume-pointer',parameters:{pointerType:'mouse'},actions:[
        {type:'pointerMove',duration:0,x:Math.round((rect.left+rect.right)/2),y:Math.round(rect.y)},
        {type:'pointerDown',button:0},
        {type:'pointerMove',duration:150,x:Math.round(value===0?rect.left+1:rect.right-1),y:Math.round(rect.y)},
        {type:'pointerUp',button:0},
      ]}]});
      assert.equal(await script('return document.querySelector("#volume-percentage").textContent;'),`${value}%`,'percentage follows pointer dragging');
    });
    await applied(value);
  }
  await withWebview('/media-popover.html',()=>script('const el=document.querySelector("input");for(const value of [20,40,60,80]){el.value=value;el.dispatchEvent(new Event("input",{bubbles:true}));}window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}));'));
  await applied(80);
  await until(()=>script('return document.activeElement?.matches("[data-testid=open-media]")'),'focus returns');
  await until(()=>fixture.reports.get('media-volume')?.visibility==='visible','page stays visible');
  assert.equal(fixture.reports.get('media-volume').hiddenCount,hiddenBefore,'popover never hides the page');
  assert.equal(fixture.loads.get('media-volume'),1,'audio controls preserve the document');
  assert.equal(fixture.reports.get('media-volume').value,'artificial-test-draft');

  const expanded = value => until(()=>script('return document.querySelector("[data-testid=open-media]")?.getAttribute("aria-expanded")===arguments[0]',String(value)),`popover ${value?'opens':'closes'}`);
  const open = async()=>{ await click('[data-testid=open-media]'); await expanded(true); };
  for (const reason of ['outside','trigger']) {
    await open();
    await withWebview('/media-popover.html',()=>script('document.dispatchEvent(new PointerEvent("pointerdown",{bubbles:true,clientX:arguments[0],clientY:arguments[1]}));',reason==='trigger'?button.left+button.width/2:10,reason==='trigger'?button.top+button.height/2:300));
    await expanded(false);
  }
  for (let i=0;i<3;i++) {
    await open();
    await script('document.querySelector("[data-testid=open-media]").click();');
    await expanded(false);
  }
  await open();
  await resizeWindow(900,700);
  const resized=await until(()=>script('const r=document.querySelector("[data-testid=open-media]").getBoundingClientRect();return innerWidth===900?{left:r.left,top:r.top,width:r.width,height:r.height}:null;'),'resized button');
  await withWebview('/media-popover.html',async()=>{
    await until(()=>script('const r=document.querySelector("#media-popover").getBoundingClientRect(),a=arguments[0];return innerWidth===900 && Math.abs(r.top-a.top-a.height-6)<=1 && r.left>=8 && r.right<=innerWidth-8 && Math.abs(r.left-Math.max(8,Math.min(a.left+a.width/2-160,innerWidth-328)))<=1;',resized),'popover follows resize');
  });
  await script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}));');
  await expanded(false);
  await resizeWindow(1280,820);

  for (const layout of [1,0]) {
    await context.openSettings();await click('[data-testid=settings-appearance_layout]');
    await script('document.querySelectorAll("#setting-browser-layout button")[arguments[0]].click();',layout);
    await context.closeSettings();
    await open();
    await withWebview('/media-popover.html',()=>until(()=>script('const p=document.querySelector("#media-popover"),r=p.getBoundingClientRect();return !p.hidden && r.height>=64 && r.height<=440 && document.querySelector("#volume-percentage").textContent===document.querySelector("input").value+"%";'),'popover in both tab layouts'));
    // Opening a shell overlay dismisses the volume control before hiding content.
    await script('document.querySelector("[data-testid=open-settings]").click();');
    await expanded(false);
    await context.closeSettings();
  }
  await open();
  const other=(await invoke('create_tab',{url:fixture.origin+'/page/volume-popover-context'})).tabs.find(tab=>tab.is_active).id;
  await expanded(false);
  await open();
  await invoke('navigate_to',{input:fixture.origin+'/page/volume-popover-next'});
  await expanded(false);
  await invoke('switch_tab',{tabId:id});
  await invoke('close_tab',{tabId:other});
  await applied(80);
}
