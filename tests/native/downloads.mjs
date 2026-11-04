import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, existsSync, writeFileSync, unlinkSync } from 'node:fs';
import { resolve, relative, isAbsolute, toNamespacedPath } from 'node:path';
export async function checkDownloads({ invoke, until, fixture, restart, directory, script, focusWindow }) {
  await focusWindow();
  const list = () => invoke('get_download_tasks');
  const task = id => list().then(rows => rows.find(row => row.id === id));
  const act = (id, action) => invoke('control_download', { taskId: id, action });
  const hash = version => createHash('sha256').update(fixture.payload(version)).digest('hex');
  const waitState = (id, status) => until(async () => (await task(id))?.status === status, `download ${status}`);
  const create = expectedSha256 => invoke('enqueue_download', { url: fixture.origin + '/download/file', expectedSha256: expectedSha256 ?? null });
  const pause = async id => { await until(async () => { const row = await task(id); if (['Interrupted','Completed','RestartRequired'].includes(row.status)) throw new Error(JSON.stringify(row)); return row.received > 32768; }, 'download progresses'); await act(id, 'pause'); await waitState(id, 'Paused'); };
  const safeRead = path => { const child = relative(toNamespacedPath(resolve(directory, 'data', 'downloads')), toNamespacedPath(resolve(path))); assert.ok(child && !child.startsWith('..') && !isAbsolute(child)); return readFileSync(path); };
  await script(`document.querySelector('[data-testid="open-downloads"]').click();`);
  await until(() => script(`return Boolean(document.querySelector('[data-testid="download-add-toggle"]'))`), 'download link control');
  await script(`document.querySelector('[data-testid="download-add-toggle"]').click();`);
  await until(() => script(`return Boolean(document.querySelector('[data-testid="download-url"]'))`), 'download controls');
  await script(`const input=document.querySelector('[data-testid=download-url]'); input.value=arguments[0]; input.dispatchEvent(new Event('input',{bubbles:true})); document.querySelector('[data-testid="download-start"]').click();`, fixture.origin + '/download/file?slow=1');
  const first = await until(async () => (await list()).find(t => t.source === 'url'), 'UI enqueues download');
  await until(async () => (await task(first.id)).received > 32768, 'UI download progresses');
  await until(() => script('return !!document.querySelector(`[data-task-id="${arguments[0]}"] [data-testid=download-pause]`)', first.id), 'pause control rendered');
  await script(`document.querySelector('[data-task-id="'+arguments[0]+'"] [data-testid="download-pause"]').click();`,first.id);
  await waitState(first.id,'Paused');
  await invoke('set_download_checksum', { taskId: first.id, checksum: hash(1) });
  const paused = await task(first.id); assert.ok(paused.can_resume); const requests = fixture.download.requests.length;
  await restart(); assert.equal(fixture.download.requests.length, requests, 'restart must not start transfers');
  await focusWindow();
  assert.equal((await task(first.id)).received, paused.received);
  await until(() => script('return document.querySelector("[data-action=downloads] [data-testid=download-progress-ring]")?.dataset.kind === "paused"'), 'paused border restored before opening downloads');
  await script(`document.querySelector('[data-testid="open-downloads"]').click();`);
  await until(() => script(`return Boolean(document.querySelector('[data-task-id="'+arguments[0]+'"] [data-testid="download-resume"]'))`,first.id), 'persistent resume button');
  await script(`document.querySelector('[data-task-id="'+arguments[0]+'"] [data-testid="download-resume"]').click();`,first.id);
  await waitState(first.id, 'Completed');
  await script(`document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));`);
  assert.equal((await task(first.id)).actual_sha256, hash(1)); assert.deepEqual(safeRead((await task(first.id)).save_path), fixture.payload(1));
  assert.ok(fixture.download.requests.some(request => request.range && request.ifRange === '"fixture-1"'));
  await invoke('set_download_checksum', { taskId: first.id, checksum: '0'.repeat(64) }); await act(first.id, 'verify'); await waitState(first.id, 'VerificationFailed');
  assert.ok(existsSync((await task(first.id)).save_path));
  await invoke('set_download_checksum', { taskId: first.id, checksum: hash(1) }); await act(first.id, 'verify'); await waitState(first.id, 'Completed');
  const second = await create(); await pause(second.id); fixture.download.version = 2;
  await act(second.id, 'resume'); await waitState(second.id, 'RestartRequired');
  await act(second.id, 'restart'); await waitState(second.id, 'Completed'); assert.deepEqual(safeRead((await task(second.id)).save_path), fixture.payload(2));
  for (const mode of ['ignore-range', 'bad-range', '416']) {
    fixture.download.mode = 'normal'; const current = await create(); await pause(current.id); fixture.download.mode = mode;
    await act(current.id, 'resume'); await waitState(current.id, 'RestartRequired'); await act(current.id, 'cancel');
  }
  fixture.download.mode = 'normal';
  for (const damage of ['changed', 'missing']) {
    const current = await create(); await pause(current.id);
    const partial = resolve(directory, 'data', 'downloads', '.veloce-partials', current.id + '.part');
    if (damage === 'missing') unlinkSync(partial); else { const bytes = readFileSync(partial); bytes[0] ^= 1; writeFileSync(partial, bytes); }
    const previousRequests = fixture.download.requests.length;
    await act(current.id, 'resume'); await waitState(current.id, 'RestartRequired');
    assert.equal(fixture.download.requests.length, previousRequests, 'invalid local partials must fail before contacting the server');
    await act(current.id, 'cancel');
  }
  fixture.download.mode = 'weak'; const weak = await create(); await pause(weak.id); assert.equal((await task(weak.id)).can_resume, false); await act(weak.id, 'cancel'); fixture.download.mode = 'normal';
  if (process.platform === 'win32') {
    await invoke('create_tab', { url: fixture.origin + '/page/b' });
    await until(() => fixture.reports.has('b'), 'native download page'); fixture.commands.set('b', 'download');
    const native = await until(async () => (await list()).find(t => t.source === 'native'), 'engine download registered');
    await pause(native.id); assert.equal((await task(native.id)).can_resume, true);
    const page = (await invoke('get_tabs')).tabs.find(t => t.is_active);
    await invoke('create_tab', { url: fixture.origin + '/page/p' });
    await assert.rejects(invoke('hibernate_tab', { tabId: page.id }), /protected/);
    await act(native.id, 'resume'); await waitState(native.id, 'Completed'); assert.deepEqual(safeRead((await task(native.id)).save_path), fixture.payload(2));
  }
  const history = await invoke('get_downloads'); assert.equal(new Set(history.map(row => row.save_path)).size, history.length);
  const completedPath = (await task(first.id)).save_path;
  const kept = await create(); await pause(kept.id);
  await invoke('clear_all_downloads'); assert.ok(existsSync(completedPath)); assert.ok(await task(kept.id)); assert.equal(await task(first.id), undefined);
}
