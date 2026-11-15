import assert from 'node:assert/strict';
import { test } from 'node:test';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { cleanRunData, createRunDirectory, runsRoot, validateRunDirectory } from './native/run-directory.mjs';
import { nativeDriverPath } from './native/driver-path.mjs';

test('native driver selection uses an explicit path or the installed Windows fixture', () => {
  assert.equal(nativeDriverPath('win32', 'custom/driver.exe', () => false), resolve('custom/driver.exe'));
  assert.equal(nativeDriverPath('win32', '', () => true), resolve('.cache/native-tools/edge/msedgedriver.exe'));
  assert.equal(nativeDriverPath('win32', '', () => false), undefined);
  assert.equal(nativeDriverPath('linux', '', () => true), undefined);
});

test('native cleanup removes scenario data while retaining reports and rejects unsafe paths', async () => {
  const directory = createRunDirectory('cleanup-test');
  writeFileSync(resolve(directory, 'data', 'fixture.sqlite'), 'artificial');
  writeFileSync(resolve(directory, 'artifacts', 'report.json'), '{}');
  assert.throws(() => validateRunDirectory(runsRoot));
  assert.throws(() => validateRunDirectory(resolve('.')));
  await cleanRunData(directory);
  assert.equal(existsSync(resolve(directory, 'data')), false);
  assert.equal(readFileSync(resolve(directory, 'artifacts', 'report.json'), 'utf8'), '{}');
  await cleanRunData(directory); // Repeated cleanup is safe.
});
