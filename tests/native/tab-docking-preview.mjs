import assert from 'node:assert/strict';

export function previewMeasurements({ fixture, until }, box) {
  const companion = async (name, side, ratio = 0.5) => {
    const bounds = await box(), since = Date.now();
    const width = (bounds.width - 8) * (side === 'right' ? ratio : 1 - ratio);
    await until(() => {
      const report = fixture.reports.get(name);
      return report?.reportedAt >= since && report.visibility === 'visible' && Math.abs(report.width - width) <= 2;
    }, name + ' occupies its final companion width while held');
    return width;
  };
  const track = async name => {
    const previous = fixture.reports.get(name).widthTracking;
    fixture.commands.set(name, 'reset-widths');
    await until(() => fixture.reports.get(name)?.widthTracking > previous, 'record native resize events through drop');
    return fixture.reports.get(name).widthTracking;
  };
  const continuous = async (name, width, tracking) => {
    const since = Date.now();
    await until(() => fixture.reports.get(name)?.reportedAt >= since, 'fresh companion report after drop');
    const report = fixture.reports.get(name);
    assert.equal(report.widthTracking, tracking, 'same document survives confirmation');
    assert.ok(report.widths.every(value => Math.abs(value - width) <= 2), 'companion never expands between preview and committed split: ' + report.widths);
  };
  return { companion, track, continuous };
}

export async function checkUnloadedCompanion(context, { active, other, name, holdAtEdge, checkPreview, cancelHold, up, measurements }) {
  const { invoke, fixture, until, wait } = context;
  await invoke('set_split_view', { leftTabId: null, rightTabId: null });
  await invoke('switch_tab', { tabId: active.id });
  await wait(250);
  await holdAtEdge(active.id, 'right');
  await measurements.companion(name, 'right');
  const loads = fixture.loads.get(name);
  await invoke('hibernate_tab', { tabId: other.id });
  await checkPreview();
  await wait(300);
  assert.equal((await invoke('get_tabs')).tabs.find(t => t.id === other.id).is_hibernated, true, 'preview does not wake a companion hibernated during the gesture');
  assert.equal(fixture.loads.get(name), loads);
  await cancelHold();
  await holdAtEdge(active.id, 'right');
  await checkPreview();
  await wait(300);
  assert.equal(fixture.loads.get(name), loads, 'hovering reserves a hibernated pane without loading');
  assert.equal((await invoke('get_tabs')).split_layout, null);
  await up();
  await until(async () => (await invoke('get_tabs')).split_layout?.left_tab_id === other.id, 'drop commits hibernated companion');
  await until(() => fixture.loads.get(name) === loads + 1, 'companion loads only after successful drop');
  const packet = await invoke('get_tabs'), split = packet.split_layout;
  const revision = Date.now() * 2;
  const presentation = {
    tabId: active.id, side: 'right', label: 'Drop on right', locale: 'en', pointer: { x: 400, y: 300 },
    context: { workspaceId: split.workspace_id, activeTabId: active.id,
      splitKey: `${split.workspace_id}:${split.left_tab_id}:${split.right_tab_id}` },
  };
  await invoke('set_tab_drag_preview', { revision, presentation });
  await measurements.companion(name, 'right');
  await invoke('set_tab_drag_preview', { revision: revision + 1, presentation: null });
  await invoke('set_tab_drag_preview', { revision, presentation });
  const since = Date.now();
  await until(() => fixture.reports.get('dock-a')?.reportedAt >= since && fixture.reports.get('dock-a').visibility === 'visible', 'late preview cannot hide a page after cancellation');
  assert.deepEqual((await invoke('get_tabs')).split_layout, split, 'preview and stale requests never change committed layout');
}
