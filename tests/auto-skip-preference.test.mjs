import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createAutoSkipPreference, readAutoSkip, AUTO_SKIP_STORAGE_KEY } from '../src/lib/services/auto-skip-preference.js';
test('auto skip defaults off and only the saved literal true enables it', () => {
  for (const saved of [null, 'false', 'yes', '1', '{}', 'TRUE']) assert.equal(readAutoSkip({ getItem: () => saved }), false);
  assert.equal(readAutoSkip({ getItem: () => 'true' }), true);
  assert.equal(readAutoSkip({ getItem: () => { throw Error(); } }), false);
});
test('startup and edits serialize; storage failure rolls native state back and can retry', async () => {
  let saved = 'true', fail = false, native = false;
  const calls = [], states = [];
  const preference = createAutoSkipPreference({ storage: { getItem: () => saved, setItem: (key, value) => {
    assert.equal(key, AUTO_SKIP_STORAGE_KEY); if (fail) return false; saved = value; return true;
  } }, apply: async value => { native = value; calls.push(value); return value; }, changed: (value, busy) => states.push({ value, busy }) });
  await Promise.all([preference.initialize(), preference.set(false)]);
  assert.deepEqual(calls, [true, false]); assert.equal(saved, 'false');
  fail = true; await assert.rejects(preference.set(true), /save auto skip/);
  assert.equal(native, false); assert.deepEqual(states.at(-1), { value: false, busy: false });
  fail = false; await preference.set(true); assert.equal(native, true); assert.equal(saved, 'true');
});
test('native failures preserve the stored choice and later changes recover', async () => {
  let fail = true, saved = null;
  const p = createAutoSkipPreference({ storage: { getItem: () => saved, setItem: (_, value) => { saved = value; } },
    apply: async value => { if (fail) throw Error('native'); return value; }, changed() {} });
  await assert.rejects(p.initialize(), /native/); assert.equal(saved, null);
  fail = false; await p.initialize(); await p.set(true); assert.equal(saved, 'true');
  await assert.rejects(p.set('false'), /boolean/);
});

test('late initialization reads the choice saved by an earlier queued user edit', async () => {
  let saved = null, native = false;
  const p = createAutoSkipPreference({ storage: { getItem: () => saved, setItem: (_, value) => { saved = value; } },
    apply: async value => { native = value; return value; }, changed() {} });
  await Promise.all([p.set(true), p.initialize()]);
  assert.equal(saved, 'true'); assert.equal(native, true);
});
