import assert from 'node:assert/strict';
import { mkdirSync, realpathSync, rmSync } from 'node:fs';
import { isAbsolute, relative, resolve } from 'node:path';

export const runsRoot = resolve('.cache/native-runs');
export function createRunDirectory(name) {
  assert.match(name, /^[a-z][a-z0-9-]*$/);
  mkdirSync(runsRoot, { recursive: true });
  const directory = resolve(runsRoot, `${Date.now()}-${process.pid}-${name}`);
  for (const child of ['data', 'profile', 'config', 'cache', 'logs', 'artifacts']) mkdirSync(resolve(directory, child), { recursive: true });
  return directory;
}
export function validateRunDirectory(directory) {
  const root = realpathSync(runsRoot);
  const target = realpathSync(directory);
  const child = relative(root, target);
  assert.ok(child && !child.startsWith('..') && !isAbsolute(child), 'Cleanup must stay within .cache/native-runs');
  return target;
}
export async function cleanRunData(directory) {
  const target = validateRunDirectory(directory);
  // Reports and screenshots survive both successful and failed cases.
  for (const child of ['data', 'profile', 'config', 'cache', 'logs']) {
    const path = resolve(target, child);
    assert.equal(relative(target, path), child);
    rmSync(path, { recursive: true, force: true, maxRetries: 20, retryDelay: 200 });
  }
}
