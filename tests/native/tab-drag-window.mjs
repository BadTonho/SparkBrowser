import { execFile } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
export function inspectDragWindows() {
  const build = JSON.parse(readFileSync('.cache/native-target/test-build.json', 'utf8'));
  if (build.identifier !== 'org.spark.browser.native-tests') throw Error('Expected isolated build');
  return new Promise((done, reject) => execFile('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve('tests/native/tab-drag-window.ps1'), '-Executable', build.executable], { windowsHide: true }, (error, stdout) => {
    if (error) reject(error); else { try { done(JSON.parse(stdout)); } catch (error) { reject(error); } }
  }));
}
