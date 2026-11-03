import assert from 'node:assert/strict';
export async function checkFeatures(invoke, until, fixture, a, b) {
  await invoke('set_split_view', { leftTabId: null, rightTabId: null });
  const keyword = 'e2e';
  await invoke('delete_site_search_shortcut', { keyword });
  await invoke('save_site_search_shortcut', { previous: null, item: { keyword, name: 'Fixture', template: fixture.origin + '/page/b?q={query}' } });
  const created = await invoke('create_tab', { url: 'E2E ação & + /%', searchEngine: 'google' });
  const c = created.tabs.find(tab => tab.is_active);
  assert.equal(new URL(c.url).searchParams.get('q'), 'ação & + /%');
  await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === c.id)?.navigation.loading === false, 'site search fixture loaded');
  await invoke('switch_tab', { tabId: a.id });
  const bulk = await invoke('close_tabs_reviewed', { tabIds: [a.id, b.id, c.id] });
  assert.deepEqual(bulk.closed, [c.id]); assert.equal(bulk.preserved.length, 2);
  await invoke('delete_site_search_shortcut', { keyword });

  const url = fixture.origin + '/page/a?note=1';
  await invoke('save_page_note', { url: url + '#one', title: 'Artificial note', text: '<script>plain text</script>' });
  assert.equal((await invoke('get_page_note', { url: url + '#two' })).text, '<script>plain text</script>');
  assert.ok((await invoke('search_page_notes', { query: 'Artificial note' })).some(note => note.url === url));
  await invoke('delete_page_note', { url });

  const name = 'Native fixture ' + Date.now();
  const before = (await invoke('get_tabs')).tabs;
  await invoke('save_named_session', { id: null, name });
  const saved = (await invoke('get_saved_sessions')).find(session => session.name === name);
  await invoke('rename_named_session', { id: saved.id, name: name + ' renamed' });
  const restored = (await invoke('restore_named_session', { id: saved.id })).snapshot.tabs;
  assert.equal(restored.length, before.length + saved.pages.length);
  assert.ok(before.every(tab => restored.some(item => item.id === tab.id)));
  assert.equal(new Set(restored.map(tab => tab.id)).size, restored.length);
  await invoke('delete_named_session', { id: saved.id });
  await invoke('switch_tab', { tabId: a.id });

  let split = await invoke('set_split_view', { leftTabId: a.id, rightTabId: b.id });
  assert.equal(split.tabs.filter(tab => tab.is_visible).length, 2);
  await invoke('commit_split_ratio', { key: `${a.workspace_id}:${a.id}:${b.id}`, ratio: 0.65 });
  await invoke('set_resident_limit', { limit: 1 });
  split = await invoke('get_tabs');
  assert.equal(split.tabs.filter(tab => tab.is_visible && !tab.is_hibernated).length, 2);
  const d = (await invoke('create_tab', { url: fixture.origin + '/page/b', searchEngine: 'google' })).tabs.find(tab => tab.is_active);
  split = await invoke('get_tabs'); assert.equal(split.split_layout.left_tab_id, d.id);
  const group = split.split_layout.left_tab_ids, index = group.indexOf(d.id);
  const neighbor = group[index + 1] ?? group[index - 1];
  split = await invoke('close_tab', { tabId: d.id, searchEngine: 'google' });
  assert.equal(split.split_layout.left_tab_id, neighbor, 'closing a selected tab chooses a member of the same group');
  assert.equal(split.split_layout.right_tab_id, b.id);
  assert.equal(split.tabs.find(tab => tab.is_active).id, neighbor);
  await invoke('switch_tab', { tabId: a.id });
  await invoke('set_split_view', { leftTabId: a.id, rightTabId: b.id });
  await invoke('commit_split_ratio', { key: `${a.workspace_id}:${a.id}:${b.id}`, ratio: 0.65 });
  await invoke('set_resident_limit', { limit: 8 });
}
