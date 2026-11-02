import assert from 'node:assert/strict';
import { DatabaseSync } from 'node:sqlite';
import { resolve, relative } from 'node:path';
import { realpathSync } from 'node:fs';
import { validateRunDirectory } from './run-directory.mjs';
import { prepareTabs } from './setup.mjs';
export async function checkShutdown(context) {
  const { invoke, script, click, until, fixture, restart } = context;
  const { a, b } = await prepareTabs(context);
  const root = validateRunDirectory(context.directory);
  const data = realpathSync(resolve(root, 'data'));
  const file = realpathSync(resolve(data, 'veloce_browser.db'));
  assert.equal(relative(data,file),'veloce_browser.db');
  function sql(statement) { const db=new DatabaseSync(file); try { db.exec('PRAGMA busy_timeout=5000'); db.exec(statement); } finally { db.close(); } }
  const noteUrl = (await invoke("get_tabs")).tabs.find(t=>t.id===a.id).url;
  await click('[data-testid="open-tools"]');
  await until(() => script('return Boolean(document.querySelector("[data-testid=tools-dialog]"))'), 'tools rendered');
  await click('[data-testid="tools-notes"]');
  await until(() => script('return Boolean(document.querySelector("textarea"))'), 'note editor');
  await script(`const input=document.querySelector('textarea'); input.value='Draft saved during protected close'; input.dispatchEvent(new Event('input',{bubbles:true})); window.__TAURI_INTERNALS__.invoke('plugin:window|close',{label:'main'});`);
  const first = await until(async () => { const s=await invoke('get_shutdown_state'); return s.phase==='review' && s; }, 'shutdown review');
  assert.ok(first.review.tabs.some(t => t.id === a.id && t.form));
  assert.equal((await invoke('get_page_note',{url:noteUrl})).text, 'Draft saved during protected close');
  await assert.rejects(() => invoke('create_tab',{url:fixture.origin+'/page/forbidden'}), /shutdown.busy/);
  await until(() => script('return document.activeElement?.dataset.testid === "shutdown-continue"'), 'safe default focus');
  fixture.commands.set('b', 'dirty');
  await until(async () => (await invoke('get_tabs')).tabs.find(t => t.id === b.id)?.activity.dirty_form, 'new risk while reviewing');
  const changed = await invoke('confirm_shutdown',{attempt:first.attempt,revision:first.review.revision});
  assert.equal(changed.phase,'review');
  assert.ok(changed.review.revision > first.review.revision);
  assert.ok(changed.review.tabs.some(t=>t.id===b.id && t.form));
  await click('[data-testid="shutdown-continue"]');
  await until(async () => (await invoke('get_shutdown_state')).phase==='idle', 'cancel close');
  assert.ok((await invoke('get_tabs')).tabs.find(t=>t.id===a.id));
  await click('[data-testid="close-tools"]');

  // Fail only a shell note save, using the existing IPC boundary; keep its draft for retry.
  await click('[data-testid="open-tools"]');
  await until(() => script('return Boolean(document.querySelector("[data-testid=tools-dialog]"))'), 'tools rendered'); await click('[data-testid="tools-notes"]');
  await until(() => script('return Boolean(document.querySelector("textarea"))'), 'note editor again');
  sql("CREATE TRIGGER native_fail_note BEFORE INSERT ON page_notes BEGIN SELECT RAISE(ABORT,'artificial note failure'); END");
  await script(`const input=document.querySelector('textarea'); input.value='Retry retained draft'; input.dispatchEvent(new Event('input',{bubbles:true})); window.__TAURI_INTERNALS__.invoke('plugin:window|close',{label:'main'});`);
  await until(() => script('return Boolean(document.querySelector("[data-testid=shutdown-retry]"))'), 'save failure keeps window open');
  assert.equal((await invoke('get_shutdown_state')).phase,'flushing');
  sql('DROP TRIGGER native_fail_note');
  await click('[data-testid="shutdown-retry"]');
  await until(async () => (await invoke('get_shutdown_state')).phase==='review', 'retry notes then review');
  assert.equal((await invoke('get_page_note',{url:noteUrl})).text,'Retry retained draft');
  await click('[data-testid="shutdown-continue"]');
  await until(async () => (await invoke('get_shutdown_state')).phase==='idle', 'continue after retry');
  // Cancelling a review must not pause a live URL transfer.
  const transfer = await invoke('enqueue_download',{url:fixture.origin+'/download/file'});
  await until(async () => (await invoke('get_download_tasks')).find(t=>t.id===transfer.id)?.received>0,'shutdown transfer progress');
  await script("window.__TAURI_INTERNALS__.invoke('plugin:window|close',{label:'main'});");
  await until(async () => (await invoke('get_shutdown_state')).phase==='review','transfer review');
  const progress = (await invoke('get_download_tasks')).find(t=>t.id===transfer.id).received;
  await click('[data-testid="shutdown-continue"]');
  await until(async () => (await invoke('get_shutdown_state')).phase==='idle','cancel transfer review');
  await until(async () => (await invoke('get_download_tasks')).find(t=>t.id===transfer.id).received>progress,'transfer keeps running after cancel');
  await invoke('control_download',{taskId:transfer.id,action:'pause'});
  await until(async () => (await invoke('get_download_tasks')).find(t=>t.id===transfer.id).status==='Paused','pause before persistence fault');
  // A failed final transaction must not close the window or mark the run clean.
  const db = new DatabaseSync(file);
  try {
    db.exec("PRAGMA busy_timeout=5000; CREATE TRIGGER native_fail_close BEFORE UPDATE ON browser_run_state WHEN NEW.clean_exit=1 BEGIN SELECT RAISE(ABORT,'artificial close failure'); END");
    await script("window.__TAURI_INTERNALS__.invoke('plugin:window|close',{label:'main'});");
    await until(async () => (await invoke('get_shutdown_state')).phase==='review','review before persistence fault');
    await click('[data-testid="shutdown-confirm"]');
    await until(async () => (await invoke('get_shutdown_state')).phase==='error','persistence fault keeps browser open');
    assert.equal(db.prepare('SELECT clean_exit FROM browser_run_state').get().clean_exit,0);
    db.exec('DROP TRIGGER native_fail_close');
    await click('[data-testid="shutdown-continue"]');
    await until(async () => (await invoke('get_shutdown_state')).phase==='idle','usable after failure');
  } finally { db.close(); }
  await restart();
  assert.equal((await invoke('get_recovery_status')).recovered,false,'confirmed close records a clean exit');
}
