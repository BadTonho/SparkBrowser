import assert from 'node:assert/strict';

/** Light pages are darkened only while the setting is on; pages that are already dark are never touched. */
export async function checkForcedDark(context) {
  const { invoke, until, fixture, withWebview, script } = context;
  const open = async kind => {
    const url = new URL('/dark-mode-probe', fixture.origin);
    url.searchParams.set('kind', kind);
    await invoke('create_tab', { url: url.toString(), searchEngine: 'google' });
    const path = url.pathname + url.search;
    await until(() => withWebview(path, () => script('return window.darkModeProbeReady===true')), `${kind} page ready`);
    return (code) => withWebview(path, () => script(code));
  };
  const state = page => page(`const root=document.documentElement;return {dark:root.classList.contains('__spark-forced-dark'),filter:getComputedStyle(root).filter};`);

  await invoke('set_forced_dark_mode', { enabled: false });
  const light = await open('light-only');
  assert.equal((await state(light)).dark, false, 'nothing changes while the setting is off');

  await invoke('set_forced_dark_mode', { enabled: true });
  await until(async () => (await state(light)).dark, 'light page is darkened once the setting is on');
  assert.match((await state(light)).filter, /invert/);

  // A site that switches itself to a dark theme must lose the filter, or it would be flipped back to light.
  await light(`document.body.style.background='#111820';document.body.style.color='#f2f4f8';`);
  await until(async () => !(await state(light)).dark, 'filter is removed when the page turns dark');

  await invoke('set_forced_dark_mode', { enabled: false });
  const dark = await open('already-dark');
  await invoke('set_forced_dark_mode', { enabled: true });
  await new Promise(resolve => setTimeout(resolve, 600));
  assert.equal((await state(dark)).dark, false, 'an already dark page is left alone');
  await invoke('set_forced_dark_mode', { enabled: false });
}
