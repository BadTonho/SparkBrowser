import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createSettingsDisplayController, findSettingsTab } from '../src/lib/services/settings-display.js';

test('settings tab is reused only in the current workspace, including hibernated tabs', () => {
  const tabs = [
    { id: 'other', url: 'spark://settings', workspace_id: 'other', is_hibernated: false },
    { id: 'local', url: 'spark://settings', workspace_id: 'current', is_hibernated: true },
    { id: 'site', url: 'https://example.test', workspace_id: 'current', is_hibernated: false },
  ];
  assert.equal(findSettingsTab(tabs, 'current')?.id, 'local');
  assert.equal(findSettingsTab(tabs, 'missing'), undefined);
});

function fixture() {
  let mode = 'popup', saved = 'popup', popup = true;
  let tabs = [{ id: 'page', url: 'https://example.test', workspace_id: 'ws', is_active: true }];
  const calls = [];
  let fail = '';
  const options = {
    getMode: () => mode, tabs: () => tabs, workspace: () => 'ws', popupOpen: () => popup,
    save(value) { calls.push(`save:${value}`); if (fail === 'save') return false; saved = value; return true; },
    set(value) { mode = value; },
    async openTab() { calls.push('open-tab'); if (fail === 'open-tab') throw Error('tab failed'); const existing = findSettingsTab(tabs, 'ws'); tabs = tabs.map(tab => ({ ...tab, is_active: tab.id === existing?.id })); if (existing) return existing.id; tabs.push({ id: 'settings', url: 'spark://settings', workspace_id: 'ws', is_active: true }); return 'settings'; },
    async closeTab(id) { calls.push(`close:${id}`); tabs = tabs.filter(tab => tab.id !== id); },
    async selectTab(id) { calls.push(`select:${id}`); tabs = tabs.map(tab => ({ ...tab, is_active: tab.id === id })); },
    async closePopup() { calls.push('close-popup'); if (fail === 'close-popup') throw Error('popup failed'); popup = false; },
    async openPopup() { calls.push('open-popup'); if (fail === 'open-popup') throw Error('popup failed'); popup = true; },
  };
  return { controller: createSettingsDisplayController(options), calls, get: () => ({ mode, saved, popup, tabs }), fail: value => { fail = value; }, setTabs: value => { tabs = value; }, setPopup: value => { popup = value; } };
}

test('switching to a tab and back saves the preference and closes only settings', async () => {
  const f = fixture();
  await f.controller.change('tab');
  assert.equal(f.get().mode, 'tab'); assert.equal(f.get().saved, 'tab'); assert.equal(f.get().popup, false);
  assert.equal(f.get().tabs.find(tab => tab.is_active)?.id, 'settings');
  await f.controller.change('popup');
  assert.equal(f.get().mode, 'popup'); assert.equal(f.get().saved, 'popup'); assert.equal(f.get().popup, true);
  assert.deepEqual(f.get().tabs.map(tab => tab.id), ['page']);
});

test('an explicitly opened settings tab can return to the already preferred popup mode', async () => {
  const f = fixture();
  f.setPopup(false);
  f.setTabs([{ id: 'settings', url: 'spark://settings', workspace_id: 'ws', is_active: true }]);
  await f.controller.change('popup');
  assert.equal(f.get().popup, true);
  assert.deepEqual(f.get().tabs, []);
  assert.equal(f.get().mode, 'popup');
});

test('failed tab transfer restores the previous selection and preference', async () => {
  const f = fixture(); f.fail('close-popup');
  await assert.rejects(f.controller.change('tab'), /popup failed/);
  assert.equal(f.get().mode, 'popup'); assert.equal(f.get().saved, 'popup');
  assert.deepEqual(f.get().tabs.map(tab => tab.id), ['page']);
  assert.equal(f.get().tabs[0].is_active, true);
});

test('failed transfer keeps an existing settings tab and restores the selected page', async () => {
  const f = fixture();
  f.setTabs([{ id: 'page', url: 'https://example.test', workspace_id: 'ws', is_active: true }, { id: 'settings', url: 'spark://settings', workspace_id: 'ws', is_active: false }]);
  f.fail('close-popup');
  await assert.rejects(f.controller.change('tab'), /popup failed/);
  assert.deepEqual(f.get().tabs.map(tab => tab.id), ['page', 'settings']);
  assert.equal(f.get().tabs.find(tab => tab.is_active)?.id, 'page');
});

test('failed durable preference write does not change the visible surface', async () => {
  const f = fixture(); f.fail('save');
  await assert.rejects(f.controller.change('tab'), /save settings/);
  assert.deepEqual(f.calls, ['save:tab']);
  assert.equal(f.get().mode, 'popup'); assert.equal(f.get().popup, true);
});

test('failed popup transfer restores the settings tab and tab preference', async () => {
  const f = fixture();
  await f.controller.change('tab');
  f.fail('open-popup');
  await assert.rejects(f.controller.change('popup'), /popup failed/);
  assert.equal(f.get().mode, 'tab'); assert.equal(f.get().saved, 'tab');
  assert.equal(f.get().tabs.find(tab => tab.is_active)?.url, 'spark://settings');
});
