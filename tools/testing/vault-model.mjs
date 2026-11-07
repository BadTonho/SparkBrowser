// Test-only candidate. No application bridge, filesystem access, or user data inputs.
import { createCipheriv, createDecipheriv, randomBytes, scryptSync } from 'node:crypto';
const MASTER = 'fixture master password';
const RECOVERY = 'fixture recovery key';
const OS_KEY = 'fixture operating system key';
const ORIGIN = 'https://fixture.example';
const RECORD = { username: 'fixture-user', password: 'fixture-password' };
const AAD = Buffer.from('veloce-test-vault-v1');
function derive(value, salt) {
  return scryptSync(value, Buffer.from(salt, 'base64'), 32, { N: 16384, r: 8, p: 1 });
}
function seal(key, data) {
  const iv = randomBytes(12);
  const cipher = createCipheriv('aes-256-gcm', key, iv);
  cipher.setAAD(AAD);
  const ciphertext = Buffer.concat([cipher.update(data), cipher.final()]);
  return { iv: iv.toString('base64'), ciphertext: ciphertext.toString('base64'), tag: cipher.getAuthTag().toString('base64') };
}
function open(key, box) {
  const cipher = createDecipheriv('aes-256-gcm', key, Buffer.from(box.iv, 'base64'));
  cipher.setAAD(AAD);
  cipher.setAuthTag(Buffer.from(box.tag, 'base64'));
  return Buffer.concat([cipher.update(Buffer.from(box.ciphertext, 'base64')), cipher.final()]);
}
function wrap(key, value) {
  const salt = randomBytes(16).toString('base64');
  const wrapping = derive(value, salt);
  try { return { salt, box: seal(wrapping, key) }; } finally { wrapping.fill(0); }
}
function unwrap(wrapper, value) {
  const wrapping = derive(value, wrapper.salt);
  try { return open(wrapping, wrapper.box); } finally { wrapping.fill(0); }
}
/** Construct an isolated fixture vault. Adapters are in-memory test doubles. */
export function createFixtureVault(storage, os) {
  let key = null;
  let selection = null;
  let generation = 0;
  let origin = ORIGIN;
  function lock() { key?.fill(0); key = null; selection = null; }
  function read() {
    const value = storage.read();
    if (!value || value.version !== 1) throw new Error('fixture storage unavailable');
    return value;
  }
  return {
    initialize() {
      const candidate = randomBytes(32);
      try {
        const data = { version: 1, master: wrap(candidate, MASTER), recovery: wrap(candidate, RECOVERY), payload: seal(candidate, Buffer.from(JSON.stringify({ origin: ORIGIN, ...RECORD }))) };
        storage.write(data);
      } finally { candidate.fill(0); }
    },
    unlock(method = 'master', valid = true) {
      lock();
      try {
        const data = read();
        const wrapper = method === 'master' ? data.master : method === 'recovery' ? data.recovery : method === 'os' ? data.os : null;
        const material = method === 'master' ? MASTER : method === 'recovery' ? RECOVERY : os.read();
        if (!wrapper || !material) throw new Error('unavailable');
        key = unwrap(wrapper, valid ? material : 'incorrect fixture');
        const plaintext = open(key, data.payload);
        plaintext.fill(0);
        return true;
      } catch { lock(); return false; }
    },
    enableOs() {
      if (!key) return false;
      try {
        const data = read();
        const wrapped = wrap(key, OS_KEY);
        os.write(OS_KEY);
        storage.write({ ...data, os: wrapped });
        return true;
      } catch { return false; }
    },
    lock,
    navigate(fixture) {
      if (fixture !== 'same' && fixture !== 'other') throw new Error('unknown fixture');
      origin = fixture === 'same' ? ORIGIN : 'https://other.example';
      generation++;
      selection = null;
    },
    select() {
      if (!key || origin !== ORIGIN) return false;
      selection = generation;
      return true;
    },
    fill() {
      if (!key || origin !== ORIGIN || selection !== generation) return false;
      selection = null;
      try {
        const plaintext = open(key, read().payload);
        try {
          const record = JSON.parse(plaintext.toString());
          return record.origin === origin && record.username === RECORD.username && record.password === RECORD.password;
        } finally { plaintext.fill(0); }
      } catch { return false; }
    },
    isUnlocked() { return key !== null; },
  };
}
