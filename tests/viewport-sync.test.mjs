import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createViewportSync } from '../src/lib/services/viewport-sync.js';

test('flush waits for the final measurement before committing a split adjustment', async () => {
  let value = 1; let release; const sent = [];
  const sync = createViewportSync(() => value, async current => { sent.push(current); await new Promise(resolve => release = resolve); }, assert.fail, () => 1, () => {});
  const first = sync.flush(); value = 2; const last = sync.flush(); let finished = false;
  last.then(() => finished = true);
  release(); await new Promise(resolve => setImmediate(resolve));
  assert.deepEqual(sent, [1, 2]); assert.equal(finished, false);
  release(); await Promise.all([first, last]); assert.equal(finished, true); sync.dispose();
});

test('resize coalesces frames, deduplicates and retains the latest in-flight measurement', async () => {
  let callback;
  let value = 1;
  let finish;
  const sent = [];
  const sync = createViewportSync(() => value, async v => {
    sent.push(v);
    await new Promise(resolve => { finish = resolve; });
  }, assert.fail, cb => { callback = cb; return 1; }, () => {});
  const tick = () => { callback(0); };
  sync.schedule(); sync.schedule(); tick();
  value = 2; sync.schedule(); tick();
  value = 3; sync.schedule(); tick();
  assert.deepEqual(sent, [1]);
  finish(); await new Promise(resolve => setImmediate(resolve));
  assert.deepEqual(sent, [1, 3]);
  finish(); await new Promise(resolve => setImmediate(resolve));
  sync.schedule(); tick();
  assert.deepEqual(sent, [1, 3]);
  sync.dispose();
  value = 4; sync.schedule();
  assert.deepEqual(sent, [1, 3]);
});
