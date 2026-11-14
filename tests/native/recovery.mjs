import assert from 'node:assert/strict';
import { DatabaseSync } from 'node:sqlite';
import { realpathSync } from 'node:fs';
import { resolve, relative } from 'node:path';
import { validateRunDirectory } from './run-directory.mjs';

const metadata = packet => packet.tabs.map(({ id, url, workspace_id, is_active, is_pinned }) => ({ id, url, workspace_id, is_active, is_pinned }));
async function prepareRecovery({ invoke, fixture, until }) {
  assert.equal((await invoke('get_recovery_status')).recovered, false);
  await invoke('set_resident_limit', { limit: 8 });
  const workspace = 'recovery-fixture';
  await invoke('create_workspace', { id: workspace, name: 'Recovery fixture', icon: 'R', accentPrimary: '#123456', accentSecondary: '#654321' });
  await invoke('switch_workspace', { workspaceId: workspace });
  const create = async name => {
    const packet = await invoke('create_tab', { url: fixture.origin + '/page/' + name });
    const tab = packet.tabs.find(tab => tab.is_active);
    await until(async () => !(await invoke('get_tabs')).tabs.find(item => item.id === tab.id)?.navigation.loading, 'recovery fixture loaded');
    return tab;
  };
  const left = await create('recovery-left');
  const right = await create('recovery-right');
  const hidden = await create('recovery-hidden');
  await invoke('set_tab_pinned', { tabId: hidden.id, pinned: true });
  await invoke('switch_tab', { tabId: right.id });
  await invoke('set_split_view', { leftTabId: left.id, rightTabId: right.id });
  await invoke('commit_split_ratio', { key: `${workspace}:${left.id}:${right.id}`, ratio: 0.63 });
  await invoke('get_history', { limit: 10 });
  return { before: await invoke('get_tabs'), workspaces: await invoke('get_workspaces') };
}
async function assertRecovered(context, before, workspaces) {
  const { invoke, until, script } = context;
  const status = await invoke('get_recovery_status');
  assert.equal(status.recovered, true);
  assert.equal(status.restored_tabs, before.tabs.length);
  const after = await invoke('get_tabs');
  assert.deepEqual(metadata(after), metadata(before));
  assert.deepEqual(after.split_layout, before.split_layout);
  assert.deepEqual(await invoke('get_workspaces'), workspaces);
  assert.ok(after.tabs.filter(tab => !tab.is_visible).every(tab => tab.is_hibernated), 'all background tabs start hibernated, including pins');
  assert.equal((await invoke('get_diagnostics')).resident_webviews, 2, 'only the current split pair creates WebViews');
  await until(() => script(`return Boolean(document.querySelector('[data-testid="recovery-notice"]'))`), 'translated recovery notice');
}
export async function checkRecovery(context) {
  const { invoke, restart, script, until } = context;
  const { before, workspaces } = await prepareRecovery(context);
  await restart({ crash: true });
  await assertRecovered(context, before, workspaces);
  await script(`document.querySelector('[data-testid="recovery-organizer"]').click();`);
  await until(() => script(`return Boolean(document.querySelector('[data-testid="tools-tabs"][aria-selected="true"]'))`), 'recovery opens tab organizer');
  await script(`document.querySelector('[data-testid="close-tools"]').click();`);
  await script(`document.querySelector('[data-testid="recovery-dismiss"]').click();`);
  await until(() => script(`return !document.querySelector('[data-testid="recovery-notice"]')`), 'recovery notice dismissed');
  await restart();
  assert.equal((await invoke('get_recovery_status')).recovered, false, 'normal exit must save a clean marker');
  assert.deepEqual(metadata(await invoke('get_tabs')), metadata(before));
  assert.equal(await script(`return Boolean(document.querySelector('[data-testid="recovery-notice"]'))`), false);
}
function withScenarioDatabase(directory, action) {
  // Only fixture data from the already identity-verified, stopped test scenario is edited.
  const root = validateRunDirectory(directory);
  const file = realpathSync(resolve(root, 'data', 'spark_browser.db'));
  assert.equal(relative(realpathSync(resolve(root, 'data')), file), 'spark_browser.db');
  const db = new DatabaseSync(file);
  try { db.exec('PRAGMA busy_timeout=5000'); return action(db); } finally { db.close(); }
}
export async function checkRecoveryFailure(context) {
  const { before, workspaces } = await prepareRecovery(context);
  await context.restart({ crash: true, expectFailure: true, beforeStart: directory => withScenarioDatabase(directory, db => {
    db.exec("CREATE TRIGGER native_fail_restore BEFORE UPDATE ON browser_run_state WHEN NEW.restoration_confirmed=1 BEGIN SELECT RAISE(ABORT,'Artificial startup persistence failure'); END");
  }) });
  withScenarioDatabase(context.directory, db => {
    assert.deepEqual(db.prepare('SELECT tab_id FROM session_tabs ORDER BY position').all().map(row => row.tab_id), before.tabs.map(tab => tab.id));
    assert.equal(db.prepare('SELECT COUNT(*) AS count FROM workspace_splits').get().count, 1);
    assert.equal(db.prepare('SELECT clean_exit FROM browser_run_state').get().clean_exit, 0);
    assert.equal(db.prepare('SELECT restoration_confirmed FROM browser_run_state').get().restoration_confirmed, 0);
    db.exec('DROP TRIGGER native_fail_restore');
  });
  await context.restart();
  await assertRecovered(context, before, workspaces);
}
