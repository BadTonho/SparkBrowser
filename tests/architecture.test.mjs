import { test } from 'node:test';
import assert from 'node:assert/strict';
import { findCycles, checkArchitecture } from '../tools/architecture/check.mjs';
import { criticalScenarios, selectScenarios } from './native/suites.mjs';
import { scenarios } from './native/scenarios.mjs';

test('architecture rejects cycles while allowing a shared dependency', () => {
  assert.equal(findCycles(new Map([['a', ['b', 'c']], ['b', ['c']], ['c', []]])).length, 0);
  assert.deepEqual(findCycles(new Map([['a', ['b']], ['b', ['a']]])), [['a', 'b', 'a']]);
});
test('application respects state and lifecycle ownership', checkArchitecture);
test('the mandatory native suite resolves real scenarios and rejects unknown selection', () => {
  const names = selectScenarios(['--suite', 'critical'], scenarios);
  assert.deepEqual(names, criticalScenarios);
  for (const name of names) assert.equal(typeof scenarios[name], 'function', name);
  assert.equal(new Set(names).size, 7);
  assert.throws(() => selectScenarios(['--suite', 'typo'], scenarios));
  assert.throws(() => selectScenarios(['--case', 'constructor'], scenarios));
});
