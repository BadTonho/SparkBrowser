import assert from 'node:assert/strict';
import { writeFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { useAutomaticDownloads } from './download-preferences.mjs';

export async function checkWebviewRuntime(context) {
  await useAutomaticDownloads(context);
  const { invoke, script, click, until, fixture, directory, restart, request, sessionId } = context;
  await context.focusWindow();
  const fixturePath = join(directory, 'config', 'runtime-fixture.json');
  const configure = (availableVersion, restartOutcome = 'fail', currentVersion = '200.0.0.0') =>
    writeFileSync(fixturePath, JSON.stringify({ currentVersion, availableVersion, restartOutcome }));
  configure('200.0.0.0');
  await until(async () => (await invoke('get_webview_runtime_status')).currentVersion === '200.0.0.0', 'runtime fixture applied');
  assert.equal(await script('return !!document.querySelector("[data-testid=runtime-notice]")'), false);
  await assert.rejects(() => invoke('restart_webview_runtime'), /no_update/);
  configure('201.0.0.0');
  await until(() => script('return !!document.querySelector("[data-testid=runtime-notice]")'), 'runtime notice');
  await click('[data-testid=runtime-later]');
  assert.equal((await invoke('get_webview_runtime_status')).restartRequired, true);
  await context.openSettings(); await click('[data-testid=settings-updates]');
  await until(() => script('return !!document.querySelector("[data-testid=runtime-settings-restart]")'), 'pending runtime visible in settings');
  await context.closeSettings();
  configure('202.0.0.0');
  await until(() => script('return !!document.querySelector("[data-testid=runtime-notice]")'), 'new version is not dismissed');

  await invoke('navigate_to', { input: fixture.origin + '/page/runtime' });
  await until(() => fixture.reports.has('runtime'), 'runtime page');
  fixture.commands.set('runtime', 'dirty');
  await until(async () => (await invoke('get_tabs')).tabs.find(t => t.is_active)?.activity.dirty_form, 'edited form before restart');
  const tab = (await invoke('get_tabs')).tabs.find(t => t.is_active);
  const transfer = await invoke('enqueue_download', { url: fixture.origin + '/download/file' });
  await until(async () => (await invoke('get_download_tasks')).find(t => t.id === transfer.id)?.received > 0, 'active download before restart');
  await click('[data-testid=open-tools]'); await click('[data-testid=tools-notes]');
  await until(() => script('return !!document.querySelector("textarea")'), 'pending note editor');
  await script(`const input=document.querySelector('textarea');input.value='Saved before runtime restart';input.dispatchEvent(new Event('input',{bubbles:true}));window.__TAURI_INTERNALS__.invoke('restart_webview_runtime');`);
  const review = await until(async () => { const s = await invoke('get_shutdown_state'); return s.phase === 'review' && s; }, 'runtime risk review');
  assert.equal(review.purpose, 'restart_runtime');
  assert.ok(review.review.tabs.some(t => t.id === tab.id && t.form));
  assert.ok(review.review.downloads.includes(transfer.id));
  assert.equal((await invoke('get_page_note', { url: tab.url })).text, 'Saved before runtime restart', 'pending draft flushed before risk review');
  const progress = (await invoke('get_download_tasks')).find(t => t.id === transfer.id).received;
  await click('[data-testid=shutdown-continue]');
  await until(async () => (await invoke('get_shutdown_state')).phase === 'idle', 'restart cancelled');
  assert.ok((await invoke('get_tabs')).tabs.some(t => t.id === tab.id));
  await until(async () => (await invoke('get_download_tasks')).find(t => t.id === transfer.id).received > progress, 'cancel keeps download running');
  await invoke('control_download', { taskId: transfer.id, action: 'pause' });
  await until(async () => (await invoke('get_download_tasks')).find(t => t.id === transfer.id).status === 'Paused', 'pause transfer before failure test');

  await invoke('restart_webview_runtime');
  const failedReview = await until(async () => { const s = await invoke('get_shutdown_state'); return s.phase === 'review' && s; }, 'second review');
  const failed = await invoke('confirm_shutdown', { attempt: failedReview.attempt, revision: failedReview.review.revision });
  assert.equal(failed.phase, 'error'); assert.equal(failed.error, 'runtime.restart_failed');
  assert.ok((await invoke('get_tabs')).tabs.length > 0, 'failed restart keeps application open');
  await click('[data-testid=shutdown-continue]');
  await until(async () => (await invoke('get_shutdown_state')).phase === 'idle', 'failed restart can be cancelled');

  configure('202.0.0.0', 'handoff');
  await invoke('restart_webview_runtime');
  const successful = await until(async () => { const s = await invoke('get_shutdown_state'); return s.phase === 'review' && s; }, 'successful restart review');
  await script(`window.__TAURI_INTERNALS__.invoke('confirm_shutdown',{attempt:arguments[0],revision:arguments[1]}).catch(()=>{});return null;`, successful.attempt, successful.review.revision);
  await until(() => existsSync(join(directory, 'config', 'runtime-restart-requested')), 'restart checkpoint handed off');
  await until(async () => !(await request('GET', `/session/${sessionId()}/window/handles`)).length, 'runtime process exits');
  await restart({ crash: true, beforeStart: () => configure('202.0.0.0', 'fail', '202.0.0.0') });
  assert.ok((await invoke('get_tabs')).tabs.some(t => t.id === tab.id && t.url === tab.url), 'session restored after handoff');
  assert.equal((await invoke('get_page_note', { url: tab.url })).text, 'Saved before runtime restart');
  assert.ok((await invoke('get_download_tasks')).some(t => t.id === transfer.id), 'download task restored');
  assert.equal((await invoke('get_webview_runtime_status')).restartRequired, false);
  configure('202.0.0.0', 'fail', 'unavailable');
  await until(async () => !(await invoke('get_webview_runtime_status')).supported, 'unsupported runtime detection');
  await assert.rejects(() => invoke('restart_webview_runtime'), /runtime.unavailable/);
  assert.equal(await script('return !!document.querySelector("[data-testid=runtime-notice]")'), false);
}
