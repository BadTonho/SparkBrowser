import assert from 'node:assert/strict';

export async function checkToolsPopover(context) {
  const { invoke, fixture, until, click, script, withWebview, screenshot, resizeWindow, wait } = context;
  const inside = action => withWebview('/index.html?surface=tools', action);
  const nativeClick = selector => inside(() => click(selector));
  const shown = async () => {
    await until(() => script('return !!document.querySelector(".tools-controller")'), 'tools host opens');
    await until(() => inside(() => script('return !!document.querySelector("[data-testid=tools-dialog]")')), 'native tools surface');
  };
  const closed = async () => {
    await until(() => script('return !document.querySelector(".tools-controller")'), 'tools host closes');
    await until(() => inside(() => script('return !document.querySelector("[data-testid=tools-dialog]")')), 'native tools closes');
  };
  const escape = () => inside(() => script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))'));
  const open = async () => { await click('[data-action=tools]'); await shown(); };
  await invoke('navigate_to', { input: fixture.origin + '/page/tools-a' });
  const a = (await invoke('get_tabs')).tabs.find(tab => tab.is_active);
  const b = (await invoke('create_tab', { url: fixture.origin + '/page/tools-b' })).tabs.find(tab => tab.is_active);
  await until(() => fixture.reports.has('tools-a') && fixture.reports.has('tools-b'), 'page fixtures');
  await invoke('set_split_view', { leftTabId: a.id, rightTabId: b.id });
  await context.focusWindow();
  const noteUrl = (await invoke('get_tabs')).tabs.find(tab => tab.is_active).url;
  const fresh = Date.now();
  await until(() => ['tools-a', 'tools-b'].every(id => fixture.reports.get(id)?.reportedAt > fresh), 'fresh split reports');
  const hidden = ['tools-a', 'tools-b'].map(id => fixture.reports.get(id).hiddenCount);
  const loads = ['tools-a', 'tools-b'].map(id => fixture.loads.get(id));
  await open();
  const anchor = await script('const r=document.querySelector("[data-action=tools]").getBoundingClientRect();return {right:r.right,bottom:r.bottom};');
  const box = await inside(() => script('const r=document.querySelector("[data-testid=tools-dialog]").getBoundingClientRect();return {right:r.right,top:r.top,width:r.width,height:r.height};'));
  assert.equal(box.width, 640); assert.ok(Math.abs(box.top - anchor.bottom - 8) < 2); assert.ok(Math.abs(box.right - anchor.right) < 2);
  assert.ok(box.height <= 640);
  await inside(() => screenshot('tools-anchored-split'));

  await nativeClick('[data-testid=tools-notes]');
  await until(() => inside(() => script('return !!document.querySelector("[data-testid=page-note-editor]")')), 'native note editor');
  const text = 'Native floating tools note — preserved while the page stays visible';
  await inside(() => script('const e=document.querySelector("[data-testid=page-note-editor]");e.focus();e.value=arguments[0];e.setSelectionRange(9,9);e.dispatchEvent(new Event("input",{bubbles:true}));', text));
  await until(async () => (await invoke('get_page_note', { url: noteUrl }))?.text === text, 'host note writer saves native input');
  assert.equal(await inside(() => script('return document.querySelector("[data-testid=page-note-editor]").selectionStart')), 9, 'snapshot updates preserve editing caret');

  await nativeClick('[data-testid=tools-sessions]');
  await until(() => inside(() => script('return !!document.querySelector("form input")')), 'sessions form');
  await inside(() => script('const e=document.querySelector("form input");e.value="Floating session";e.dispatchEvent(new Event("input",{bubbles:true}));'));
  await until(() => inside(() => script('return !document.querySelector("form button").disabled')), 'save session enabled');
  await nativeClick('form button');
  await until(async () => (await invoke('get_saved_sessions')).some(s => s.name === 'Floating session'), 'native surface saves session');
  await nativeClick('[data-testid=tools-tabs]');
  await until(() => inside(() => script('return document.querySelectorAll(".tool-card").length>=2')), 'tab organizer rendered');
  await nativeClick('[data-testid=tools-closed]');
  await until(() => inside(() => script('return document.querySelector("[data-testid=tools-closed]").getAttribute("aria-selected")==="true"')), 'closed tabs section');
  await nativeClick('[data-testid=tools-diagnostics]');
  await until(() => inside(() => script('return !!document.querySelector("[data-testid=diagnostics-totals]")')), 'diagnostics loaded');
  await inside(() => screenshot('tools-diagnostics'));

  await inside(() => script('location.href="veloce-internal://tools-popover/action?token=invalid&sequence=999&payload="+encodeURIComponent(JSON.stringify({kind:"dismiss",reason:"outside"}));'));
  await wait(100);
  assert.equal(await script('return document.querySelector("[data-action=tools]").getAttribute("aria-expanded")'), 'true', 'forged action rejected');
  await inside(() => script('window.toolsForbidden=false;window.__TAURI_INTERNALS__.invoke("get_tabs").then(()=>window.toolsForbidden=false,()=>window.toolsForbidden=true);'));
  await until(() => inside(() => script('return window.toolsForbidden')), 'child cannot invoke application commands');
  await escape(); await closed();
  assert.equal(await script('return document.activeElement?.matches("[data-action=tools]")'), true, 'Escape restores trigger focus');

  await open();
  await nativeClick('[data-testid=tools-notes]');
  await until(() => inside(() => script('return document.querySelector("[data-testid=page-note-editor]")?.value===arguments[0]', text)), 'saved note restored');
  const lastEdit = text + ' — last keystroke before closing';
  await inside(() => script('const e=document.querySelector("[data-testid=page-note-editor]");e.value=arguments[0];e.dispatchEvent(new Event("input",{bubbles:true}));document.querySelector("[data-testid=close-tools]").click();', lastEdit));
  await closed();
  await until(async () => (await invoke('get_page_note', { url: noteUrl }))?.text === lastEdit, 'last edit persists when close is immediate');
  await open();
  await inside(() => script('document.querySelector(".tools-dismiss").click()'));
  await closed();

  const beforeCompact = Date.now();
  await until(() => ['tools-a', 'tools-b'].every(id => fixture.reports.get(id)?.reportedAt > beforeCompact), 'reports after tools');
  assert.deepEqual(['tools-a', 'tools-b'].map(id => fixture.reports.get(id).hiddenCount), hidden, 'tools never hide either page');
  await resizeWindow(900, 700);
  await context.focusWindow();
  await until(() => script('return !!document.querySelector("[data-testid=open-titlebar-menu]")'), 'compact toolbar');
  await click('[data-testid=open-titlebar-menu]');
  await until(() => script('return !!document.querySelector("[data-testid=titlebar-menu]")'), 'compact menu opens');
  await click('[data-testid=titlebar-menu] [data-testid=open-tools]');
  await shown();
  const compactAnchor = await script('return document.querySelector("[data-testid=open-titlebar-menu]").getBoundingClientRect().bottom');
  const compactTop = await inside(() => script('return document.querySelector("[data-testid=tools-dialog]").getBoundingClientRect().top'));
  assert.ok(Math.abs(compactTop - compactAnchor - 8) < 2);
  await inside(() => screenshot('tools-compact'));
  await escape(); await closed();
  await resizeWindow(1280, 820);
  const after = Date.now();
  await until(() => ['tools-a', 'tools-b'].every(id => fixture.reports.get(id)?.reportedAt > after), 'fresh preserved split reports');
  assert.deepEqual(['tools-a', 'tools-b'].map(id => fixture.loads.get(id)), loads, 'tools never reload either page');
  for (const id of ['tools-a', 'tools-b']) assert.equal(fixture.reports.get(id).value, 'artificial-test-draft');
}
