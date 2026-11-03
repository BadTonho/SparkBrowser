import assert from 'node:assert/strict';
import { withScenarioDatabase } from './database.mjs';

export async function checkWorkspaceDeletePersistence({ invoke, fixture, until, directory }) {
  await invoke('navigate_to', { input: fixture.origin + '/page/delete-personal' });
  await until(() => fixture.reports.has('delete-personal'), 'personal fixture');
  const personal = (await invoke('get_tabs')).tabs.find(t => t.is_active);
  async function openWork(name) {
    await invoke('switch_workspace', { workspaceId: 'ws-work' });
    await invoke('navigate_to', { input: fixture.origin + '/page/' + name });
    await until(() => fixture.reports.has(name), 'work fixture');
    await invoke('get_history', { limit: 1 });
  }
  async function failThenRetry() {
    const before = await invoke('get_tabs');
    await withScenarioDatabase(directory, async db => {
      const saved = db.prepare('SELECT * FROM session_tabs ORDER BY position').all();
      const usage = db.prepare('SELECT * FROM tab_usage_events ORDER BY id').all();
      db.exec("CREATE TRIGGER fail_activation BEFORE INSERT ON tab_usage_events BEGIN SELECT RAISE(ABORT,'injected delete activation failure'); END");
      try {
        await assert.rejects(invoke('delete_workspace', { workspaceId: 'ws-work', action: 'close' }), /injected delete activation failure/);
        const after = await invoke('get_tabs');
        assert.equal(after.tabs.filter(t => t.is_active).length, 1);
        assert.equal(after.tabs.find(t => t.is_active).id, before.tabs.find(t => t.is_active).id);
        assert.deepEqual(after.tabs.map(t => t.id), before.tabs.map(t => t.id));
        assert.ok((await invoke('get_workspaces')).some(w => w.id === 'ws-work'));
        assert.deepEqual(db.prepare('SELECT * FROM session_tabs ORDER BY position').all(), saved);
        assert.deepEqual(db.prepare('SELECT * FROM tab_usage_events ORDER BY id').all(), usage);
        assert.equal(db.prepare("SELECT count(*) AS n FROM workspaces WHERE id='ws-work'").get().n, 1);
      } finally { db.exec('DROP TRIGGER fail_activation'); }
      const result = await invoke('delete_workspace', { workspaceId: 'ws-work', action: 'close' });
      assert.equal(result.tabs.tabs.filter(t => t.is_active).length, 1);
      assert.equal(result.tabs.tabs.find(t => t.is_active).workspace_id, 'ws-personal');
      assert.ok(!result.workspaces.some(w => w.id === 'ws-work'));
      return result;
    });
  }
  await openWork('delete-work-existing');
  await failThenRetry();
  assert.equal((await invoke('get_tabs')).tabs.find(t => t.is_active).id, personal.id);
  await invoke('create_workspace', { id: 'ws-work', name: 'Work', icon: 'W', accentPrimary: '#123456', accentSecondary: '#654321' });
  await openWork('delete-work-new');
  await invoke('close_tab', { tabId: personal.id });
  await failThenRetry();
  const fallback = (await invoke('get_tabs')).tabs.find(t => t.is_active);
  assert.notEqual(fallback.id, personal.id, 'empty Personal workspace receives a new active tab');
  const second = (await invoke('create_tab', { url: fixture.origin + '/page/delete-second' })).tabs.find(t => t.is_active);
  await until(() => fixture.reports.has('delete-second'), 'second destination fixture');
  await invoke('create_tab', { url: fixture.origin + '/page/delete-hidden' });
  await until(() => fixture.reports.has('delete-hidden'), 'unpaired destination fixture');
  await invoke('set_split_view', { leftTabId: fallback.id, rightTabId: second.id });
  await invoke('create_workspace', { id: 'ws-work', name: 'Work', icon: 'W', accentPrimary: '#123456', accentSecondary: '#654321' });
  await openWork('delete-work-split');
  await failThenRetry();
  const restored = await invoke('get_tabs');
  assert.equal(restored.tabs.find(t => t.is_active).id, fallback.id, 'closing returns to the remembered focused pane');
  assert.equal(restored.tabs.filter(t => t.is_visible).length, 2);
  assert.equal(restored.split_layout.focused_tab_id, fallback.id);
  await invoke('switch_workspace', { workspaceId: 'ws-study' });
  const moved = (await invoke('get_tabs')).tabs.find(t => t.is_active).id;
  const result = await invoke('delete_workspace', { workspaceId: 'ws-study', action: 'move', targetWorkspaceId: 'ws-personal' });
  assert.equal(result.tabs.tabs.find(t => t.is_active).id, moved);
  assert.equal(result.active_workspace_id, 'ws-personal');
  assert.equal(result.tabs.tabs.filter(t => t.is_visible).length, 2, 'moving into a split replaces its focused pane');
  assert.equal(result.tabs.split_layout.left_tab_id, moved);
  assert.equal(result.tabs.split_layout.right_tab_id, second.id);
  assert.equal(result.tabs.split_layout.focused_tab_id, moved);
}
