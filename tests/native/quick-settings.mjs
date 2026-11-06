import assert from 'node:assert/strict';

export async function checkQuickSettings(context) {
  const { invoke, fixture, until, script, click, withWebview, wait } = context;
  const inside = action => withWebview('/index.html?surface=settings-quick', action);
  await invoke('navigate_to', { input: fixture.origin + '/page/quick-settings' });
  await until(() => fixture.reports.has('quick-settings'), 'settings test page');
  await context.focusWindow();
  const initial = await script(`return {
    theme: document.documentElement.dataset.theme,
    bookmarks: localStorage.getItem('veloce_show_bookmarks_bar'),
    layout: localStorage.getItem('veloce_tab_layout'),
  };`);
  const loads = fixture.loads.get('quick-settings');
  await click('[data-testid="open-settings"]');
  await until(() => inside(() => script('return !!document.querySelector("[data-testid=settings-quick-menu]")')), 'native quick settings menu');
  assert.equal(await script('return document.querySelector(".browser-preview-fallback") === null'), true, 'quick settings keep the page visible');
  const anchor = await script('const r=document.querySelector("[data-testid=open-settings]").getBoundingClientRect();return {right:r.right,bottom:r.bottom};');
  const box = await inside(() => script('const r=document.querySelector("[data-testid=settings-quick-menu]").getBoundingClientRect();return {right:r.right,top:r.top,width:r.width};'));
  assert.equal(box.width, 300);
  assert.ok(Math.abs(box.top - anchor.bottom - 8) < 2, 'menu opens below settings icon');
  assert.ok(Math.abs(box.right - anchor.right) < 2, 'menu aligns to settings icon');

  const theme = initial.theme === 'light' ? 'dark' : 'light';
  await inside(() => click(`[data-testid="settings-quick-theme-${theme}"]`));
  await until(() => script(`return document.documentElement.dataset.theme === '${theme}'`), 'quick theme changes');
  await until(() => inside(() => script('return !!document.querySelector("[data-testid=settings-quick-menu]")')), 'menu stays open after theme change');
  await inside(() => click(`[data-testid="settings-quick-theme-${initial.theme || 'dark'}"]`));

  const bookmarksBefore = initial.bookmarks !== 'false';
  await inside(() => click('[data-testid="settings-quick-bookmarks"]'));
  await until(() => script(`return localStorage.getItem('veloce_show_bookmarks_bar') === '${!bookmarksBefore}'`), 'quick bookmarks toggle');
  await inside(() => click('[data-testid="settings-quick-bookmarks"]'));

  const originalLayout = initial.layout === 'vertical' ? 'vertical' : 'horizontal';
  const nextLayout = originalLayout === 'vertical' ? 'horizontal' : 'vertical';
  await inside(() => click(`[data-testid="settings-quick-layout-${nextLayout}"]`));
  await until(() => script(`return localStorage.getItem('veloce_tab_layout') === '${nextLayout}'`), 'quick tab layout changes');
  await until(() => inside(() => script('return !!document.querySelector("[data-testid=settings-quick-menu]")')), 'menu stays open after layout change');
  await inside(() => click(`[data-testid="settings-quick-layout-${originalLayout}"]`));

  await inside(() => script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))'));
  await until(() => inside(() => script('return !document.querySelector("[data-testid=settings-quick-menu]")')), 'Escape closes quick settings');
  await click('[data-testid="open-settings"]');
  await until(() => inside(() => script('return !!document.querySelector("[data-testid=settings-quick-menu]")')), 'quick settings reopens');
  await inside(() => click('[data-testid="settings-quick-all"]'));
  await until(() => script('return !!document.querySelector("#settings-search-input")'), 'all settings opens existing dialog');
  await script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))');
  await until(() => script('return !document.querySelector("#settings-search-input")'), 'Escape closes full settings');
  await until(() => script('return document.querySelector(".browser-preview-fallback") === null'), 'page preview returns after closing settings');

  await click('[data-testid="open-settings"]');
  await until(() => inside(() => script('return !!document.querySelector("[data-testid=settings-quick-menu]")')), 'menu opens for outside click');
  await inside(() => script('document.querySelector(".settings-quick-dismiss").click()'));
  await until(() => inside(() => script('return !document.querySelector("[data-testid=settings-quick-menu]")')), 'outside click closes quick settings');

  await context.resizeWindow(900, 700);
  await context.focusWindow();
  await click('[data-testid="open-settings"]');
  await until(() => inside(() => script('return !!document.querySelector("[data-testid=settings-quick-menu]")')), 'quick menu opens in narrow window');
  const narrow = await inside(() => script('const r=document.querySelector("[data-testid=settings-quick-menu]").getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom};'));
  assert.ok(narrow.left >= 0 && narrow.right <= 900 && narrow.bottom <= 700, 'menu stays within narrow window');
  await inside(() => script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))'));
  await until(() => inside(() => script('return !document.querySelector("[data-testid=settings-quick-menu]")')), 'narrow menu closes');
  await wait(100);
  assert.equal(fixture.loads.get('quick-settings'), loads, 'quick settings never reload the page');
}
