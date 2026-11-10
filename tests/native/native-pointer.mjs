import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { validateRunDirectory } from './run-directory.mjs';

export function createNativePointerSender(directory) {
  const build = JSON.parse(readFileSync('.cache/native-target/test-build.json', 'utf8'));
  assert.equal(build.identifier, 'org.veloce.browser.native-tests');
  const actionsFile = resolve(validateRunDirectory(directory), 'artifacts/pointer-actions.json');
  return actions => new Promise((done, reject) => {
    writeFileSync(actionsFile, JSON.stringify(actions));
    const child = spawn('powershell.exe', ['-NoProfile', '-File', resolve('tests/native/windows-pointer.ps1'), '-Executable', resolve(build.executable), '-ActionsFile', actionsFile], { windowsHide: true, stdio: ['ignore', 'ignore', 'pipe'] });
    let error = '';
    child.stderr.on('data', data => { error += data; });
    child.on('error', reject);
    child.on('exit', code => code === 0 ? done() : reject(new Error(error || 'Native pointer operation failed')));
  });
}
