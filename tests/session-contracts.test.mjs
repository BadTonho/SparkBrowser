import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createSharedLifecycle } from '../src/lib/services/lifecycle.js';
import { mergeSessionObservations, mergeNavigationObservation } from '../src/lib/services/session-reconciliation.js';
import { createSurfaceCoordinator } from '../src/lib/services/surface-coordinator.js';

test('shared initialization owns one subscription and ignores callbacks after disposal', async () => {
  let starts = 0, stops = 0, calls = 0, resolve;
  let callback;
  const pending = new Promise(done => { resolve = done; });
  const initialize = createSharedLifecycle(scope => {
    starts++;
    callback = scope.guard(() => calls++);
    void pending.then(scope.own);
  });
  const a = initialize(), b = initialize();
  callback(); a(); a(); callback();
  assert.equal(starts, 1);
  assert.equal(calls, 2);
  b(); callback();
  resolve(() => stops++);
  await pending; await Promise.resolve();
  assert.equal(stops, 1);
  assert.equal(calls, 2);
  const c = initialize();
  assert.equal(starts, 2);
  c();
});

test('late snapshots preserve navigation without resurrecting removed tabs or losing identity', () => {
  const previous = [{ id: 'a', url: 'new', title: 'new', navigation: { revision: 9 }, activity: { playing_media: true } }, { id: 'closed' }];
  const incoming = [{ id: 'a', url: 'old', title: 'old', navigation: { revision: 8 }, container_id: 'account', workspace_id: 'work', is_ephemeral: true }];
  const [tab] = mergeSessionObservations(previous, incoming);
  assert.equal(tab.url, 'new');
  assert.equal(tab.container_id, 'account');
  assert.equal(tab.workspace_id, 'work');
  assert.equal(tab.is_ephemeral, true);
  assert.equal(tab.activity.playing_media, true);
  assert.equal(incoming[0].url, 'old');
  assert.equal(mergeSessionObservations(previous, incoming).length, 1);
});

test('navigation cannot change workspace, selection, identity, split residency or resurrect a tab', () => {
  const original = { id: 'a', url: 'old', title: 'old', workspace_id: 'work', container_id: 'account', is_active: false, is_hibernated: true, navigation: { revision: 2 } };
  const patch = { id: 'a', url: 'new', title: 'new', workspace_id: 'wrong', container_id: null, is_active: true, is_hibernated: false, navigation: { revision: 3 } };
  const [tab] = mergeNavigationObservation([original], patch);
  assert.deepEqual(tab, { ...original, url: 'new', title: 'new', navigation: { revision: 3 } });
  assert.equal(mergeNavigationObservation([tab], { ...patch, navigation: { revision: 1 } })[0], tab);
  assert.deepEqual(mergeNavigationObservation([], patch), []);
});

test('a delayed surface opening cannot overtake a newer request or restore its focus', async () => {
  const coordinator = createSurfaceCoordinator();
  let release;
  const slow = new Promise(resolve => { release = resolve; });
  coordinator.register('media', () => slow);
  coordinator.register('privacy', async () => {});
  const earlier = coordinator.claim('zoom');
  const latest = await coordinator.claim('media');
  release();
  assert.equal((await earlier)(), false);
  assert.equal(latest(), true);
});

test('one failed cleanup does not leave other domain resources subscribed', () => {
  let cleaned = 0;
  const initialize = createSharedLifecycle(scope => {
    scope.own(() => cleaned++);
    scope.own(() => { throw new Error('injected cleanup failure'); });
  });
  assert.throws(initialize(), AggregateError);
  assert.equal(cleaned, 1);
});
