import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { writeFileSync, mkdirSync } from 'node:fs';

// Separate build output and application identifier prevent tests from touching a personal profile.
const result = spawnSync(process.execPath, [
  resolve('node_modules/@tauri-apps/cli/tauri.js'), 'build', '--no-bundle', '--config', 'tests/native/tauri.e2e.conf.json',
], { stdio: 'inherit', env: { ...process.env, CARGO_TARGET_DIR: resolve('.cache/native-target') } });
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
mkdirSync('.cache/native-target', { recursive: true });
writeFileSync('.cache/native-target/test-build.json', JSON.stringify({ identifier: 'org.veloce.browser.native-tests', executable: resolve('.cache/native-target/release', process.platform === 'win32' ? 'veloce-browser.exe' : 'veloce-browser') }));
