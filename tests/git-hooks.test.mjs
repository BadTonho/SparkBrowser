import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { installHooks } from '../tools/git/install-hooks.mjs';
import { validatePush as runPush } from '../tools/git/pre-push.mjs';
import { runValidation } from '../tools/testing/validate.mjs';

// Git exports repository-specific variables while executing a hook. Temporary
// repositories must not inherit those variables when this suite runs on push.
const isolatedGitEnv = () => Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith('GIT_')));
const validatePush = (input, options) => runPush(input, { ...options, env: isolatedGitEnv() });

function fixture(t) {
  const cwd = mkdtempSync(join(tmpdir(), 'spark hook tests '));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  const env = isolatedGitEnv();
  env.GIT_CONFIG_NOSYSTEM = '1';
  env.GIT_CONFIG_GLOBAL = process.platform === 'win32' ? 'NUL' : '/dev/null';
  const git = (...args) => execFileSync('git', args, { cwd, env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  git('init'); git('config', 'user.name', 'Hook Test'); git('config', 'user.email', 'hook@example.test');
  mkdirSync(join(cwd, 'src-tauri'));
  for (const [path, value] of [
    ['package.json', '{"version":"0.1.1"}'],
    ['package-lock.json', '{"version":"0.1.1","packages":{"":{"version":"0.1.1"}}}'],
    ['src-tauri/tauri.conf.json', '{"version":"0.1.1"}'],
    ['src-tauri/Cargo.toml', '[package]\nname = "spark-browser"\nversion = "0.1.1"\n'],
    ['src-tauri/Cargo.lock', '[[package]]\nname = "spark-browser"\nversion = "0.1.1"\n'],
  ]) writeFileSync(join(cwd, path), value);
  git('add', '.'); git('commit', '-m', 'fixture');
  return { cwd, env, git, head: git('rev-parse', 'HEAD') };
}
const zero = '0'.repeat(40);
const ref = (oid, name = 'refs/heads/main') => `${name} ${oid} ${name} ${zero}\n`;

test('installer is idempotent, handles spaces, preserves custom hooks and skips CI', t => {
  const { cwd, env, git } = fixture(t);
  const options = { cwd, env: { ...env, CI: 'false', GITHUB_ACTIONS: 'false' }, log() {} };
  assert.equal(installHooks({ ...options, env: { ...env, CI: 'true' } }), 'ci');
  const target = join(cwd, '.git/hooks/pre-push');
  assert.equal(existsSync(target), false);
  assert.equal(installHooks(options), 'installed');
  const contents = readFileSync(target, 'utf8');
  assert.ok(!contents.includes('\r'));
  assert.equal(installHooks(options), 'installed');
  assert.equal(readFileSync(target, 'utf8'), contents);
  writeFileSync(target, '#!/bin/sh\nexit 42\n');
  assert.equal(installHooks(options), 'existing');
  assert.equal(readFileSync(target, 'utf8'), '#!/bin/sh\nexit 42\n');
  git('config', 'core.hooksPath', 'custom hooks');
  assert.equal(installHooks(options), 'custom');
  assert.equal(git('config', '--get', 'core.hooksPath'), 'custom hooks');
  git('config', 'core.hooksPath', '');
  assert.equal(installHooks(options), 'custom');
  assert.equal(existsSync(join(cwd, 'pre-push')), false);
});

test('installer skips source exports without Git metadata', t => {
  const cwd = mkdtempSync(join(tmpdir(), 'spark no git '));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  assert.equal(installHooks({ cwd, env: { ...isolatedGitEnv(), CI: 'false', GITHUB_ACTIONS: 'false' }, log() {} }), 'no-git');
});

test('ordinary pushes validate once; deletions and no-op pushes do not validate', t => {
  const { cwd, head } = fixture(t);
  const calls = [];
  const options = { cwd, validate: value => calls.push(value) };
  validatePush(ref(head) + ref(head, 'refs/heads/other'), options);
  assert.deepEqual(calls, [{ quick: true }]);
  validatePush('', options);
  validatePush(ref(zero, 'refs/tags/v0.1.0'), options);
  assert.equal(calls.length, 1);
  assert.throws(() => validatePush('malformed', options), /Invalid/);
});

test('release pushes require matching versions, clean HEAD and fresh critical validation', t => {
  const { cwd, head, git } = fixture(t);
  const calls = [];
  const options = { cwd, validate: value => calls.push(value) };
  git('tag', '-a', 'v0.1.1', '-m', 'fixture tag');
  validatePush(ref(head) + ref(git('rev-parse', 'v0.1.1'), 'refs/tags/v0.1.1'), options);
  assert.deepEqual(calls, [{ native: 'critical' }]);
  assert.throws(() => validatePush(ref(head, 'refs/tags/v9.0.0'), options), /do not match/);
  writeFileSync(join(cwd, 'pending.txt'), 'dirty');
  assert.throws(() => validatePush(ref(head, 'refs/tags/v0.1.1'), options), /pending changes/);
  assert.equal(calls.length, 1);
});

test('old commits and errors from any validation step block pushes', t => {
  const { cwd, head, git } = fixture(t);
  writeFileSync(join(cwd, 'new.txt'), 'next'); git('add', '.'); git('commit', '-m', 'next');
  assert.throws(() => validatePush(ref(head), { cwd, validate() { assert.fail('must not run'); } }), /does not point to HEAD/);
  const current = git('rev-parse', 'HEAD');
  for (const name of ['JavaScript', 'types', 'format', 'clippy', 'Rust', 'build', 'security', 'native build', 'native scenario']) {
    assert.throws(() => validatePush(ref(current, 'refs/tags/v0.1.1'), { cwd, validate() { throw new Error(name); } }), new RegExp(name));
  }
  assert.throws(() => validatePush(ref(current, 'refs/tags/v0.1.1'), { cwd, validate() { writeFileSync(join(cwd, 'changed.txt'), 'unexpected'); } }), /changed the checkout/);
});

test('validation stops at every failed command and release builds precede native scenarios', () => {
  const steps = [];
  runValidation({ native: 'critical' }, { log() {}, run(binary, args) { steps.push([binary, args]); return { status: 0 }; } });
  assert.equal(steps.length, 9);
  assert.ok(steps.at(-2)[1].includes('tools/testing/build-native.mjs'));
  assert.deepEqual(steps.at(-1)[1], ['tools/testing/native-smoke.mjs', '--suite', 'critical']);
  for (let failure = 0; failure < steps.length; failure++) {
    let executed = 0;
    assert.throws(() => runValidation({ native: 'critical' }, { log() {}, run() { return { status: executed++ === failure ? 1 : 0 }; } }), /Validation failed/);
    assert.equal(executed, failure + 1);
  }
  let quick = 0;
  runValidation({ quick: true }, { log() {}, run() { quick++; return { status: 0 }; } });
  assert.equal(quick, 2);
});

test('Git executes the hook in a spaced path, blocks failure and permits a validated local push', t => {
  const { cwd, env, git } = fixture(t);
  mkdirSync(join(cwd, 'tools/git'), { recursive: true });
  mkdirSync(join(cwd, 'tools/testing'), { recursive: true });
  mkdirSync(join(cwd, 'tools/release'), { recursive: true });
  for (const file of ['tools/git/pre-push.mjs', 'tools/release/shared.mjs']) {
    writeFileSync(join(cwd, file), readFileSync(new URL(`../${file}`, import.meta.url)));
  }
  writeFileSync(join(cwd, 'tools/testing/validate.mjs'), 'export function runValidation() { throw new Error("fixture validation failed"); }');
  git('add', '.'); git('commit', '-m', 'hook runtime');
  const remote = join(cwd, 'local remote.git');
  git('init', '--bare', remote);
  git('remote', 'add', 'fixture', remote);
  assert.equal(installHooks({ cwd, env: { ...env, CI: 'false', GITHUB_ACTIONS: 'false' }, log() {} }), 'installed');
  assert.throws(() => git('push', 'fixture', 'HEAD:refs/heads/main'), error => {
    assert.match(error.stderr, /fixture validation failed/);
    return true;
  });
  assert.equal(git('ls-remote', 'fixture'), '');
  writeFileSync(join(cwd, 'tools/testing/validate.mjs'), 'export function runValidation() {}');
  git('add', '.'); git('commit', '-m', 'passing validator');
  git('push', 'fixture', 'HEAD:refs/heads/main');
  assert.match(git('ls-remote', 'fixture'), new RegExp(`${git('rev-parse', 'HEAD')}\\s+refs/heads/main`));
});
