import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { readFileSync } from 'node:fs';
import { validateRunDirectory } from './run-directory.mjs';

export function createNativeKeySender(directory) {
  const build = JSON.parse(readFileSync('.cache/native-target/test-build.json', 'utf8'));
  assert.equal(build.identifier, 'org.spark.browser.native-tests');
  const executable = resolve(build.executable);
  assert.ok(executable.startsWith(resolve('.cache/native-target') + '\\'));
  const runDirectory = directory ? validateRunDirectory(directory) : undefined;
  const send = (code, up = false, prompt = false, stroke = false) => new Promise((resolveKey, reject) => {
    if (prompt) assert.ok(runDirectory, 'Native prompts must belong to a verified scenario');
    const child = spawn('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve('tests/native/windows-input.ps1'), '-Executable', executable, '-Key', String(code), ...(up ? ['-Up'] : []), ...(stroke ? ['-Stroke'] : []), ...(prompt ? ['-Prompt', '-RunDirectory', runDirectory] : [])], { windowsHide: true, stdio: ['ignore', 'ignore', 'pipe'] });
    let error = '';
    child.stderr.on('data', data => { error += data; });
    child.on('error', reject);
    child.on('exit', code => code === 0 ? resolveKey() : reject(new Error(error || 'Native key helper failed')));
  });
  send.press = (code, prompt = false) => send(code, false, prompt, true);
  return send;
}
