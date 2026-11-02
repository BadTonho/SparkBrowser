import assert from 'node:assert/strict';
import { mkdirSync, realpathSync } from 'node:fs';
import { rm } from 'node:fs/promises';
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
  const results = await Promise.allSettled(['data', 'profile', 'config', 'cache', 'logs'].map(async child => {
    const path = resolve(target, child);
    assert.equal(relative(target, path), child);
    await rm(path, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
  }));
  const failures = results.filter(result => result.status === 'rejected').map(result => result.reason);
  if (failures.length) throw new AggregateError(failures, failures.map(String).join('; '));
}
