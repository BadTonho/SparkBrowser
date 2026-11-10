import assert from 'node:assert/strict';
import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

export async function checkMinimumFontProbe(context) {
  const { invoke, until, fixture, screenshot, directory, withWebview } = context;
  const script = (...args) => withWebview('/minimum-font-probe', () => context.script(...args));
  const url = new URL('/minimum-font-probe', fixture.origin).toString();
  await invoke('create_tab', { url, searchEngine: 'google' });
  await until(() => script('return window.minimumFontProbeReady && document.querySelector("iframe")?.contentDocument?.readyState === "complete"'), 'font samples and iframe ready');

  const snapshots = [];
  for (const factor of [1, 1.5]) {
    await invoke('set_site_zoom', { origin: fixture.origin, factor });
    const metrics = await script(`
      const selectors=['#px8','#px12','#px16','#px24','#relative','a','button','input'];
      const sizes=Object.fromEntries(selectors.map(selector=>[selector,getComputedStyle(document.querySelector(selector)).fontSize]));
      const frame=document.querySelector('iframe');
      return {zoom:devicePixelRatio,viewport:{width:innerWidth,height:innerHeight},background:getComputedStyle(document.body).backgroundColor,
        sizes,iframeFont:getComputedStyle(frame.contentDocument.querySelector('p')).fontSize,iframeWidth:frame.getBoundingClientRect().width};
    `);
    assert.equal(metrics.sizes['#px8'], '8px');
    assert.equal(metrics.sizes['#px12'], '12px');
    assert.equal(metrics.sizes['#px16'], '16px');
    assert.equal(metrics.sizes['#px24'], '24px');
    assert.equal(metrics.iframeFont, '10px');
    snapshots.push({ factor, ...metrics });
    await screenshot(`minimum-font-zoom-${factor === 1 ? '100' : '150'}`);
  }
  assert.ok(snapshots[1].zoom > snapshots[0].zoom, 'page zoom scales the whole WebView, not a minimum CSS font size');

  let nativeMinimumFont = null;
  if (process.platform === 'linux') {
    const originalSize = await invoke('set_minimum_font_probe_size', { size: 0 });
    try {
      const baseline = await script(`return Object.fromEntries(['#px8','#px12','#px16','#px24','#relative','a','button','input'].map(selector=>[selector,getComputedStyle(document.querySelector(selector)).fontSize]))`);
      await screenshot('minimum-font-native-0px');
      await invoke('set_minimum_font_probe_size', { size: 12 });
      const minimumMetrics = await script(`
        const selectors=['#px8','#px12','#px16','#px24','#relative','a','button','input'];
        const sizes=Object.fromEntries(selectors.map(selector=>[selector,getComputedStyle(document.querySelector(selector)).fontSize]));
        return {sizes,iframeFont:getComputedStyle(document.querySelector('iframe').contentDocument.querySelector('p')).fontSize};
      `);
      await screenshot('minimum-font-native-12px');
      nativeMinimumFont = { engine: 'WebKitGTK', originalSize, testedSize: 12, baseline, minimumMetrics };
    } finally {
      await invoke('set_minimum_font_probe_size', { size: originalSize });
    }
  }
  writeFileSync(resolve(directory, 'artifacts', 'minimum-font-probe.json'), JSON.stringify({
    engine: process.platform === 'win32' ? 'WebView2' : 'WebKitGTK',
    nativeMinimumFontSettingFound: process.platform === 'linux',
    note: 'The probe compares page zoom at 100% and 150%; screenshots show the visual effect on all text sizes and controls. Linux also compares the native WebKitGTK minimum-font setting at 0px and 12px.',
    snapshots,
    nativeMinimumFont,
  }, null, 2));
  console.log(JSON.stringify({ engine: process.platform === 'win32' ? 'WebView2' : 'WebKitGTK', zoomLevels: snapshots.map(sample => sample.factor), nativeMinimumFont: Boolean(nativeMinimumFont) }));
}
