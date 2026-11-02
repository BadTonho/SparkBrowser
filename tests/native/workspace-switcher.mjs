import assert from 'node:assert/strict';

export async function checkWorkspaceSwitcher(invoke, until, fixture, a, b) {
  const workspaces = await invoke('get_workspaces');
  const picker = visible => invoke('set_workspace_switcher_overlay', {
    visible, workspaces, activeWorkspaceId: a.workspace_id, backdropBlurPx: 16,
  });
  async function reportsAfter(ids, after) {
    await until(() => ids.every(id => fixture.reports.get(id)?.reportedAt > after), 'fresh visibility reports');
    return ids.map(id => fixture.reports.get(id));
  }

  for (const split of [false, true]) {
    await invoke('set_split_view', { leftTabId: split ? a.id : null, rightTabId: split ? b.id : null });
    await invoke('switch_tab', { tabId: a.id });
    const ids = split ? ['a', 'b'] : ['a'];
    const before = await reportsAfter(ids, Date.now());
    assert.ok(before.every(report => report.visibility === 'visible'));
    const counts = before.map(report => report.hiddenCount);
    const loads = ids.map(id => fixture.loads.get(id));
    await picker(true);
    // Include another lifecycle reconciliation while the picker is already open.
    await invoke('set_resident_limit', { limit: 1 });
    let reports = await reportsAfter(ids, Date.now() + 300);
    assert.ok(reports.every(report => report.visibility === 'visible'), 'workspace picker keeps the pages visible behind it');
    assert.deepEqual(reports.map(report => report.hiddenCount), counts, 'opening the picker must not briefly hide pages');
    await picker(false);
    await picker(true);
    reports = await reportsAfter(ids, Date.now() + 300);
    assert.ok(reports.every(report => report.visibility === 'visible'), 'reopening during the exit animation keeps pages visible');
    await picker(false);
    reports = await reportsAfter(ids, Date.now() + 300);
    assert.deepEqual(reports.map(report => report.hiddenCount), counts);
    assert.deepEqual(ids.map(id => fixture.loads.get(id)), loads, 'picker preserves existing documents');
    assert.equal(fixture.reports.get('a').value, 'artificial-test-draft');
    assert.ok(fixture.reports.get('a').scroll >= 500);
  }

  // Shell dialogs must still hide both native panes so their controls remain accessible.
  await invoke('set_overlay_visibility', { visible: true });
  let reports = await reportsAfter(['a', 'b'], Date.now() + 300);
  assert.ok(reports.every(report => report.visibility === 'hidden'));
  await invoke('set_overlay_visibility', { visible: false });
  reports = await reportsAfter(['a', 'b'], Date.now() + 300);
  assert.ok(reports.every(report => report.visibility === 'visible'));
  await invoke('set_split_view', { leftTabId: null, rightTabId: null });
  await invoke('set_resident_limit', { limit: 8 });
  await invoke('switch_tab', { tabId: a.id });
  console.log('Native workspace picker passed: page visibility, split panes, rapid reopening and document preservation.');
}
