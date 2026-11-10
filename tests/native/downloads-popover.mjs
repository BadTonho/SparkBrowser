import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { useAutomaticDownloads } from './download-preferences.mjs';

export async function checkDownloadsPopover(c) {
  await useAutomaticDownloads(c);
  const { invoke, script, click, until, withWebview, fixture, wait, resizeWindow } = c;
  const inside = action => withWebview('/index.html?surface=downloads', action);
  const shown = async () => {
    await until(() => script('return !!document.querySelector(".downloads-controller")'), 'downloads controller');
    await inside(() => until(() => script('return !!document.querySelector("[data-testid=downloads-dialog]")'), 'downloads surface'));
  };
  const closed = () => until(() => script('return !document.querySelector(".downloads-controller")'), 'downloads closed');
  const escape = async () => { await inside(() => script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))')); await closed(); };
  const open = async () => { await click('[data-action=downloads]'); await shown(); };
  const reveal = async () => {
    await inside(async () => { if (!await script('return !!document.querySelector("[data-testid=download-url]")')) await click('[data-testid=download-add-toggle]'); });
    await inside(() => until(() => script('return !!document.querySelector("[data-testid=download-url]")'), 'link field expanded'));
  };
  const ring = () => script('const e=document.querySelector("[data-testid=download-progress-ring]"),r=e?.querySelector("rect");return e?{kind:e.dataset.kind,color:getComputedStyle(e).color,offset:getComputedStyle(r).strokeDashoffset,animation:getComputedStyle(r).animationName}:null');
  const triggerClick = async () => {
    const point = await script('const r=document.querySelector("[data-action=downloads]").getBoundingClientRect();return {x:Math.round(r.x+r.width/2),y:Math.round(r.y+r.height/2)}');
    await inside(() => c.request('POST', `/session/${c.sessionId()}/actions`, { actions: [{ type: 'pointer', id: 'download-trigger', parameters: { pointerType: 'mouse' }, actions: [{ type: 'pointerMove', ...point, duration: 0 }, { type: 'pointerDown', button: 0 }, { type: 'pointerUp', button: 0 }] }] }));
    await closed();
  };
  const state = id => invoke('get_download_tasks').then(tasks => tasks.find(t => t.id === id));
  const a = (await invoke('create_tab', { url: fixture.origin + '/page/downloads-a' })).tabs.find(t => t.is_active);
  const b = (await invoke('create_tab', { url: fixture.origin + '/page/downloads-b' })).tabs.find(t => t.is_active);
  await until(() => fixture.reports.has('downloads-a') && fixture.reports.has('downloads-b'), 'download page fixtures');
  await invoke('set_split_view', { leftTabId: a.id, rightTabId: b.id });
  await c.focusWindow();
  const before = Date.now(); await until(() => ['downloads-a','downloads-b'].every(id => fixture.reports.get(id).reportedAt > before), 'visible split fixtures');
  const hidden = ['downloads-a','downloads-b'].map(id => fixture.reports.get(id).hiddenCount);
  const loads = ['downloads-a','downloads-b'].map(id => fixture.loads.get(id));
  await open();
  const anchor = await script('const r=document.querySelector("[data-action=downloads]").getBoundingClientRect();return {right:r.right,bottom:r.bottom}');
  const box = await inside(() => script('const r=document.querySelector("[data-testid=downloads-dialog]").getBoundingClientRect();return {right:r.right,top:r.top,width:r.width,height:r.height}'));
  assert.equal(box.width, 480); assert.ok(Math.abs(box.top - anchor.bottom - 8) < 2); assert.ok(Math.abs(box.right - anchor.right) < 2);
  assert.ok(box.height < 560, 'empty panel fits its content');
  assert.match(await inside(() => script('return getComputedStyle(document.querySelector("[data-testid=downloads-dialog]")).backgroundColor')), /^rgb\(\d+, \d+, \d+\)$/, 'card background is opaque in the current theme');
  assert.equal(await inside(() => script('return document.activeElement?.matches("[data-testid=download-add-toggle]")')), true);
  assert.equal(await inside(() => script('return !!document.querySelector("[data-testid=download-url]")')), false);
  await inside(() => c.screenshot('downloads-empty'));
  await inside(() => assert.rejects(() => invoke('get_tabs'), /restricted|denied|not allowed/i));
  await inside(() => script('location.href="https://external-navigation.invalid/"')); await wait(100);
  await reveal();
  await inside(() => until(() => script('return document.activeElement?.matches("[data-testid=download-url]")'), 'link field receives focus'));
  const hash = createHash('sha256').update(fixture.payload(1)).digest('hex');
  await inside(() => script(`const input=document.querySelector('[data-testid=download-url]');input.value=arguments[0];input.dispatchEvent(new Event('input',{bubbles:true}));document.querySelector('form').requestSubmit();`, fixture.origin + '/download/file?slow=1'));
  const first = await until(async () => (await invoke('get_download_tasks')).find(t => t.source === 'url'), 'native form enqueues');
  await inside(() => until(() => script('return !document.querySelector("[data-testid=download-url]") && document.activeElement?.matches("[data-testid=download-add-toggle]")'), 'successful download collapses form and restores focus'));
  await escape();
  await until(() => script('return document.querySelector("[data-action=downloads] [data-testid=download-progress-ring]")?.dataset.kind === "determinate"'), 'closed panel still updates progress');
  await open();
  await inside(() => until(() => script('return document.querySelector("[data-testid=download-task] .download-size")?.textContent.includes("/")'), 'native compact size updates'));
  await click(`[data-task-id="${first.id}"] [data-testid=download-pause]`);
  await until(async () => (await state(first.id)).status === 'Paused', 'native pause');
  await invoke('set_download_checksum', { taskId: first.id, checksum: hash });
  await until(() => script('return document.querySelector("[data-action=downloads] [data-testid=download-progress-ring]")?.dataset.kind === "paused"'), 'paused static border');
  assert.equal((await ring()).color, 'rgb(234, 179, 8)');
  assert.equal((await ring()).animation, 'none');
  await inside(() => c.screenshot('downloads-paused'));
  await escape();
  assert.equal(await script('return document.activeElement?.matches("[data-action=downloads]")'), true, 'Escape restores the trigger');

  const unknown = await invoke('enqueue_download', { url: fixture.origin + '/download/file?unknown=1&slow=1' });
  await until(async () => { const task = await state(unknown.id); return task.status === 'Downloading' && task.received > 0; }, 'unknown download receives bytes');
  assert.equal((await state(unknown.id)).total, null);
  await until(() => script('return document.querySelector("[data-action=downloads] [data-testid=download-progress-ring]")?.dataset.kind === "indeterminate"'), 'unknown length border');
  await invoke('control_download', { taskId: unknown.id, action: 'pause' });
  await until(async () => (await state(unknown.id)).status === 'Paused', 'unknown length pause');
  await until(async () => (await ring())?.kind === 'paused', 'paused indicator after unknown length transfer');
  const frozen = await ring(); await wait(150);
  assert.equal((await ring()).offset, frozen.offset, 'unknown-size paused border does not move');
  assert.equal((await ring()).color, 'rgb(234, 179, 8)');
  await invoke('control_download', { taskId: unknown.id, action: 'cancel' });
  await until(async () => (await state(unknown.id)).status === 'Cancelled', 'unknown download cancelled');
  const direct = Date.now(); await until(() => ['downloads-a','downloads-b'].every(id => fixture.reports.get(id).reportedAt > direct), 'reports after direct downloads');
  assert.deepEqual(['downloads-a','downloads-b'].map(id => fixture.reports.get(id).hiddenCount), hidden, 'direct downloads never hide either pane');

  // The progress SVG must not intercept the existing hold-and-drag gesture.
  for (const end of ['left', 'right']) {
    const point = await script('const b=document.querySelector("[data-action=downloads]"),r=b.getBoundingClientRect(),s=b.parentElement.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2,left:s.left+1,right:s.right-1}');
    await c.request('POST', `/session/${c.sessionId()}/actions`, { actions: [{ type: 'pointer', id: 'download-reorder', parameters: { pointerType: 'mouse' }, actions: [{ type: 'pointerMove', x: Math.round(point.x), y: Math.round(point.y), duration: 0 }, { type: 'pointerDown', button: 0 }, { type: 'pause', duration: 650 }, { type: 'pointerMove', x: Math.round(point[end]), y: Math.round(point.y), duration: 100 }, { type: 'pointerUp', button: 0 }] }] });
    assert.equal(await script('return !!document.querySelector(".downloads-controller")'), false, 'reorder does not open downloads');
    await open();
    const reference = await script('const r=document.querySelector("[data-action=downloads]").getBoundingClientRect();return {right:r.right,bottom:r.bottom,width:innerWidth}');
    const actual = await inside(() => script('const r=document.querySelector("[data-testid=downloads-dialog]").getBoundingClientRect();return {left:r.left,top:r.top}'));
    assert.ok(Math.abs(actual.left - Math.max(8, Math.min(reference.right - 480, reference.width - 488))) < 2);
    assert.ok(Math.abs(actual.top - reference.bottom - 8) < 2); await escape();
  }
  await c.openSettings(); await click('[data-testid=settings-appearance_layout]');
  await click('#setting-browser-layout .mini-pill:nth-child(2)'); await c.closeSettings();
  await open(); await resizeWindow(780, 620);
  await inside(() => until(() => script('const r=document.querySelector("[data-testid=downloads-dialog]").getBoundingClientRect();return r.left>=8 && r.right<=innerWidth-8 && r.bottom<=innerHeight-8'), 'vertical resize bounds'));
  await inside(() => c.screenshot('downloads-vertical')); await click('[data-testid=close-downloads]'); await closed();
  await c.openSettings(); await click('[data-testid=settings-appearance_layout]');
  await click('#setting-browser-layout .mini-pill:first-child'); await c.closeSettings();
  await resizeWindow(1280, 820);
  await click('[data-action=zen]');
  await until(() => script('return !document.querySelector(".titlebar")'), 'Zen before download shortcut');
  await c.request('POST', `/session/${c.sessionId()}/actions`, { actions: [{ type: 'key', id: 'downloads-shortcut', actions: [{ type: 'keyDown', value: '\uE009' }, { type: 'keyDown', value: 'j' }, { type: 'keyUp', value: 'j' }, { type: 'keyUp', value: '\uE009' }] }] });
  await shown(); assert.equal(await script('return !!document.querySelector(".titlebar")'), true, 'download shortcut restores the toolbar'); await escape();
  await resizeWindow(900, 700); await c.focusWindow();
  await until(() => script('return !!document.querySelector("[data-testid=open-titlebar-menu] [data-testid=download-progress-ring]")'), 'More shows progress');
  await click('[data-testid=open-titlebar-menu]'); await click('[data-testid=titlebar-menu] [data-testid=open-downloads]'); await shown();
  const compact = await script('return document.querySelector("[data-testid=open-titlebar-menu]").getBoundingClientRect().bottom');
  assert.ok(Math.abs((await inside(() => script('return document.querySelector("[data-testid=downloads-dialog]").getBoundingClientRect().top'))) - compact - 8) < 2);
  await escape(); await resizeWindow(1280, 820);
  await open();
  const clicks = ['downloads-a','downloads-b'].map(id => fixture.reports.get(id).pageClicks);
  await inside(async () => {
    const height = await script('return innerHeight');
    await c.request('POST', `/session/${c.sessionId()}/actions`, { actions: [{ type: 'pointer', id: 'downloads-outside', parameters: { pointerType: 'mouse' }, actions: [{ type: 'pointerMove', x: 4, y: height - 4, duration: 0 }, { type: 'pointerDown', button: 0 }, { type: 'pointerUp', button: 0 }] }] });
  });
  await closed();
  const outside = Date.now(); await until(() => ['downloads-a','downloads-b'].every(id => fixture.reports.get(id).reportedAt > outside), 'outside click consumed');
  assert.deepEqual(['downloads-a','downloads-b'].map(id => fixture.reports.get(id).pageClicks), clicks);

  // A duplicated field is rejected before dispatch, then the valid sequence can act once.
  await open(); await reveal();
  await inside(() => script(`window.originalDownloadUrlString=URL.prototype.toString;URL.prototype.toString=function(){window.savedDownloadAction=window.originalDownloadUrlString.call(this);return window.savedDownloadAction+'&token=duplicate'};const input=document.querySelector('[data-testid=download-url]');input.value='https://fixture.example/original';input.dispatchEvent(new Event('input',{bubbles:true}));`));
  await wait(100);
  assert.notEqual(await script('return document.querySelector("[data-testid=download-url]").value'), 'https://fixture.example/original');
  await inside(() => script('URL.prototype.toString=window.originalDownloadUrlString;location.href=window.savedDownloadAction'));
  await until(() => script('return document.querySelector("[data-testid=download-url]").value === "https://fixture.example/original"'), 'valid typed input');
  await inside(() => script(`const u=new URL(window.savedDownloadAction),p=JSON.parse(u.searchParams.get('payload'));p.value='https://fixture.example/replayed';u.searchParams.set('payload',JSON.stringify(p));location.href=u.toString();`));
  await wait(100);
  assert.equal(await script('return document.querySelector("[data-testid=download-url]").value'), 'https://fixture.example/original', 'replayed sequence is inert');
  await escape(); await open(); await reveal();
  await inside(() => script(`const u=new URL(window.savedDownloadAction),p=JSON.parse(u.searchParams.get('payload'));p.value='https://fixture.example/stale';u.searchParams.set('sequence','999');u.searchParams.set('payload',JSON.stringify(p));location.href=u.toString();`));
  await wait(100);
  assert.equal(await script('return document.querySelector("[data-testid=download-url]").value'), 'https://fixture.example/original', 'closed session token is inert');
  await inside(() => script('location.href="veloce-internal://downloads-popover/action?token=invalid&sequence=999&payload="+encodeURIComponent(JSON.stringify({kind:"invoke",command:"close_tab"}))'));
  await wait(100); assert.equal(await script('return !!document.querySelector(".downloads-controller")'), true);
  await escape();

  // Opening another panel dismisses downloads, and resizing preserves the pages.
  await open(); await script('document.querySelector("[data-action=tools]").click()'); await closed();
  await withWebview('/index.html?surface=tools', () => until(() => script('return !!document.querySelector("[data-testid=tools-dialog]")'), 'tools after downloads'));
  await withWebview('/index.html?surface=tools', () => script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))'));
  await open(); await invoke('plugin:window|minimize', { label: 'main' }); await closed(); await c.focusWindow();
  const after = Date.now(); await until(() => ['downloads-a','downloads-b'].every(id => fixture.reports.get(id).reportedAt > after), 'split reports after floating panels');
  assert.deepEqual(['downloads-a','downloads-b'].map(id => fixture.loads.get(id)), loads);
  for(const id of ['downloads-a','downloads-b']) { assert.equal(fixture.reports.get(id).value, 'artificial-test-draft'); assert.ok(fixture.reports.get(id).scroll >= 500, 'panel preserves page scroll'); }
  // More itself uses its existing modal handoff; direct Downloads never hid a page.
  assert.ok(['downloads-a','downloads-b'].every((id, index) => fixture.reports.get(id).hiddenCount >= hidden[index]));
  await invoke('control_download', { taskId: first.id, action: 'resume' });
  await until(async () => (await state(first.id)).status === 'Completed', 'resume completes and verifies');
  assert.equal((await state(first.id)).actual_sha256, hash);
  await until(async () => (await ring())?.kind === 'completed', 'completion stays green until acknowledged');
  assert.equal((await ring()).color, 'rgb(34, 197, 94)');
  await until(async () => (await ring())?.offset === '0px', 'completion transition fills the border');
  await c.restart(); await c.focusWindow();
  await until(async () => (await ring())?.kind === 'completed', 'green notice survives restart');
  await open(); assert.equal(await ring(), null, 'opening acknowledges completion');

  await invoke('set_download_checksum', { taskId: first.id, checksum: '0'.repeat(64) });
  await invoke('control_download', { taskId: first.id, action: 'verify' });
  await until(async () => (await state(first.id)).status === 'VerificationFailed', 'integrity error while the panel is open');
  await until(async () => (await ring())?.kind === 'error', 'new result creates a red notice');
  assert.equal((await ring()).color, 'rgb(239, 68, 68)');
  await click('[data-testid=close-downloads]'); await closed();
  assert.equal((await ring()).kind, 'error', 'header dismissal does not acknowledge a new result');
  await c.restart(); await c.focusWindow();
  await until(async () => (await ring())?.kind === 'error', 'red notice survives restart');
  const active = await invoke('enqueue_download', { url: fixture.origin + '/download/file?slow=1' });
  await until(async () => (await state(active.id)).status === 'Downloading', 'another transfer starts');
  assert.equal((await ring()).kind, 'error', 'red takes priority over an active transfer');
  await open();
  await until(async () => (await ring())?.kind === 'determinate', 'acknowledgement restores transfer progress');
  await invoke('control_download', { taskId: active.id, action: 'cancel' });
  await until(async () => (await state(active.id)).status === 'Cancelled', 'cancel additional transfer');
  await invoke('set_download_checksum', { taskId: first.id, checksum: hash });
  await invoke('control_download', { taskId: first.id, action: 'verify' });
  await until(async () => (await ring())?.kind === 'completed', 'new successful verification creates green');
  await escape(); assert.equal((await ring()).kind, 'completed', 'Escape preserves a new notice');
  await resizeWindow(900, 700);
  await until(() => script('return !!document.querySelector("[data-testid=open-titlebar-menu] .completed")'), 'More carries completion notice');
  await click('[data-testid=open-titlebar-menu]');
  assert.equal((await ring()).kind, 'completed', 'opening More alone does not acknowledge');
  await click('[data-testid=titlebar-menu] [data-testid=open-downloads]'); await shown();
  assert.equal(await ring(), null, 'Downloads menu entry acknowledges');
  await escape(); await resizeWindow(1280, 820);
  await open();
  await invoke('set_download_checksum', { taskId: first.id, checksum: '0'.repeat(64) });
  await invoke('control_download', { taskId: first.id, action: 'verify' });
  await until(async () => (await ring())?.kind === 'error', 'error before native trigger dismissal');
  await triggerClick();
  assert.equal(await ring(), null, 'native intercepted trigger click acknowledges and closes');
  await c.restart(); await c.focusWindow();
  assert.equal(await ring(), null, 'acknowledged errors do not return after restart');
}
