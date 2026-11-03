import assert from 'node:assert/strict';

export async function checkSplitTabHeader(context, originalLayout) {
  const { invoke, script, click, until, resizeWindow, fixture, wait, withWebview } = context;
  const key = name => script('(document.activeElement??document).dispatchEvent(new KeyboardEvent("keydown",{key:arguments[0],bubbles:true,cancelable:true}))', name);
  const closed = () => until(() => script('return !document.querySelector("[data-testid=titlebar-menu]")'), 'More menu dismissed');
  const usableAddress = () => until(() => script(`const address=document.querySelector('.omnibox-input').getBoundingClientRect(),controls=document.querySelector('.window-controls').getBoundingClientRect(),nav=document.querySelector('.nav-controls').getBoundingClientRect(),wrapper=document.querySelector('.omnibox-wrapper').getBoundingClientRect(),go=document.querySelector('.go-btn').getBoundingClientRect();
    return address.width>=64 && controls.right<=innerWidth && nav.right<=wrapper.left && wrapper.right<=document.querySelector('.titlebar-controls').getBoundingClientRect().left && go.right<=wrapper.right;`), 'address contents and fixed window controls fit without overlap');
  const open = async () => {
    await click('[data-testid=open-titlebar-menu]');
    await until(() => script('return document.querySelector("[data-testid=titlebar-menu]")?.contains(document.activeElement)'), 'More menu takes keyboard focus');
    assert.equal(await script('return Boolean(document.querySelector(".browser-preview-fallback"))'), false, 'More uses a neutral page reserve');
  };
  const visible = state => until(() => ['a','b'].every(id => fixture.reports.get('groups-'+id)?.visibility === state), 'both pages ' + state);

  await until(() => script(`const address=document.querySelector('.titlebar').getBoundingClientRect(),tabs=document.querySelector('.h-tabbar').getBoundingClientRect(),frame=document.querySelector('.viewport-frame').getBoundingClientRect();
    return address.top===0 && tabs.top>=address.bottom && tabs.bottom<=frame.top && document.querySelectorAll('.window-controls').length===1 && Boolean(document.querySelector('.titlebar .window-controls')) && window.fixtureTabBar===document.querySelector('.h-tabbar');`), 'split repositions the same tab bar below the address without an empty top row');

  fixture.commands.set('groups-a', 'spa');
  await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === originalLayout.left_tab_id).navigation.can_go_back, 'fixture history before header changes');
  const scroll = fixture.reports.get('groups-a').scroll;

  // Add a real bookmark through the editor so the shell and database agree.
  await click('.star-btn');
  await until(() => script('return Boolean(document.querySelector(".bookmark-dialog .primary-button"))'), 'bookmark editor');
  await click('.bookmark-dialog .primary-button');
  await until(() => script('return Boolean(document.querySelector(".bookmark-bar-transition"))&&!document.querySelector(".bookmark-dialog")'), 'bookmarks above pane tab lists');
  await until(() => script(`const a=document.querySelector('.titlebar').getBoundingClientRect(),b=document.querySelector('.bookmark-bar-transition').getBoundingClientRect(),t=document.querySelector('.h-tabbar').getBoundingClientRect();return a.bottom<=b.top && b.bottom<=t.top+1;`), 'address, bookmarks, pane lists order');
  await resizeWindow(680,780);
  await usableAddress();

  await script('const b=document.querySelector("[data-testid=open-titlebar-menu]");b.focus();b.dispatchEvent(new KeyboardEvent("keydown",{key:"ArrowDown",bubbles:true,cancelable:true}));');
  await until(() => script('return document.activeElement?.matches("[data-testid=titlebar-menu] [data-testid=open-tools]")'), 'ArrowDown opens More at first item');
  await visible('hidden');
  await key('End');
  assert.equal(await script('return document.activeElement?.dataset.testid'), 'toggle-zen');
  await key('Home'); await key('ArrowDown');
  assert.equal(await script('return document.activeElement?.dataset.testid'), 'toggle-bookmarks');
  await key('Escape'); await closed();
  await until(() => script('return document.activeElement?.matches("[data-testid=open-titlebar-menu]")'), 'Escape restores focus to More');
  await visible('visible');
  await open(); await click('.menu-dismiss'); await closed();
  await open(); await key('Tab'); await closed();

  for (const item of ['open-tools','open-command-bar','open-history','open-downloads','open-settings']) {
    await open(); await visible('hidden');
    await wait(250);
    const hiddenCount = fixture.reports.get('groups-a').hiddenCount;
    await click(`[data-testid=titlebar-menu] [data-testid=${item}]`);
    await closed();
    await until(() => script('return Boolean(document.querySelector("[role=dialog]"))'), item + ' opens from More');
    await wait(300);
    assert.equal(fixture.reports.get('groups-a').visibility, item === 'open-tools' ? 'visible' : 'hidden');
    assert.equal(fixture.reports.get('groups-a').hiddenCount, hiddenCount, 'handoff does not expose then hide the page again');
    await key('Escape');
    await until(() => script('return !document.querySelector("[role=dialog]")'), item + ' closes');
    await until(() => script('return document.activeElement?.matches("[data-testid=open-titlebar-menu]")'), 'dialog restores its persistent opener');
    await visible('visible');
  }

  await open(); await click('[data-testid=toggle-bookmarks]'); await closed();
  await until(() => script('return !document.querySelector(".bookmark-bar-transition")'), 'More hides bookmarks');
  await open(); await click('[data-testid=toggle-bookmarks]'); await closed();
  await until(() => script('return Boolean(document.querySelector(".bookmark-bar-transition"))'), 'More restores bookmarks');

  await open(); await click('[data-testid=titlebar-menu] [data-testid=open-privacy]'); await closed();
  await withWebview('/privacy-popover.html', async () => {
    await until(() => script('return !document.querySelector("#privacy-popover").hidden'), 'More opens the native privacy popover');
    await script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))');
  });
  await until(() => script('return document.activeElement?.matches("[data-testid=open-titlebar-menu]")'), 'native privacy returns focus to More');

  await open(); await resizeWindow(1001,780); await closed();
  await until(() => script('return !document.querySelector("[data-testid=open-titlebar-menu]") && Boolean(document.querySelector(".titlebar [data-testid=open-settings]"))'), 'wide header restores direct shortcuts');
  await usableAddress();
  await resizeWindow(1000,780);
  await open(); await context.screenshot('split-tab-more');
  await invoke('switch_workspace', {workspaceId:'ws-work'}); await closed();
  await invoke('switch_workspace', {workspaceId:'ws-personal'});
  await until(() => script('return Boolean(document.querySelector("[data-testid=open-titlebar-menu]"))'), 'return to split workspace');
  await open(); await click('[data-testid=toggle-zen]'); await closed();
  await until(() => script('return !document.querySelector(".titlebar")&&!document.querySelector(".h-tabbar")'), 'Zen hides chrome');
  await script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"z",altKey:true,bubbles:true,cancelable:true}))');
  await until(() => script('return Boolean(document.querySelector("[data-testid=split-tabs-left]"))'), 'leaving Zen restores pane tabs below address');
  await resizeWindow(1280,820);
  await visible('visible');
  assert.deepEqual((await invoke('get_tabs')).split_layout, originalLayout);
  for (const id of ['a','b']) assert.equal(fixture.loads.get('groups-'+id), 1, 'header operations preserve resident documents');
  assert.equal(fixture.reports.get('groups-a').value, 'artificial-test-draft');
  assert.equal(fixture.reports.get('groups-a').scroll, scroll);
  assert.equal((await invoke('get_tabs')).tabs.find(tab=>tab.id===originalLayout.left_tab_id).navigation.can_go_back, true);
}
