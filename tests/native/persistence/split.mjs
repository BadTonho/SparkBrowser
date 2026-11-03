import assert from 'node:assert/strict';
import { withScenarioDatabase } from './database.mjs';

export async function checkSplitPersistence(context) {
  const { invoke, fixture, until, directory, script, wait } = context;
  await invoke('navigate_to', { input: fixture.origin + '/page/failure-left' });
  await until(() => fixture.reports.has('failure-left'), 'left fixture');
  const a = (await invoke('get_tabs')).tabs.find(t => t.is_active);
  const b = (await invoke('create_tab', { url: fixture.origin + '/page/failure-right' })).tabs.find(t => t.is_active);
  await until(() => fixture.reports.has('failure-right'), 'right fixture');
  fixture.commands.set('failure-right', 'spa');
  await until(async () => (await invoke('get_tabs')).tabs.find(t => t.id === b.id).navigation.can_go_back, 'right native history');
  const c = (await invoke('create_tab', { url: fixture.origin + '/page/failure-other' })).tabs.find(t => t.is_active);
  await until(() => fixture.reports.has('failure-other'), 'third fixture');
  await invoke('set_split_view', { leftTabId: a.id, rightTabId: b.id });
  await invoke('set_resident_limit', { limit: 2 });
  await invoke('get_history', { limit: 1 });
  await withScenarioDatabase(directory, async db => {
    assert.match(c.id, /^tab-\d+$/);
    db.exec(`CREATE TRIGGER fail_split BEFORE INSERT ON workspace_splits WHEN NEW.right_tab_id='${c.id}' BEGIN SELECT RAISE(ABORT,'injected split failure'); END`);
    try {
      await assert.rejects(invoke('set_split_view', { leftTabId: a.id, rightTabId: c.id }), /injected split failure/);
      const packet = await invoke('get_tabs');
      assert.equal(packet.split_layout.left_tab_id, a.id);
      assert.equal(packet.split_layout.right_tab_id, b.id);
      assert.equal(packet.tabs.find(t => t.id === b.id).navigation.can_go_back, true);
      await wait(300);
      assert.equal(fixture.loads.get('failure-right'), 1, 'failed change must not recreate the old pane');
      assert.equal(fixture.loads.get('failure-other'), 1, 'failed change must not wake the new pane');
      assert.equal(db.prepare('SELECT right_tab_id FROM workspace_splits').get().right_tab_id, b.id);
    } finally { db.exec('DROP TRIGGER fail_split'); }
    await invoke('set_split_view', { leftTabId: a.id, rightTabId: c.id });
    await until(() => fixture.loads.get('failure-other') === 2, 'successful replacement wakes the third page');
    await invoke('set_split_view', { leftTabId: a.id, rightTabId: b.id });
    await until(() => fixture.loads.get('failure-right') === 2, 'successful replacement restores the right pane');
    await invoke('get_history', { limit: 1 });
    const split = (await invoke('get_tabs')).split_layout;
    const key = `${split.workspace_id}:${a.id}:${b.id}`;
    const bounds = await script('const r=document.querySelector(".viewport-frame").getBoundingClientRect(); return {x:r.left,y:r.top,width:r.width,height:r.height}');
    await until(() => fixture.reports.get('failure-left')?.width > 100, 'measured pane width');
    const width = fixture.reports.get('failure-left').width;
    await invoke('sync_viewport_bounds', { bounds, splitKey: key, splitRatio: 0.7 });
    await until(() => fixture.reports.get('failure-left').width > width * 1.3, 'preview resizes actual native geometry');
    assert.equal((await invoke('get_tabs')).split_layout.ratio, 0.5, 'snapshots retain the committed ratio');
    // An unrelated session save during dragging must not persist the preview.
    await invoke('set_tab_pinned', { tabId: a.id, pinned: true });
    assert.equal(db.prepare('SELECT ratio FROM workspace_splits').get().ratio, 0.5);
    db.exec("CREATE TRIGGER fail_ratio BEFORE UPDATE ON workspace_splits WHEN NEW.ratio != OLD.ratio BEGIN SELECT RAISE(ABORT,'injected ratio failure'); END");
    try {
      await assert.rejects(invoke('commit_split_ratio', { key, ratio: 0.7 }), /injected ratio failure/);
      assert.equal((await invoke('get_tabs')).split_layout.ratio, 0.5);
      assert.equal(db.prepare('SELECT ratio FROM workspace_splits').get().ratio, 0.5);
      await until(() => Math.abs(fixture.reports.get('failure-left').width - width) <= 2, 'failed commit restores native pane size');
    } finally { db.exec('DROP TRIGGER fail_ratio'); }
    await invoke('commit_split_ratio', { key, ratio: 0.65 });
    assert.equal((await invoke('get_tabs')).split_layout.ratio, 0.65);
    assert.equal(db.prepare('SELECT ratio FROM workspace_splits').get().ratio, 0.65);
    await until(() => fixture.reports.get('failure-left').width > width * 1.2, 'successful commit applies geometry');
    await invoke('sync_viewport_bounds', { bounds, splitKey: key, splitRatio: 0.6 });
    await invoke('sync_viewport_bounds', { bounds, splitKey: key, splitRatio: 0.65 });
    assert.equal((await invoke('get_tabs')).split_layout.ratio, 0.65, 'canceling a preview keeps the saved value');
  });
}
