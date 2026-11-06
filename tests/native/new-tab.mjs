import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {resolve} from 'node:path';
import {readFileSync} from 'node:fs';
import { withScenarioDatabase } from './persistence/database.mjs';
import {checkNewTabWidgets} from './new-tab-widgets.mjs';
import {newTabDriver} from './new-tab-driver.mjs';

export async function checkNewTab(context) {
  const {invoke,script,click,until,fixture,resizeWindow,request,sessionId,restart}=context;
  const {inside,ready,active}=newTabDriver(context);
  assert.equal((await active()).url,'veloce://newtab','empty profile starts locally');
  await ready();
  await inside(async()=>{
    assert.equal(await script('return document.activeElement.id'),'search','central search has initial focus');
    assert.equal(await script('return [...document.querySelectorAll("link[href],script[src],img[src]")].every(n=>new URL(n.href||n.src).origin===location.origin)'),true);
    assert.equal(await script('return document.querySelectorAll(".recent-site").length'),0);
    assert.equal(await script('const c=document.querySelector(".content").getBoundingClientRect();return Math.abs(c.top+c.height/2-(innerHeight/2-80))<=2'),true,'content is 80px above center');
  });
  assert.equal(await script('return document.querySelector(".omnibox-input").value'),'');
  assert.equal(await script('return document.querySelector(".star-btn").disabled && document.querySelector(".omnibox-icon").disabled && document.querySelector("[data-testid=open-zoom]").disabled'),true);
  assert.equal((await invoke('get_history')).length,0,'internal document is never a history visit');

  await withScenarioDatabase(context.directory,db=>{
    const insert=db.prepare('INSERT INTO history(title,url,visited_at) VALUES(?,?,?)');
    for(let i=0;i<220;i++)insert.run('Search','https://google.com/search?q=ignored',1000+i);
    for(const [i,url] of ['http://www.one.test/old','https://one.test/last?q=a%26b#part','https://two.test/page','https://two.test:8443/port','https://three.test/page','https://four.test/page'].entries())insert.run('Artificial page '+i,url,10+i);
    db.prepare('INSERT INTO site_icons VALUES(?,?,?)').run('four.test',fixture.icons.png,1);
  });
  await invoke('set_new_tab_preferences',{engine:'duckduckgo',language:'pt-BR'});
  await inside(async()=>{
    await until(()=>script('return document.querySelectorAll(".recent-site").length===5'),'five older distinct sites');
    assert.deepEqual(await script('return [...document.querySelectorAll(".site-domain")].map(n=>n.textContent)'),['four.test','three.test','two.test:8443','two.test','one.test']);
    await until(()=>script('const e=document.querySelector(".site-icon");return e.src.startsWith("data:image/png;base64,")&&e.complete&&e.naturalWidth===1'),'local cached icon renders');
    assert.equal(await script('const e=document.querySelector(".site-icon").getBoundingClientRect();return e.width===24&&e.height===24'),true);
    assert.equal(await script('return document.querySelectorAll(".site-icon")[1].getAttribute("src")'),'/new-tab/site-fallback.svg');
    await script('window.staleRecentCard=document.querySelector(".recent-site")');
    await context.screenshot('new-tab-recent');
    await script('document.querySelector("#search").value="draft preserved";document.querySelector("#search").focus()');
  });
  await invoke('set_new_tab_preferences',{engine:'google',language:'en'});
  await inside(async()=>{
    await until(()=>script('return document.documentElement.lang==="en"'),'language update');
    assert.equal(await script('return document.querySelector("#search").value'),'draft preserved');
  });
  await invoke('clear_browsing_history');
  await inside(()=>until(()=>script('return document.querySelectorAll(".recent-site").length===0 && !document.querySelector("#empty").hidden'),'history clear refreshes existing page'));
  await withScenarioDatabase(context.directory,db=>assert.equal(db.prepare('SELECT COUNT(*) AS n FROM site_icons').get().n,0));
  await inside(()=>script('window.staleRecentCard.click()'));
  await context.wait(150);assert.equal((await active()).url,'veloce://newtab','cleared card cannot navigate using an old snapshot');
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
  await until(()=>withScenarioDatabase(context.directory,db=>db.prepare('SELECT COUNT(*) AS n FROM site_icons').get().n===1),'native favicon captured into cache');
  const iconRequests=fixture.icons.requests.length;
  await invoke('go_back');await ready();assert.equal((await active()).url,'veloce://newtab');
  await inside(async()=>{
    await until(()=>script('const e=document.querySelector(".site-icon");return e?.src.startsWith("data:image/png;base64,")&&e.complete&&e.naturalWidth>0'),'captured favicon renders');
    assert.equal(fixture.icons.requests.length,iconRequests,'opening new tab does not request a favicon');
    await script('document.querySelector(".recent-site").click()');
  });
  await until(()=>fixture.loads.get('new-tab-first')>=2,'card opens exact last visited page');
  await until(async()=>{const tab=await active();return tab.url===fixture.origin+'/page/new-tab-first'&&!tab.navigation.loading;},'card navigation settles');
  await invoke('go_back');await ready();
  await invoke('go_forward');await until(async()=>{const tab=await active();return tab.url===fixture.origin+'/page/new-tab-first'&&!tab.navigation.loading;},'forward returns to page');
  await invoke('go_back');await ready();
  await inside(()=>script('document.querySelector("#search").value=arguments[0];document.querySelector("#search-form").requestSubmit()',fixture.origin+'/page/new-tab-returned'));
  await until(()=>fixture.reports.has('new-tab-returned'),'search remains authenticated after returning through history');
  await invoke('go_back');await ready();
  await invoke('reload_page');await ready();
  const late=(await invoke('create_tab',{url:fixture.origin+'/page/new-tab-late'})).tabs.find(t=>t.is_active);
  await until(()=>fixture.icons.pending.length>0,'delayed native favicon is in flight');
  await invoke('clear_browsing_history');fixture.icons.release();
  await context.wait(300);
  await withScenarioDatabase(context.directory,db=>assert.equal(db.prepare('SELECT COUNT(*) AS n FROM site_icons').get().n,0,'late pre-clear favicon cannot repopulate cache'));
  await invoke('navigate_to',{input:fixture.origin+'/page/new-tab-bad'});
  await until(()=>fixture.reports.has('new-tab-bad'),'invalid favicon fixture loaded');
  await invoke('navigate_to',{input:'veloce://newtab'});await ready();
  await inside(()=>until(()=>script('return document.querySelector(".site-icon")?.getAttribute("src")==="/new-tab/site-fallback.svg"'),'invalid native image falls back'));
  await invoke('close_tab',{tabId:late.id});await ready();
  const secondPacket=await invoke('create_tab');const second=secondPacket.tabs.find(t=>t.is_active);
  await ready();
  await invoke('set_split_view',{leftTabId:first.id,rightTabId:second.id});
  // Only the active new-tab view is selected by path helper; use its field and submit a loopback URL.
  await inside(()=>script('document.querySelector("#search").value=arguments[0];document.querySelector("#search-form").requestSubmit()',fixture.origin+'/page/new-tab-right'));
  await until(()=>fixture.reports.has('new-tab-right'),'right split page navigation');
  await withScenarioDatabase(context.directory,db=>db.prepare('INSERT INTO history(title,url,visited_at) VALUES(?,?,?)').run('Left card destination',fixture.origin+'/page/new-tab-left-card?exact=1',Math.floor(Date.now()/1000)+1));
  await invoke('set_new_tab_preferences',{engine:'google',language:'en'});
  await inside(()=>until(()=>script('return document.querySelector(".site-title")?.textContent==="Left card destination"'),'left card updated'),first.id);
  await inside(()=>click('.recent-site'),first.id);
  await until(async()=>{const p=await invoke('get_tabs');return p.tabs.find(t=>t.id===first.id).url===fixture.origin+'/page/new-tab-left-card?exact=1'&&p.tabs.find(t=>t.id===second.id).url===fixture.origin+'/page/new-tab-right';},'card navigates its own pane only');
  await invoke('go_back');await ready();
  const rightLoads=fixture.loads.get('new-tab-right');
  const rightDocument=fixture.reports.get('new-tab-right');
  assert.equal((await invoke('get_tabs')).tabs.find(t=>t.id===first.id).url,'veloce://newtab');
  await until(()=>context.focusWindow().then(()=>true,()=>false),'isolated new-tab test receives foreground focus');
  for(const layout of ['vertical','horizontal']) {
    await click('[data-testid=open-settings]');
    await context.withWebview('/index.html?surface=settings-quick',async()=>{
      await click(`[data-testid=settings-quick-layout-${layout}]`);
      await script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))');
      await until(()=>script('return !document.querySelector("[data-testid=settings-quick-menu]")'),'quick settings closes');
    });
    await until(()=>script('return localStorage.getItem("veloce_tab_layout")===arguments[0]',layout),'layout changes');
  }
  await invoke('switch_tab',{tabId:first.id});
  await invoke('reload_page');await ready();
  await resizeWindow(680,600);
  await inside(()=>{
    return script('const r=document.querySelector("#search-form").getBoundingClientRect(),c=document.querySelector(".content").getBoundingClientRect(),m=document.querySelector("main").getBoundingClientRect();if(r.left<23||r.right>innerWidth-23||c.top<31)throw Error("search overflows pane");const offset=Math.min(80,Math.max(0,(innerHeight/2-160)/2));if(m.height<=innerHeight+1&&Math.abs(c.top+c.height/2-(innerHeight/2-offset))>2)throw Error("search vertical offset is wrong")');
  });
  await resizeWindow(680,480);
  await inside(()=>script('const c=document.querySelector(".content").getBoundingClientRect();if(c.top<31||document.documentElement.scrollWidth>innerWidth)throw Error("small-window content clipped")'));
  await resizeWindow(680,600);
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
  await until(()=>withScenarioDatabase(context.directory,db=>db.prepare('SELECT COUNT(*) AS n FROM site_icons').get().n>0),'icon cache populated before restart');
  const cached=await withScenarioDatabase(context.directory,db=>db.prepare('SELECT site_key,hex(png) AS image FROM site_icons ORDER BY site_key').all());
  await restart();
  assert.equal((await active()).id,restored);assert.equal((await active()).url,'veloce://newtab');await ready();
  const after=await withScenarioDatabase(context.directory,db=>db.prepare('SELECT site_key,hex(png) AS image FROM site_icons ORDER BY site_key').all());
  for(const entry of cached)assert.ok(after.some(row=>row.site_key===entry.site_key&&row.image===entry.image),'favicon persists across restart');
  await context.screenshot('new-tab');
  await checkNewTabWidgets(context,{inside,ready,active});
}
