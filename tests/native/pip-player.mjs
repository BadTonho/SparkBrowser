import assert from 'node:assert/strict';

export async function withFloatingPlayer({request,sessionId,script},operation) {
  const base=`/session/${sessionId()}`;
  const previous=await request('GET',base+'/window');
  try {
    for(const handle of await request('GET',base+'/window/handles')) {
      await request('POST',base+'/window',{handle});
      if((await script('return location.href')).startsWith('about:blank#spark-pip-'))return await operation();
    }
    throw new Error('Custom floating player not found');
  } finally { await request('POST',base+'/window',{handle:previous}); }
}

export async function checkFloatingVolume(context,id) {
  const {script,invoke,until,wait,request,sessionId,screenshot}=context;
  const applied=async value=>{
    try {await until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id)?.applied_volume===value,`PiP applies ${value}%`);}
    catch(error){const state=(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id);console.error('Fixture volume state',JSON.stringify({requested:value,applied:state?.applied_volume,status:state?.volume_status}));throw error;}
  };
  await withFloatingPlayer(context,async()=>{
    await until(()=>script('const v=document.querySelector("video");return v?.readyState>=2&&v.videoWidth>0;'),'silent moving image');
    assert.equal(await script('return document.querySelector("video").muted && document.querySelector("video").srcObject.getAudioTracks().length===0;'),true,'copy has no audio');
    const before=await script('return document.querySelector("video").getVideoPlaybackQuality().totalVideoFrames');
    await wait(600);
    assert.ok(await script('return document.querySelector("video").getVideoPlaybackQuality().totalVideoFrames>arguments[0]',before),'image keeps advancing');
    const speaker=await script('const r=document.querySelector("[data-testid=pip-speaker]").getBoundingClientRect();return {x:Math.round(r.left+r.width/2),y:Math.round(r.top+r.height/2)};');
    await request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'pointer',id:'pip-pointer',parameters:{pointerType:'mouse'},actions:[{type:'pointerMove',duration:0,...speaker}]}]});
    assert.equal(await script('return document.querySelector("#pip-volume-panel").hidden'),false);
    const ui=await script(`const p=document.querySelector('#pip-volume-panel'),s=p.querySelector('input'),v=p.querySelector('span'),r=p.getBoundingClientRect(),b=document.querySelector('[data-testid=pip-speaker]').getBoundingClientRect(),t=document.querySelector('.time').getBoundingClientRect(),sr=s.getBoundingClientRect(),vr=v.getBoundingClientRect(),css=getComputedStyle(p);
      return {width:r.width,height:r.height,gap:b.left-r.right,center:(r.top+r.height/2)-(b.top+b.height/2),timeClear:t.right<=r.left,inside:r.left>=0&&r.right<=innerWidth,left:vr.right<=sr.left,aligned:(vr.top+vr.height/2)===(sr.top+sr.height/2),inputHeight:sr.height,appearance:getComputedStyle(s).appearance,background:css.backgroundColor,borders:[css.borderTopWidth,css.borderRightWidth,css.borderBottomWidth,css.borderLeftWidth],radius:css.borderRadius,shadow:css.boxShadow,min:s.min,max:s.max,step:s.step};`);
    assert.equal(ui.width,120);assert.equal(ui.height,28);assert.equal(ui.gap,6);assert.equal(ui.center,0);assert.equal(ui.timeClear,true);assert.equal(ui.inside,true);assert.equal(ui.left,true);assert.equal(ui.aligned,true);assert.equal(ui.inputHeight,28);assert.equal(ui.appearance,'none');assert.equal(ui.background,'rgba(0, 0, 0, 0)');assert.deepEqual(ui.borders,['0px','0px','0px','0px']);assert.equal(ui.radius,'0px');assert.equal(ui.shadow,'none');assert.deepEqual([ui.min,ui.max,ui.step],['0','200','1']);
    const percent=await script('const r=document.querySelector("[data-testid=pip-volume-percentage]").getBoundingClientRect();return {x:Math.round(r.left+r.width/2),y:Math.round(r.top+r.height/2)};');
    await request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'pointer',id:'pip-pointer',parameters:{pointerType:'mouse'},actions:[{type:'pointerMove',duration:100,...percent}]}]});
    await wait(400);assert.equal(await script('return document.querySelector("#pip-volume-panel").hidden'),false,'hover over percentage keeps the entire volume panel open');
    await screenshot('pip-volume-hover');
    const original=await request('GET',`/session/${sessionId()}/window/rect`);
    await request('POST',`/session/${sessionId()}/window/rect`,{width:280,height:158});
    assert.equal(await script('const r=document.querySelector("#pip-volume-panel").getBoundingClientRect();return r.left>=0&&r.right<=innerWidth&&r.top>=0&&r.bottom<=innerHeight;'),true,'volume stays inside a resized floating window');
    await request('POST',`/session/${sessionId()}/window/rect`,original);
    // Resizing moves the panel away from the pointer. Reopen it through keyboard
    // focus before exercising values and real pointer drags.
    await script('document.querySelector("[data-testid=pip-speaker]").focus();document.querySelector("[data-testid=pip-volume]").focus();');
  });
  for(const value of [0,100,200]) {
    const ui=await withFloatingPlayer(context,()=>script('const s=document.querySelector("[data-testid=pip-volume]"),p=document.querySelector("[data-testid=pip-volume-percentage]");s.value=arguments[0];s.dispatchEvent(new Event("input",{bubbles:true}));s.dispatchEvent(new Event("change",{bubbles:true}));return {text:p.textContent,fill:s.style.getPropertyValue("--volume-progress"),width:p.getBoundingClientRect().width};',value));
    assert.equal(ui.text,`${value}%`);assert.equal(ui.fill,`${value/2}%`);assert.equal(ui.width,36);
    await applied(value);
  }
  for(const value of [0,200]) {
    await withFloatingPlayer(context,async()=>{
      const r=await script('const r=document.querySelector("[data-testid=pip-volume]").getBoundingClientRect();return {left:r.left,right:r.right,y:Math.round(r.top+r.height/2)};');
      await request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'pointer',id:'pip-pointer',parameters:{pointerType:'mouse'},actions:[
        {type:'pointerMove',duration:0,x:Math.round((r.left+r.right)/2),y:r.y},{type:'pointerDown',button:0},
        {type:'pointerMove',duration:150,x:Math.round(value===0?r.left+1:r.right-1),y:r.y},{type:'pointerUp',button:0},
      ]}]});
      assert.equal(await script('return document.querySelector("[data-testid=pip-volume-percentage]").textContent'),`${value}%`);
    });
    await applied(value);
  }
  await withFloatingPlayer(context,async()=>{
    const element=await request('POST',`/session/${sessionId()}/element`,{using:'css selector',value:'[data-testid=pip-volume]'});
    await request('POST',`/session/${sessionId()}/element/${element['element-6066-11e4-a52e-4f735466cecf']}/value`,{text:'\uE012',value:['\uE012']});
    assert.equal(await script('return document.querySelector("[data-testid=pip-volume-percentage]").textContent'),'199%');
    await script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}));');
    assert.equal(await script('return document.querySelector("#pip-volume-panel").hidden && document.activeElement.matches("[data-testid=pip-speaker]")'),true);
  });
  await applied(199);
  const tab=(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id);
  await invoke('set_tab_volume',{tabId:id,volume:100,documentKey:tab.document_key});
  await withFloatingPlayer(context,()=>until(()=>script('return document.querySelector("[data-testid=pip-volume-percentage]").textContent==="100%"'),'PiP follows browser volume'));
  const peer=(await invoke('create_tab',{url:context.fixture.origin+'/page/pip-moving-peer'})).tabs.find(t=>t.is_active).id;
  await withFloatingPlayer(context,async()=>{
    const frames=await script('return document.querySelector("video").getVideoPlaybackQuality().totalVideoFrames');
    await wait(700);
    assert.ok(await script('return document.querySelector("video").getVideoPlaybackQuality().totalVideoFrames>arguments[0]',frames),'image advances while the source tab is hidden');
    await script('document.activeElement?.blur();');
    const speaker=await script('const r=document.querySelector("[data-testid=pip-speaker]").getBoundingClientRect();return {x:Math.round(r.left+r.width/2),y:Math.round(r.top+r.height/2)};');
    await request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'pointer',id:'pip-pointer',parameters:{pointerType:'mouse'},actions:[{type:'pointerMove',duration:0,...speaker},{type:'pointerMove',duration:0,x:5,y:5}]}]});
    await until(()=>script('return document.querySelector("#pip-volume-panel").hidden;'),'hover dismissal is not throttled by the hidden source tab',800);
  });
  await invoke('switch_tab',{tabId:id});await invoke('close_tab',{tabId:peer});
}

