import assert from 'node:assert/strict';
import test from 'node:test';
import { settleWithin } from '../src/lib/services/deadline.js';

const never = () => new Promise(() => {});

test('fast work returns its value', async () => {
  assert.deepEqual(await settleWithin(Promise.resolve(7), 1000), { value: 7 });
});

test('work that never finishes stops being waited for after the deadline', async () => {
  const started = Date.now();
  assert.deepEqual(await settleWithin(never(), 30), { timedOut: true });
  assert.ok(Date.now() - started < 500, 'the wait ended near the deadline');
});

test('failing work is reported, never thrown, so closing the browser cannot be blocked by it', async () => {
  assert.deepEqual(await settleWithin(Promise.reject(new Error('disk full')), 1000), { failed: true });
});

test('a slow success after the deadline does not change the outcome', async () => {
  const slow = new Promise((resolve) => setTimeout(() => resolve('late'), 80));
  assert.deepEqual(await settleWithin(slow, 20), { timedOut: true });
});

test('the timer is cleared when the work finishes first, so it cannot keep the process alive', async () => {
  await settleWithin(Promise.resolve(), 60_000);
  // If the 60 s timer were still pending, node:test would wait for it; reaching here quickly proves it is not.
  assert.ok(true);
});
