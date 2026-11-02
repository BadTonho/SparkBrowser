import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { performance } from 'node:perf_hooks';

function memory() {
  if (process.platform !== 'win32') return Promise.resolve(null);
  const build = JSON.parse(readFileSync('.cache/native-target/test-build.json', 'utf8'));
  return new Promise((done, reject) => {
    const child = spawn('powershell.exe', ['-NoProfile', '-File', resolve('tools/testing/memory-sample.ps1'), '-Executable', build.executable], { windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
    let output = '', error = '';
    child.stdout.on('data', value => output += value); child.stderr.on('data', value => error += value);
    child.on('error', reject);
    child.on('close', code => { if (code) reject(new Error(error)); else { try { done(JSON.parse(output)); } catch (error) { reject(error); } } });
  });
}

export async function checkTabChurn(context, { minutes = 0 } = {}) {
  const { invoke, until, fixture, wait, directory } = context;
  const output = resolve(directory, 'artifacts/tab-churn.json');
  const report = { minutes, cycles: 0, samples: [] };
  const persist = () => writeFileSync(output, JSON.stringify(report, null, 2));
  await invoke('navigate_to', { input: fixture.origin + '/page/churn-base' });
  await until(() => fixture.reports.has('churn-base'), 'initial churn fixture');
  for (let i = 1; i < 10; i++) {
    const name = `churn-base-${i}`;
    await invoke('create_tab', { url: fixture.origin + '/page/' + name });
    await until(() => fixture.reports.has(name), 'retained churn fixture');
  }
  await invoke('set_resident_limit', { limit: 8 }); await wait(5000);
  const started = performance.now();
  async function sample(phase) {
    const diagnostics = await invoke('get_diagnostics');
    assert.equal(diagnostics.tabs, 10);
    assert.ok(diagnostics.resident_webviews <= 8);
    const resources = diagnostics.native_resources;
    assert.ok(resources.bridge_tokens <= diagnostics.resident_webviews);
    assert.ok(resources.shield_scripts <= diagnostics.resident_webviews);
    assert.ok(resources.media_entries <= diagnostics.tabs);
    if (process.platform === 'win32') {
      assert.notEqual(resources.media_context_views, null);
      assert.ok(resources.media_context_views <= diagnostics.resident_webviews);
      assert.ok(resources.execution_contexts <= diagnostics.resident_webviews * 4, 'simple local pages retain bounded contexts');
    }
    const measurement = { phase, elapsedMs: performance.now() - started, diagnostics, memory: await memory() };
    report.samples.push(measurement); persist();
    console.log('Tab churn', JSON.stringify({ phase, cycles: report.cycles, resources, workingSetMiB: measurement.memory?.workingSetBytes / 1048576, privateMiB: measurement.memory?.privateBytes / 1048576 }));
  }
  await sample('start');
  for (let round = 1; round <= (minutes || 2); round++) {
    for (let i = 0; i < 10; i++) {
      const name = `churn-${round}-${i}`;
      const tab = (await invoke('create_tab', { url: fixture.origin + '/page/' + name })).tabs.find(t => t.is_active);
      await until(() => fixture.reports.has(name), 'temporary churn fixture');
      await invoke('close_tab', { tabId: tab.id }); report.cycles++;
    }
    await invoke('set_resident_limit', { limit: 8 });
    if (minutes) while (performance.now() - started < round * 60_000) await wait(Math.min(30_000, round * 60_000 - (performance.now() - started)));
    else await wait(1000);
    await sample(round);
  }
  await invoke('hibernate_inactive_tabs', { minIdleSecs: 0 }); await wait(10_000);
  await sample('hibernated');
  assert.equal(report.samples.at(-1).diagnostics.resident_webviews, 1);
  report.completed = true; persist();
}
