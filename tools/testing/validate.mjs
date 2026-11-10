import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

/** Local checks must succeed before release packaging or access to signing material. */
export function runValidation({ native = false } = {}) {
  const npm = process.env.npm_execpath ?? resolve(dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js');
  const steps = [
    [process.execPath, ['tools/architecture/check.mjs']],
    [process.execPath, [npm, 'test']],
    [process.execPath, [npm, 'run', 'check']],
    ['cargo', ['fmt', '--manifest-path', 'src-tauri/Cargo.toml', '--check']],
    ['cargo', ['clippy', '--manifest-path', 'src-tauri/Cargo.toml', '--offline', '--all-targets', '--', '-D', 'warnings']],
    ['cargo', ['test', '--manifest-path', 'src-tauri/Cargo.toml', '--offline']],
    [process.execPath, [npm, 'run', 'build']],
    [process.execPath, ['tools/security/check-repo.mjs']],
    ...(native ? [
      [process.execPath, ['tools/testing/build-native.mjs']],
      [process.execPath, ['tools/testing/native-smoke.mjs']],
    ] : []),
  ];
  for (const [binary, args] of steps) {
    console.log(`Validation: ${binary} ${args.join(' ')}`);
    const result = spawnSync(binary, args, { stdio: 'inherit', windowsHide: true });
    if (result.error || result.status !== 0) throw new Error(`Validation failed: ${binary} ${args.join(' ')}. Packaging stopped.`, { cause: result.error });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.slice(2).some(arg => arg !== '--native')) throw new Error('Usage: npm run validate -- [--native]');
  runValidation({ native: process.argv.includes('--native') });
}
