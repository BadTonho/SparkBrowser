import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {resolve} from 'node:path';
import {readFileSync} from 'node:fs';
import { withScenarioDatabase } from './persistence/database.mjs';

export async function checkNewTab(context) {
  const {invoke,script,click,until,fixture,resizeWindow,request,sessionId,restart}=context;
  const inside=async(action,id)=>{
    const target=id??(await active()).id;
    const previous=await request('GET',`/session/${sessionId()}/window`);
    try {
      await until(async()=>{
        const handles=await request('GET',`/session/${sessionId()}/window/handles`);
        for(const handle of handles){
          try {await request('POST',`/session/${sessionId()}/window`,{handle});if(await script('return window.__veloceNewTabId===arguments[0]',target))return true;}catch(error){if(!/no such window|web view not found/i.test(String(error)))throw error;}
        }
        return false;
      },'new tab WebView '+target);
      return await action();
    } finally {await request('POST',`/session/${sessionId()}/window`,{handle:previous});}
  };
  const ready=async()=>{
    await until(async()=>{const tab=await active();return tab.url==='veloce://newtab'&&!tab.navigation.loading;},'internal navigation settled');
    return inside(()=>until(()=>script('return Boolean(document.querySelector("#search")?.placeholder && document.querySelector("#recent")?.dataset.key)'),'local new tab ready'));
  };
  const active=()=>invoke('get_tabs').then(p=>p.tabs.find(t=>t.is_active));
  assert.equal((await active()).url,'veloce://newtab','empty profile starts locally');
  await ready();
  await inside(async()=>{
    assert.equal(await script('return document.activeElement.id'),'search','central search has initial focus');
    assert.equal(await script('return [...document.querySelectorAll("link[href],script[src],img[src]")].every(n=>new URL(n.href||n.src).origin===location.origin)'),true);
    assert.equal(await script('return document.querySelectorAll(".recent-search").length'),0);
  });
  assert.equal(await script('return document.querySelector(".omnibox-input").value'),'');
  assert.equal(await script('return document.querySelector(".star-btn").disabled && document.querySelector(".omnibox-icon").disabled && document.querySelector("[data-testid=open-zoom]").disabled'),true);
  assert.equal((await invoke('get_history')).length,0,'internal document is never a history visit');

  await withScenarioDatabase(context.directory,db=>{
    const insert=db.prepare('INSERT INTO history(title,url,visited_at) VALUES(?,?,?)');
    for(let i=0;i<220;i++)insert.run('Fixture','https://example.test/'+i,1000+i);
    for(const [i,url] of ['https://www.google.com/search?q=Rust+%26+Svelte','https://duckduckgo.com/?q=rust+%26+svelte','https://www.google.com.br/search?q=second','https://html.duckduckgo.com/html/?q=third','https://lite.duckduckgo.com/lite/?q=fourth','https://google.com/search?q=fifth'].entries())insert.run('Artificial search',url,10+i);
  });
  await invoke('set_new_tab_preferences',{engine:'duckduckgo',language:'pt-BR'});
  await inside(async()=>{
    await until(()=>script('return document.querySelectorAll(".recent-search").length===5'),'five older unique searches');
    assert.deepEqual(await script('return [...document.querySelectorAll(".query")].map(n=>n.textContent)'),['fifth','fourth','third','second','rust & svelte']);
    await context.screenshot('new-tab-recent');
    await script('document.querySelector("#search").value="draft preserved";document.querySelector("#search").focus()');
  });
  await invoke('set_new_tab_preferences',{engine:'google',language:'en'});
  await inside(async()=>{
    await until(()=>script('return document.documentElement.lang==="en"'),'language update');
    assert.equal(await script('return document.querySelector("#search").value'),'draft preserved');
  });
  await invoke('clear_browsing_history');
  await inside(()=>until(()=>script('return document.querySelectorAll(".recent-search").length===0 && !document.querySelector("#empty").hidden'),'history clear refreshes existing page'));
  // Fault injection stays inside the disposable test database.
  await withScenarioDatabase(context.directory,async db=>{
    db.exec('ALTER TABLE history RENAME TO unavailable_history');
    try {
      await inside(()=>script('window.__veloceRefreshNewTab()'));
      await inside(()=>until(()=>script('return !document.querySelector("#error").hidden'),'read failure appears'));
    } finally {db.exec('ALTER TABLE unavailable_history RENAME TO history');}
  });
  await inside(()=>click('#retry'));
  await inside(()=>until(()=>script('return document.querySelector("#error").hidden'),'retry recovers'));

  await inside(()=>script('location.href="veloce-internal://new-tab/navigate?token=invalid&document=fake&sequence=999&input=https://example.test"'));
  assert.equal((await active()).url,'veloce://newtab');
  await inside(()=>script('const e=document.querySelector("#search");e.value=arguments[0];document.querySelector("#search-form").requestSubmit()',fixture.origin+'/page/new-tab-first'));
  await until(()=>fixture.reports.has('new-tab-first'),'search form navigates originating page');
  const first=await active();
  await invoke('go_back');await ready();assert.equal((await active()).url,'veloce://newtab');
  await invoke('go_forward');await until(async()=>{const tab=await active();return tab.url===fixture.origin+'/page/new-tab-first'&&!tab.navigation.loading;},'forward returns to page');
  await invoke('go_back');await ready();
  await inside(()=>script('document.querySelector("#search").value=arguments[0];document.querySelector("#search-form").requestSubmit()',fixture.origin+'/page/new-tab-returned'));
  await until(()=>fixture.reports.has('new-tab-returned'),'search remains authenticated after returning through history');
  await invoke('go_back');await ready();
  await invoke('reload_page');await ready();
  const secondPacket=await invoke('create_tab');const second=secondPacket.tabs.find(t=>t.is_active);
  await ready();
  await invoke('set_split_view',{leftTabId:first.id,rightTabId:second.id});
  // Only the active new-tab view is selected by path helper; use its field and submit a loopback URL.
  await inside(()=>script('document.querySelector("#search").value=arguments[0];document.querySelector("#search-form").requestSubmit()',fixture.origin+'/page/new-tab-right'));
  await until(()=>fixture.reports.has('new-tab-right'),'right split page navigation');
  const rightLoads=fixture.loads.get('new-tab-right');
  const rightDocument=fixture.reports.get('new-tab-right');
  assert.equal((await invoke('get_tabs')).tabs.find(t=>t.id===first.id).url,'veloce://newtab');
  for(const layout of ['vertical','horizontal']) {
    await click('[data-testid=open-settings]');
    await click('[data-testid=settings-appearance_layout]');
    await script('document.querySelectorAll("#setting-browser-layout button")[arguments[0]==="horizontal"?0:1].click()',layout);
    await script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))');
  }
  await until(()=>context.focusWindow().then(()=>true,()=>false),'isolated new-tab test receives foreground focus');
  await invoke('switch_tab',{tabId:first.id});
  await invoke('reload_page');await ready();
  await resizeWindow(680,600);
  await inside(()=>{
    return script('const r=document.querySelector("#search-form").getBoundingClientRect();if(r.left<23||r.right>innerWidth-23)throw Error("search overflows pane")');
  });
  await inside(()=>click('#search'),first.id);
  await inside(()=>request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'key',id:'new-tab-keys',actions:[{type:'keyDown',value:'\uE004'},{type:'keyUp',value:'\uE004'}]}]}));
  await inside(()=>until(()=>script('return document.activeElement.id==="submit"'),'Tab reaches search button'));
  if(process.platform==='win32') {
    const build=JSON.parse(readFileSync('.cache/native-target/test-build.json','utf8'));
    assert.equal(build.identifier,'org.veloce.browser.native-tests');
    const executable=resolve(build.executable);assert.ok(executable.startsWith(resolve('.cache/native-target')+'\\'));
    const key=(code,up=false)=>new Promise((done,fail)=>{
      const child=spawn('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',resolve('tests/native/windows-input.ps1'),'-Executable',executable,'-Key',String(code),...(up?['-Up']:[])],{windowsHide:true,stdio:['ignore','ignore','pipe']});
      let error='';child.stderr.on('data',chunk=>error+=chunk);child.on('error',fail);child.on('exit',code=>code===0?done():fail(Error(error||'Native new-tab keyboard input failed')));
    });
    try {await key(0x11);await key(0x4C);}finally {
      const released=await Promise.allSettled([key(0x4C,true),key(0x11,true)]);
      const failure=released.find(result=>result.status==='rejected');if(failure)throw failure.reason;
    }
  } else {
    await inside(()=>request('POST',`/session/${sessionId()}/actions`,{actions:[{type:'key',id:'new-tab-keys',actions:[{type:'keyDown',value:'\uE009'},{type:'keyDown',value:'l'},{type:'keyUp',value:'l'},{type:'keyUp',value:'\uE009'}]}]}));
  }
  await until(()=>script('return document.hasFocus()&&document.activeElement.matches(".omnibox-input")'),'Ctrl+L focuses shared address');
  assert.equal(fixture.loads.get('new-tab-right'),rightLoads,'new-tab interactions preserve the companion document');
  assert.equal(fixture.reports.get('new-tab-right').value,rightDocument.value);
  assert.equal(fixture.reports.get('new-tab-right').scroll,rightDocument.scroll);
  await invoke('close_tab',{tabId:first.id});
  await invoke('reopen_closed_tab');await ready();
  const reopened=(await active()).id;
  await invoke('switch_tab',{tabId:second.id});
  await invoke('hibernate_tab',{tabId:reopened});
  assert.equal((await invoke('get_tabs')).tabs.find(t=>t.id===reopened).is_hibernated,true);
  await invoke('switch_tab',{tabId:reopened});await ready();
  await invoke('save_named_session',{name:'Local new tab'});
  const saved=(await invoke('get_saved_sessions')).find(s=>s.name==='Local new tab');
  assert.ok(saved.pages.some(p=>p.url==='veloce://newtab'));
  await invoke('restore_named_session',{id:saved.id});await ready();
  const restored=(await active()).id;
  await restart();
  assert.equal((await active()).id,restored);assert.equal((await active()).url,'veloce://newtab');await ready();
  await context.screenshot('new-tab');
}
