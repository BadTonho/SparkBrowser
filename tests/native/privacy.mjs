import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

export async function checkPrivacy(context) {
  console.log('Privacy: fixture setup');
  const { invoke, fixture, until, click, script, withWebview, restart, resizeWindow, wait, request, sessionId } = context;
  await invoke('navigate_to', { input: fixture.origin + '/page/privacy' });
  await until(() => fixture.reports.has('privacy'), 'privacy fixture');
  const a = (await invoke('get_tabs')).tabs.find(t => t.is_active);
  fixture.commands.set('privacy', 'spa');
  await until(async () => (await invoke('get_tabs')).tabs.find(t => t.id === a.id).navigation.can_go_back, 'privacy history');
  await until(() => script('return document.querySelector(".omnibox-input").value.endsWith("/page/privacy/spa")'), 'shell navigation settled');
  const hidden = fixture.reports.get('privacy').hiddenCount;
  const stats = await invoke('get_privacy_stats');
  assert.equal(stats.auto_skip_ads_enabled, false);
  const expanded = value => until(() => script('return document.querySelector("[data-testid=open-privacy]").getAttribute("aria-expanded")===arguments[0]', String(value)), 'privacy expanded ' + value);
  const open = async () => { await click('[data-testid=open-privacy]'); await expanded(true); };
  const inside = action => withWebview('/privacy-popover.html', async () => {
    await until(() => script('return !document.querySelector("#privacy-popover").hidden'), 'privacy native panel');
    return action();
  });
  await open();
  console.log('Privacy: popup opened');
  assert.equal((await invoke('get_privacy_stats')).shield_enabled, stats.shield_enabled, 'opening does not toggle');
  const anchor = await script('const r=document.querySelector("[data-testid=open-privacy]").getBoundingClientRect();return {right:r.right,bottom:r.bottom}');
  await inside(async () => {
    await context.screenshot('privacy-popover');
    const r = await script('const r=document.querySelector("#privacy-popover").getBoundingClientRect();return {width:r.width,right:r.right,top:r.top}');
    assert.equal(r.width, 300); assert.ok(Math.abs(r.right - anchor.right) < 2); assert.ok(Math.abs(r.top - anchor.bottom - 6) < 2);
    const checkbox = await request('POST', `/session/${sessionId()}/element`, { using: 'css selector', value: '#auto-skip' });
    await request('POST', `/session/${sessionId()}/element/${checkbox['element-6066-11e4-a52e-4f735466cecf']}/value`, { text: ' ', value: [' '] });
  });
  await until(async () => (await invoke('get_privacy_stats')).auto_skip_ads_enabled, 'auto skip enabled');
  console.log('Privacy: auto skip enabled');
  await inside(() => until(() => script('return document.querySelector("#auto-skip").checked && !document.querySelector("#auto-skip").disabled'), 'confirmed checkbox'));
  assert.equal((await invoke('get_privacy_stats')).shield_enabled, stats.shield_enabled, 'independent controls');
  await script(`window.fixtureStorageSet=Storage.prototype.setItem;Storage.prototype.setItem=function(key,value){if(key==='veloce_auto_skip_ads')throw Error('fixture storage failure');return window.fixtureStorageSet.call(this,key,value);}`);
  await inside(() => script('document.querySelector("#auto-skip").click()'));
  await inside(() => until(() => script('return !document.querySelector("#privacy-error").hidden && document.querySelector("#auto-skip").checked && !document.querySelector("#auto-skip").disabled'), 'storage failure restores checkbox'));
  assert.equal((await invoke('get_privacy_stats')).auto_skip_ads_enabled, true, 'storage failure rolls native policy back');
  await script('Storage.prototype.setItem=window.fixtureStorageSet;delete window.fixtureStorageSet;');
  await inside(() => script('location.href="veloce-internal://privacy-popover/auto-skip?token=invalid&sequence=999&value=false"'));
  await wait(100);
  assert.equal((await invoke('get_privacy_stats')).auto_skip_ads_enabled, true, 'unauthenticated popup action rejected');
  await inside(() => script('document.querySelector("#adblock").click()'));
  await until(async () => (await invoke('get_privacy_stats')).shield_enabled !== stats.shield_enabled, 'shield toggled');
  assert.equal((await invoke('get_privacy_stats')).auto_skip_ads_enabled, true);
  await inside(() => script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))'));
  await expanded(false);
  await until(() => script('return document.activeElement?.matches("[data-testid=open-privacy]")'), 'privacy returns focus');
  assert.equal(fixture.loads.get('privacy'), 1); assert.equal(fixture.reports.get('privacy').hiddenCount, hidden);
  assert.equal((await invoke('get_tabs')).tabs.find(t => t.id === a.id).navigation.can_go_back, true);
  assert.equal(fixture.reports.get('privacy').value, 'artificial-test-draft');
  await invoke('toggle_adblock_shield');

  for (const reason of ['outside', 'trigger']) {
    await open();
    await inside(() => script('const r=document.querySelector("#privacy-popover").getBoundingClientRect();document.dispatchEvent(new PointerEvent("pointerdown",{bubbles:true,clientX:arguments[0]?arguments[1]-10:2,clientY:arguments[0]?arguments[2]-10:300}));', reason === 'trigger', anchor.right, anchor.bottom));
    await expanded(false);
  }
  await open();
  await inside(() => script('document.querySelector("#auto-skip").focus();window.dispatchEvent(new KeyboardEvent("keydown",{key:"Tab",bubbles:true,cancelable:true}))'));
  await expanded(false);
  await open(); await script('document.querySelector("[data-testid=open-media]").click()'); await expanded(false);
  await script('document.querySelector("[data-testid=open-media]").click()');
  await open(); await script('document.querySelector("[data-testid=open-settings]").click()'); await expanded(false);
  await context.closeSettings();
  const b = (await invoke('create_tab', { url: fixture.origin + '/page/privacy-b' })).tabs.find(t => t.is_active);
  await until(() => fixture.reports.has('privacy-b'), 'privacy split fixture');
  await invoke('set_split_view', { leftTabId: a.id, rightTabId: b.id });
  for (const layout of [1, 0]) {
    await context.openSettings(); await click('[data-testid=settings-appearance_layout]');
    await script('document.querySelectorAll("#setting-browser-layout button")[arguments[0]].click()', layout);
    await context.closeSettings();
    await open();
    await resizeWindow(900, 700);
    if (layout === 0) {
      // The compact split header replaces the direct trigger with More.
      await until(()=>script('return !document.querySelector(".titlebar [data-testid=open-privacy]")&&Boolean(document.querySelector("[data-testid=open-titlebar-menu]"))'),'compact privacy entry');
      await withWebview('/privacy-popover.html',()=>until(()=>script('return document.querySelector("#privacy-popover").hidden'),'removed trigger dismisses privacy'));
      await click('[data-testid=open-titlebar-menu]');
      await until(()=>script('return Boolean(document.querySelector("[data-testid=titlebar-menu] [data-testid=open-privacy]"))'),'More privacy action');
      await click('[data-testid=titlebar-menu] [data-testid=open-privacy]');
    }
    await inside(() => until(() => script('const r=document.querySelector("#privacy-popover").getBoundingClientRect();return r.width===300 && r.left>=8 && r.right<=innerWidth-8'), 'privacy bounds'));
    await inside(() => script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))'));
    await withWebview('/privacy-popover.html',()=>until(()=>script('return document.querySelector("#privacy-popover").hidden'),'privacy dismissed after resize'));
    await resizeWindow(1280, 820);
    await expanded(false);
  }
  assert.equal(fixture.loads.get('privacy'), 1); assert.equal(fixture.loads.get('privacy-b'), 1);
  await open(); await invoke('switch_workspace', { workspaceId: 'ws-work' }); await expanded(false);
  await invoke('switch_workspace', { workspaceId: 'ws-personal' });

  // Exercise the unchanged production controller against a local real DOM with
  // a lexical URL/window facade. The production allowlist remains unchanged.
  const source = readFileSync('src-tauri/src/privacy/auto_skip.js', 'utf8').replace('__AUTO_SKIP_TOKEN__', JSON.stringify('fixture-auto-skip'));
  await withWebview('/page/privacy/spa', async () => {
    await script(`const player=document.createElement('div');player.id='movie_player';player.className='ad-showing';
      player.innerHTML='<button class="ytp-ad-skip-button">Skip</button>';document.body.prepend(player);
      window.fixtureClicks=0;player.querySelector('button').onclick=()=>window.fixtureClicks++;
      const facade={addEventListener:window.addEventListener.bind(window),removeEventListener:window.removeEventListener.bind(window)};facade.top=facade;
      new Function('location','window',arguments[0])(new URL('https://www.youtube.com/watch?v=fixture'),facade);
      window.fixtureAutoSkip=facade.__veloceSetAutoSkip;window.fixtureAutoSkip('fixture-auto-skip',true);`, source);
    await until(() => script('return window.fixtureClicks===1'), 'native DOM auto click');
    await wait(600); assert.equal(await script('return window.fixtureClicks'), 1);
    await script('window.fixtureAutoSkip("fixture-auto-skip",false);const b=document.querySelector("#movie_player button");const n=b.cloneNode(true);n.onclick=()=>window.fixtureClicks++;b.replaceWith(n);');
    await wait(600); assert.equal(await script('return window.fixtureClicks'), 1, 'disabled controller stays inert');
  });
  await invoke('switch_tab', { tabId: b.id });
  await invoke('set_split_view', { leftTabId: null, rightTabId: null });
  assert.equal((await invoke('get_tabs')).tabs.find(tab => tab.id === a.id).is_visible, false);
  await withWebview('/page/privacy/spa', () => script('window.fixtureAutoSkip("fixture-auto-skip",true)'));
  await withWebview('/page/privacy/spa', () => until(() => script('return window.fixtureClicks===2'), 'background page skips'));
  if (process.platform === 'win32') {
    await withWebview('/page/privacy/spa', () => script(`const c=document.createElement('canvas');c.width=320;c.height=180;const ctx=c.getContext('2d');
      window.fixtureVideoTimer=setInterval(()=>{ctx.fillStyle=Date.now()%2?'red':'blue';ctx.fillRect(0,0,320,180);},40);
      const v=document.createElement('video');v.muted=true;v.width=320;v.height=180;v.srcObject=c.captureStream(25);document.body.prepend(v);v.play().catch(()=>{});`));
    const candidate = await until(async () => {
      const state = (await invoke('refresh_tab_media', { tabId: a.id })).tabs.find(t => t.tab_id === a.id);
      for (const frame of state.frames) for (const video of frame.videos) if (video.eligible) return { frame, video };
      return false;
    }, 'background PiP candidate');
    await invoke('enter_video_pip', { tabId: a.id, context: candidate.frame.context, document: candidate.frame.document, video: candidate.video.id });
    await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === a.id).activity.picture_in_picture, 'PiP source active');
    await withWebview('/page/privacy/spa', () => script(`const b=document.querySelector('#movie_player button'),n=b.cloneNode(true);n.onclick=()=>window.fixtureClicks++;b.replaceWith(n);`));
    await withWebview('/page/privacy/spa', () => until(() => script('return window.fixtureClicks===3'), 'PiP source auto skip'));
    await invoke('exit_video_pip');
    await withWebview('/page/privacy/spa', () => script('clearInterval(window.fixtureVideoTimer);document.querySelector("video").srcObject.getTracks().forEach(track=>track.stop());document.querySelector("video").remove();'));
  }
  await restart();
  await until(async () => (await invoke('get_privacy_stats')).auto_skip_ads_enabled, 'auto skip persists after restart');
}
