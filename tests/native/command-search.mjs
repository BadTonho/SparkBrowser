import assert from 'node:assert/strict';
import { prepareTabs } from './setup.mjs';
export async function checkCommandSearch(context) {
  const { invoke, fixture, script, until, click, wait, request, sessionId } = context;
  const { a,b } = await prepareTabs(context);
  const original=(await invoke('get_tabs')).tabs.find(t=>t.id===a.id).url;
  const noteUrl=fixture.origin+'/page/saved-note?q=1';
  await invoke('save_page_note',{url:noteUrl,title:'Café research',text:'100%_ café <b>plain text</b> '+ 'x'.repeat(400)});
  await invoke('save_page_note',{url:fixture.origin+'/page/other-note',title:'Other café note',text:'CAFE reference'});
  await invoke('save_named_session',{name:'Café collection'});
  const session=(await invoke('get_saved_sessions')).find(s=>s.name==='Café collection');
  const before=(await invoke('get_tabs')).tabs.length;
  const query=async value=>{
    await script(`document.dispatchEvent(new KeyboardEvent('keydown',{key:'k',ctrlKey:true,bubbles:true,cancelable:true}));`);
    await until(()=>script('return Boolean(document.querySelector(".command-input"))'),'command bar');
    // Wait for the opening select/focus timer before setting the test query.
    await wait(30);
    await script(`const input=document.querySelector('.command-input'); input.value=arguments[0];input.dispatchEvent(new Event('input',{bubbles:true}));`,value);
  };
  await query('cafe');
  await until(()=>script('return Boolean(document.querySelector("[data-command-id=navigate-current]"))'),'navigation before search replies');
  await script(`document.querySelector('[data-command-id=navigate-current]').dispatchEvent(new MouseEvent('mouseenter',{bubbles:false}));`);
  await until(()=>script('return document.querySelectorAll("[data-command-id^=note-]").length===2 && document.querySelectorAll("[data-command-id^=session-]").length===1'),'note and session results');
  assert.equal(await script('return document.querySelector(".command-item.selected").dataset.commandId'),'navigate-current','selection survives arriving results');
  assert.ok(await script('return document.querySelectorAll(".command-item").length<=12'));
  assert.equal(await script('return document.querySelectorAll("[data-command-id=open-new-tab],[data-command-id=navigate-current]").length'),2);
  await click(`[data-command-id="note-${noteUrl}"]`);
  await until(()=>script('return document.querySelector("[data-testid=page-note-editor]")?.value.startsWith("100%_")'),'saved note editor');
  assert.equal((await invoke('get_tabs')).tabs.length,before,'opening a note editor does not navigate');
  assert.equal(await script('return document.querySelector("[data-testid=note-pinned-url]").textContent'),noteUrl);
  // A draft for the selected URL survives page changes and another search selection.
  await script(`const editor=document.querySelector('[data-testid=page-note-editor]');editor.value='Unsaved draft for original saved URL';editor.dispatchEvent(new Event('input',{bubbles:true}));`);
  await invoke('switch_tab',{tabId:b.id});
  await until(()=>script('return document.querySelector("textarea").value==="Unsaved draft for original saved URL"'),'note remains bound to original URL');
  await click('[data-testid="note-follow-active"]');
  await until(()=>script('return !document.querySelector("[data-testid=note-pinned-url]")'),'follow active page');
  await click('[data-testid="close-tools"]');
  await query('cafe');
  await until(()=>script('return Boolean(document.querySelector("[data-command-id^=session-]"))'),'session result');
  await click(`[data-command-id="session-${session.id}"]`);
  await until(()=>script('return Boolean(document.querySelector("[data-testid=session-details]"))'),'session details');
  assert.equal((await invoke('get_tabs')).tabs.length,before,'Enter on a session only opens details');
  assert.ok(await script('return document.querySelector("[data-testid=session-details]").textContent.includes("Café")'));
  await invoke('rename_named_session',{id:session.id,name:'Café collection revised'});
  await click('[data-testid="session-details-restore"]');
  await until(()=>script('return document.querySelector("[data-testid=session-details]").textContent.includes("revised")'),'changed session refreshed');
  assert.equal((await invoke('get_tabs')).tabs.length,before,'changed details need another confirmation');
  await assert.rejects(()=>invoke('restore_named_session',{id:session.id,expectedSession:session}),/command_search.changed/);
  await click('[data-testid="session-details-restore"]');
  await until(async()=> (await invoke('get_tabs')).tabs.length===before+session.pages.length,'explicit restoration adds tabs');
  await click('[data-testid="close-tools"]');
  // Stale search items are revalidated before opening.
  await query('other cafe');
  await until(()=>script('return Boolean(document.querySelector("[data-command-id^=note-]"))'),'note to remove');
  await invoke('delete_page_note',{url:fixture.origin+'/page/other-note'});
  await click(`[data-command-id="note-${fixture.origin}/page/other-note"]`);
  assert.ok(await script('return Boolean(document.querySelector(".command-input"))'),'removed item keeps command bar open');
  await script(`document.querySelector('.command-input').value='';document.querySelector('.command-input').dispatchEvent(new Event('input',{bubbles:true}));`);
  await script(`document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));`);
  await invoke('switch_tab',{tabId:a.id});
  assert.ok(original.includes('/spa'));
}
