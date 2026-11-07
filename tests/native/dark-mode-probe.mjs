import assert from 'node:assert/strict';
import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

export async function checkDarkModeProbe(context) {
  const { invoke, until, fixture, script, screenshot, directory, wait } = context;
  const initialScheme = await invoke('set_dark_mode_probe_scheme', { mode: 'dark' });
  const results = [];
  try {
    const cases = [
      { name: 'native-supported-dark', kind: 'supported', transform: false, expectedDark: true },
      { name: 'native-light-only-dark', kind: 'light-only', transform: false, expectedDark: false },
      { name: 'native-already-dark', kind: 'already-dark', transform: false, expectedDark: true },
      { name: 'transform-light-only', kind: 'light-only', transform: true, expectedDark: false },
      { name: 'transform-already-dark', kind: 'already-dark', transform: true, expectedDark: true },
    ];

    for (const item of cases) {
      const url = new URL('/dark-mode-probe', fixture.origin);
      url.searchParams.set('kind', item.kind);
      if (item.transform) url.searchParams.set('transform', '1');
      await invoke('create_tab', { url: url.toString(), searchEngine: 'google' });
      await until(() => script(`return window.darkModeProbeReady===true && document.querySelector('#probe-image')?.complete && document.querySelector('#probe-image')?.naturalWidth===240 && document.querySelector('#probe-video')?.readyState>=2`), `dark mode probe ready: ${item.name}`);
      await until(() => script('return matchMedia("(prefers-color-scheme: dark)").matches'), 'native dark preference active');
      await wait(800);
      const metrics = await script(`
        const style = selector => { const node=document.querySelector(selector), css=getComputedStyle(node); return {background:css.backgroundColor,color:css.color,filter:css.filter}; };
        return {kind:document.documentElement.dataset.kind,dark:matchMedia('(prefers-color-scheme: dark)').matches,
          root:style('html'),body:style('body'),card:style('.card'),image:style('#probe-image'),video:style('#probe-video'),
          imageLoaded:document.querySelector('#probe-image').naturalWidth,videoReadyState:document.querySelector('#probe-video').readyState,
          dynamic:document.querySelector('#dynamic-content').textContent,documentHeight:document.documentElement.scrollHeight,scrollY};
      `);
      assert.equal(metrics.dark, true);
      assert.equal(metrics.body.background === 'rgb(23, 27, 34)' || metrics.body.background === 'rgb(17, 24, 32)', item.expectedDark,
        `Unexpected page background for ${item.name}: ${metrics.body.background}`);
      assert.ok(metrics.imageLoaded > 0 && metrics.videoReadyState >= 2, 'local image and generated video are visible to the renderer');
      if (item.transform) assert.match(metrics.root.filter, /invert\(1\)/, 'experimental root transform is applied');
      await screenshot(item.name);
      results.push({ name: item.name, ...metrics });
    }

    await invoke('set_dark_mode_probe_scheme', { mode: 'light' });
    for (const kind of ['supported', 'light-only']) {
      const url = new URL('/dark-mode-probe', fixture.origin);
      url.searchParams.set('kind', kind);
      await invoke('create_tab', { url: url.toString(), searchEngine: 'google' });
      await until(() => script('return window.darkModeProbeReady===true && document.querySelector("#probe-video")?.readyState>=2'), `light preference page ready: ${kind}`);
      await until(() => script('return !matchMedia("(prefers-color-scheme: dark)").matches'), 'native light preference active');
      const metrics = await script(`return {kind:document.documentElement.dataset.kind,dark:matchMedia('(prefers-color-scheme: dark)').matches,background:getComputedStyle(document.body).backgroundColor,scrollY}`);
      assert.equal(metrics.dark, false);
      await screenshot(`native-${kind}-light`);
      results.push({ name: `native-${kind}-light`, ...metrics });
    }
  } finally {
    await invoke('set_dark_mode_probe_scheme', { mode: initialScheme }).catch(() => {});
  }

  writeFileSync(resolve(directory, 'artifacts', 'dark-mode-probe.json'), JSON.stringify({
    engine: 'WebView2 PreferredColorScheme',
    initialScheme,
    cases: results,
    transformation: 'CSS invert(1) hue-rotate(180deg) on root, counter-filter on image/video',
  }, null, 2));
  console.log(JSON.stringify({ engine: 'WebView2 PreferredColorScheme', initialScheme, cases: results.length }));
}
