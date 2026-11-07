import test from 'node:test';
import assert from 'node:assert/strict';
import { createFixtureVault } from '../tools/testing/vault-model.mjs';
function setup() {
  let persisted;
  let osKey;
  let failStorage = false;
  let failOs = false;
  const storage = {
    read: () => structuredClone(persisted),
    write: value => { if (failStorage) throw Error('storage'); persisted = structuredClone(value); },
  };
  const os = {
    read: () => { if (failOs) throw Error('os'); return osKey; },
    write: value => { if (failOs) throw Error('os'); osKey = value; },
  };
  const vault = createFixtureVault(storage, os);
  return { vault, storage, os, data: () => persisted, failStorage: () => { failStorage = true; }, failOs: () => { failOs = true; } };
}
test('encrypted fixture survives reopening and requires explicit selection', () => {
  const f = setup();
  f.vault.initialize();
  assert.doesNotMatch(JSON.stringify(f.data()), /fixture-password|fixture-user|fixture master password/);
  assert.equal(f.vault.unlock('master', false), false);
  assert.equal(f.vault.unlock('master'), true);
  assert.equal(f.vault.fill(), false);
  assert.equal(f.vault.select(), true);
  assert.equal(f.vault.fill(), true);
  f.vault.lock();
  assert.equal(f.vault.fill(), false);
  const reopened = createFixtureVault(f.storage, f.os);
  assert.equal(reopened.unlock('recovery'), true);
  assert.equal(reopened.select(), true);
  assert.equal(reopened.fill(), true);
});
test('origin changes, navigation and corrupted ciphertext fail closed', () => {
  const f = setup();
  f.vault.initialize();
  assert.equal(f.vault.unlock(), true);
  f.vault.select();
  f.vault.navigate('other');
  assert.equal(f.vault.fill(), false);
  assert.equal(f.vault.select(), false);
  f.vault.navigate('same');
  assert.equal(f.vault.fill(), false);
  const damaged = structuredClone(f.data());
  damaged.payload.tag = Buffer.alloc(16).toString('base64');
  f.storage.write(damaged);
  f.vault.lock();
  assert.equal(f.vault.unlock(), false);
  assert.equal(f.vault.isUnlocked(), false);
});
test('simulated OS and persistence failures never enable unlock', () => {
  const f = setup();
  f.vault.initialize();
  assert.equal(f.vault.unlock(), true);
  assert.equal(f.vault.enableOs(), true);
  f.vault.lock();
  assert.equal(f.vault.unlock('os'), true);
  f.failOs();
  assert.equal(f.vault.unlock('os'), false);
  assert.equal(f.vault.isUnlocked(), false);
  const g = setup();
  g.failStorage();
  assert.throws(() => g.vault.initialize());
  assert.equal(g.vault.unlock(), false);
});
