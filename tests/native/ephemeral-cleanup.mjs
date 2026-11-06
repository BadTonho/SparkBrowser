import assert from 'node:assert/strict';
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

export async function checkEphemeralCleanup({ invoke, until, fixture, restart, directory }) {
  const root = join(directory, 'profile', 'webview-ephemeral');
  const tabPath = tab => join(root, tab.id);
  const create = async () => {
    const tab = (await invoke('create_ephemeral_tab', { url: `${fixture.origin}/page/temporary` })).tabs.find(item => item.is_active);
    await until(() => existsSync(tabPath(tab)), 'temporary WebView data created');
    await until(async () => !(await invoke('get_tabs')).tabs.find(item => item.id === tab.id)?.navigation.loading, 'temporary page loaded');
    return tab;
  };
  const first = await create();
  await invoke('close_tab', { tabId: first.id });
  await until(() => !existsSync(tabPath(first)), 'closed temporary tab data removed');

  const second = await create();
  await invoke('create_tab', { url: `${fixture.origin}/page/normal` });
  await invoke('hibernate_tab', { tabId: second.id });
  assert.ok(existsSync(tabPath(second)), 'hibernation preserves temporary data');
  const result = await invoke('close_tabs_reviewed', { tabIds: [second.id] });
  assert.ok(result.closed.includes(second.id));
  await until(() => !existsSync(tabPath(second)), 'bulk-closed temporary data removed');

  await restart({ beforeStart: run => {
    const leftover = join(run, 'profile', 'webview-ephemeral', 'old-tab');
    mkdirSync(leftover, { recursive: true });
    writeFileSync(join(leftover, 'fixture'), 'temporary');
  } });
  assert.ok(!existsSync(join(root, 'old-tab')), 'startup removes old temporary data before accepting tabs');
}
