import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

export function backupPreferences() {
  // Reuse the checked-in action metadata; no copy of its default key combinations.
  const source = readFileSync('src/lib/stores/shortcutsStore.ts', 'utf8');
  const array = source.slice(source.indexOf('= [', source.indexOf('export const SHORTCUT_DEFINITIONS')) + 2).split('\n];', 1)[0] + '\n]';
  const combo = (key, options = {}) => ({ key, ctrl: false, alt: false, shift: false, meta: false, ...options });
  const definitions = Function('combo', `return ${array}`)(combo);
  return { tab_layout: 'horizontal', show_bookmarks_bar: true, overlay_blur_px: 16, workspace_switcher_display: 'toolbar', default_search_engine: 'google', auto_hibernate_mode: 10, resident_limit: 8, language: 'system', keyboard_shortcuts: Object.fromEntries(definitions.map(item => [item.id, item.defaults])), update_channel: 'stable' };
}
export async function checkBackup(context) {
  const { invoke, restart, fixture, script, until } = context;
  const before = await invoke('get_tabs');
  const resident = (await invoke('get_diagnostics')).resident_webviews;
  const prefs = backupPreferences();
  const workspaces = await invoke('get_workspaces');
  const source = { format: 'veloce-backup', version: 1, created_at: 1, preferences: prefs,
    data: { bookmarks: await invoke('get_bookmark_data'), workspaces, tabs: before.tabs.map(({ id, title, url, workspace_id, is_pinned, is_active }) => ({ id, title, url, workspace_id, is_pinned, is_active })), split_layouts: [], sessions: [], notes: [], site_search: [] } };
  source.version=3;
  source.preferences = { ...prefs, auto_skip_ads_enabled:true,pip_window_shadow:true,language: 'en', overlay_blur_px: 5, resident_limit: 4, update_channel: 'beta' };
  const active = source.data.tabs.find(tab => tab.is_active);
  source.data.tabs.push({ ...active, id: 'foreign-duplicate-1', is_active: false }, { ...active, id: 'foreign-duplicate-2', is_active: false, is_pinned: true });
  source.data.workspaces.push({ id: 'foreign-workspace', name: 'Imported artificial workspace', icon: 'T', accent_primary: '#123456', accent_secondary: '#654321' });
  for (const id of ['left', 'right']) source.data.tabs.push({ id: `foreign-${id}`, title: `Imported ${id}`, url: fixture.origin + `/page/${id}`, workspace_id: 'foreign-workspace', is_pinned: false, is_active: false });
  source.data.split_layouts.push({ workspace_id: 'foreign-workspace', left_tab_id: 'foreign-left', right_tab_id: 'foreign-right', focused_tab_id: 'foreign-right', ratio: 0.7 });
  source.data.notes.push({ url: fixture.origin + '/page/note#fragment', title: 'Imported note', text: '<b>Plain backup text</b>', updated_at: 1 });
  source.data.sessions.push({ id: 400, name: 'Artificial imported session', pages: [{ title: 'First', url: fixture.origin + '/page/a', is_pinned: true, is_selected: true }] });
  source.data.bookmarks.folders.push({ id: 400, name: 'Artificial imported folder' });
  source.data.bookmarks.bookmarks.push({ id: 400, title: 'Artificial imported bookmark', url: fixture.origin + '/page/bookmark', icon: 'T', folder_id: 400 });
  source.data.site_search.push({ keyword: 'backup', name: 'Artificial imported search', template: fixture.origin + '/page/search?q={query}' });
  const preview = () => invoke('preview_backup_import', { document: source, currentPreferences: prefs });
  const commit = (token, applyPreferences = false) => invoke('commit_backup_import', { token, applyPreferences, currentPreferences: prefs });
  let review = await preview(); assert.equal(review.added.tabs, 4); assert.equal(review.added.workspaces, 1);
  await invoke('save_page_note', { url: fixture.origin + '/changed', title: 'Changed during review', text: 'Keep' });
  await assert.rejects(commit(review.token), /backup.preview_stale/);
  assert.equal((await invoke('get_tabs')).tabs.length, before.tabs.length);
  review = await preview(); const result = await commit(review.token);
  assert.equal(result.pending_preferences, null);
  assert.notEqual(await script('return localStorage.getItem("veloce_pip_window_shadow");'),'true','data-only import leaves the local preference unchanged');
  assert.equal((await invoke('get_privacy_stats')).auto_skip_ads_enabled, false, 'data-only import keeps auto skip unchanged');
  assert.deepEqual(result.snapshot.tabs.slice(0, before.tabs.length).map(tab => [tab.id, tab.is_active]), before.tabs.map(tab => [tab.id, tab.is_active]));
  const imported = result.snapshot.tabs.slice(before.tabs.length);
  assert.ok(imported.every(tab => tab.is_hibernated && !tab.is_visible && !tab.is_active));
  assert.ok(imported.every(tab => !source.data.tabs.some(original => original.id === tab.id)));
  assert.equal((await invoke('get_diagnostics')).resident_webviews, resident, 'import does not create WebViews');
  assert.equal((await invoke('get_page_note', { url: fixture.origin + '/page/note#other' })).text, '<b>Plain backup text</b>');
  await assert.rejects(commit(review.token), /backup.preview_missing/);
  review = await preview(); assert.ok(Object.values(review.added).every(count => count === 0));
  await commit(review.token);
  review = await preview(); const withPreferences = await commit(review.token, true);
  const taskId = withPreferences.pending_preferences.id;
  await invoke('finish_backup_preferences', { id: taskId, error: 'Artificial preference failure' });
  await restart();
  const task = await invoke('get_backup_pending_preferences'); assert.equal(task.id, taskId); assert.equal(task.error, 'Artificial preference failure');
  await until(() => script(`return Boolean(document.querySelector('[data-testid="backup-pending"]'))`), 'persistent preference notice');
  await script(`document.querySelector('[data-testid="open-settings"]').click();`);
  await until(() => script(`return Boolean(document.querySelector('[data-testid="settings-data_backup"]'))`), 'backup settings category');
  await script(`document.querySelector('[data-testid="settings-data_backup"]').click();`);
  await until(() => script(`return Boolean(document.querySelector('[data-testid="backup-retry-preferences"]'))`), 'preference retry control');
  await script(`window.__nativeOriginalStorageWrite = Storage.prototype.setItem; Storage.prototype.setItem = function() { throw new DOMException('Artificial preference failure', 'QuotaExceededError'); }; document.querySelector('[data-testid="backup-retry-preferences"]').click();`);
  await until(async () => { const current = await invoke('get_backup_pending_preferences'); try { return JSON.parse(current?.error ?? '{}').code === 'storage.failed'; } catch { return false; } }, 'actual preference write failure remains pending');
  assert.equal((await invoke('get_tabs')).tabs.length, result.snapshot.tabs.length);
  await script(`Storage.prototype.setItem = window.__nativeOriginalStorageWrite; delete window.__nativeOriginalStorageWrite; document.querySelector('[data-testid="backup-retry-preferences"]').click();`);
  await until(async () => !(await invoke('get_backup_pending_preferences')), 'preference retry acknowledged');
  assert.equal(await script(`return localStorage.getItem('veloce_language')`), 'en');
  assert.equal(await script(`return localStorage.getItem('veloce_overlay_blur_px')`), '5');
  assert.equal(await script('return localStorage.getItem("veloce_pip_window_shadow");'),'true','preference retry applies imported shadow');
  assert.equal(await script('return localStorage.getItem("veloce_auto_skip_ads");'),'true','preference retry applies imported auto skip');
  assert.equal((await invoke('get_privacy_stats')).auto_skip_ads_enabled, true);
  assert.equal((await invoke('get_update_state')).channel, 'beta');
  await script(`document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));`);
  assert.equal((await invoke('get_tabs')).tabs.length, result.snapshot.tabs.length);
  const importedWorkspace = (await invoke('get_workspaces')).find(row => row.name === 'Imported artificial workspace');
  await invoke('switch_workspace', { workspaceId: importedWorkspace.id });
  const restored = await invoke('get_tabs'); assert.equal(restored.split_layout.ratio, 0.7);
  assert.ok(restored.tabs.filter(tab => tab.is_visible).every(tab => tab.workspace_id === importedWorkspace.id));
}
