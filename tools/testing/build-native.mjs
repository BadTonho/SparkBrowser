import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { writeFileSync, mkdirSync } from 'node:fs';

const args = process.argv.slice(2);
if (args.some(arg => arg !== '--fast') || args.length > 1) throw new Error('Usage: build-native.mjs [--fast]');
const fast = args.includes('--fast');
// Separate build output and application identifier prevent tests from touching a personal profile.
const result = spawnSync(process.execPath, [
  resolve('node_modules/@tauri-apps/cli/tauri.js'), 'build', '--no-bundle', '--config', 'tests/native/tauri.e2e.conf.json',
], { stdio: 'inherit', env: { ...process.env, CARGO_TARGET_DIR: resolve('.cache/native-target'),
  ...(fast ? { CARGO_PROFILE_RELEASE_LTO: 'false', CARGO_PROFILE_RELEASE_CODEGEN_UNITS: '16', CARGO_PROFILE_RELEASE_OPT_LEVEL: '1' } : {}) } });
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
mkdirSync('.cache/native-target', { recursive: true });
writeFileSync('.cache/native-target/test-build.json', JSON.stringify({ identifier: 'org.spark.browser.native-tests', profile: fast ? 'fast' : 'release', executable: resolve('.cache/native-target/release', process.platform === 'win32' ? 'spark-browser.exe' : 'spark-browser') }));