export async function checkBackgroundFloatingVolume(context,id,other) {
  const {script,invoke,until}=context;
  await withFloatingPlayer(context,()=>script('const s=document.querySelector("[data-testid=pip-volume]");s.value="42";s.dispatchEvent(new Event("input",{bubbles:true}));s.dispatchEvent(new Event("change",{bubbles:true}));'));
  await until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id)?.applied_volume===42,'PiP controls source tab from another workspace');
  assert.equal((await invoke('get_tabs')).tabs.find(t=>t.is_active).id,other);
  assert.equal((await invoke('get_tab_media')).tabs.find(t=>t.tab_id===other)?.applied_volume,100,'focused peer volume is unchanged');
  await withFloatingPlayer(context,()=>script('document.querySelector("[data-testid=pip-return]").click();'));
  await until(async()=> (await invoke('get_tabs')).tabs.find(t=>t.is_active)?.id===id,'return button selects source tab and workspace');
  await invoke('switch_tab',{tabId:other});
}

export async function checkFloatingFrameChurn(context,id) {
  const {invoke,fixture,until,script,withWebview}=context;
  const before=(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id).document_key;
  fixture.commands.set('media-pip','media-frame');
  await until(()=>fixture.reports.has('media-frame'),'unrelated dynamic iframe');
  await withFloatingPlayer(context,()=>script('const s=document.querySelector("[data-testid=pip-volume]");s.value="65";s.dispatchEvent(new Event("input",{bubbles:true}));s.dispatchEvent(new Event("change",{bubbles:true}));'));
  await until(async()=>{const s=(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id);return s.applied_volume===65&&s.document_key!==before;},'volume survives unrelated iframe changes');
  // Keep discovery on another tab so it cannot repair the invalidated source
  // key before the floating player edits volume, as on ad-heavy pages.
  const peer=(await invoke('create_tab',{url:fixture.origin+'/page/pip-frame-churn-peer'})).tabs.find(t=>t.is_active).id;
  fixture.commands.set('media-pip','remove-iframe');
  await withWebview('/page/media-pip',()=>until(()=>script('return !document.querySelector("iframe")'),'remove unrelated iframe'));
  await until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id)?.document_key==='','iframe removal invalidates the aggregate key');
  await assert.rejects(()=>invoke('set_tab_volume',{tabId:id,volume:75,documentKey:''}),/media.stale/,'ordinary edits still require a current key');
  await withFloatingPlayer(context,()=>script('const s=document.querySelector("[data-testid=pip-volume]");s.value="75";s.dispatchEvent(new Event("input",{bubbles:true}));s.dispatchEvent(new Event("change",{bubbles:true}));'));
  await until(async()=> (await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id)?.applied_volume===75,'revalidated PiP session survives the cleared aggregate key');
  await invoke('switch_tab',{tabId:id});await invoke('close_tab',{tabId:peer});
}
