import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

export async function checkFullPageCaptureProbe(context) {
  const { invoke, until, fixture, directory, screenshot, withWebview } = context;
  const script = (...args) => withWebview('/capture-probe', () => context.script(...args));
  const packet = await invoke('create_tab', { url: fixture.origin + '/capture-probe', searchEngine: 'google' });
  const tab = packet.tabs.find(item => item.is_active);
  assert.ok(tab, 'capture fixture tab is active');
  await until(() => script('return window.captureProbeReady===true && document.querySelector("#bottom-marker")!==null'), 'capture fixture loaded');
  await until(() => script('return scrollY>=600'), 'fixture scroll position established');
  await until(() => script('const i=document.querySelector("#lazy-image");return i.complete&&i.naturalWidth===120'), 'lazy image loaded after scrolling');
  const before = await script('return {scrollY,innerWidth,innerHeight,documentHeight:document.documentElement.scrollHeight,devicePixelRatio}');
  assert.ok(before.documentHeight > before.innerHeight * 5, 'fixture is longer than the viewport');

  const result = await invoke('full_page_capture_probe');
  const after = await script('return {scrollY,innerWidth,innerHeight,documentHeight:document.documentElement.scrollHeight}');
  assert.equal(after.scrollY, before.scrollY, 'capture preserves the page scroll position');
  assert.equal(after.documentHeight, before.documentHeight, 'capture does not change document height');
  assert.ok(result.height > before.innerHeight * 3, `capture should extend beyond the viewport; got ${result.width}x${result.height}`);
  assert.ok(result.width >= before.innerWidth * 0.9, 'captured width covers the page viewport');
  assert.ok(result.png_bytes > 1000, 'capture produces a nontrivial PNG');

  const png = readFileSync(resolve(directory, 'artifacts', 'full-page-probe.png'));
  assert.equal(png.subarray(0, 8).toString('hex'), '89504e470d0a1a0a', 'artifact is a PNG');
  assert.equal(png.readUInt32BE(16), result.width, 'PNG width matches native metadata');
  assert.equal(png.readUInt32BE(20), result.height, 'PNG height matches native metadata');
  await screenshot('capture-probe-viewport');
  console.log(JSON.stringify({ ...result, viewport: before, scrollRestored: true }));
}
