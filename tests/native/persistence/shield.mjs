import assert from 'node:assert/strict';
import { withScenarioDatabase } from './database.mjs';

export async function checkShieldPersistence({ invoke, fixture, until, directory }) {
  if (process.platform !== 'win32') return;
  await invoke('navigate_to', { input: fixture.origin + '/page/shield-failure' });
  await until(() => fixture.reports.has('shield-failure'), 'shield fixture');
  const id = (await invoke('get_tabs')).tabs.find(t => t.is_active).id;
  fixture.commands.set('shield-failure', 'spa');
  await until(async () => (await invoke('get_tabs')).tabs.find(t => t.id === id).navigation.can_go_back, 'native history before shield change');
  for (const enabled of [true, false]) {
    await invoke('get_history', { limit: 1 });
    assert.equal((await invoke('get_privacy_stats')).shield_enabled, enabled);
    await withScenarioDatabase(directory, async db => {
      db.exec("CREATE TRIGGER fail_shield_session BEFORE INSERT ON session_tabs BEGIN SELECT RAISE(ABORT,'injected shield persistence failure'); END");
      try {
        await assert.rejects(invoke('toggle_adblock_shield'), /injected shield persistence failure/);
        assert.equal((await invoke('get_privacy_stats')).shield_enabled, enabled, 'failed toggle retains the old policy');
        assert.equal((await invoke('get_tabs')).tabs.find(t => t.id === id).navigation.can_go_back, true);
        assert.equal(fixture.loads.get('shield-failure'), 1);
      } finally { db.exec('DROP TRIGGER fail_shield_session'); }
    });
    // A late title/navigation callback can hit the same injected trigger. Flush
    // and acknowledge that expected worker error before testing the retry.
    await invoke('get_history', { limit: 1 }).catch(error => {
      assert.match(String(error), /injected shield persistence failure/);
    });
    await invoke('get_history', { limit: 1 });
    assert.equal((await invoke('toggle_adblock_shield')).shield_enabled, !enabled, 'retry applies the policy');
    assert.equal(fixture.loads.get('shield-failure'), 1);
  }
}
