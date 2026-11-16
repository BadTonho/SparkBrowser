import { spawnSync } from 'node:child_process';
import { existsSync, realpathSync } from 'node:fs';
import { delimiter, dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

/** Local checks must succeed before release packaging or access to signing material. */
export function runValidation({ native = false, quick = false } = {}, { run = spawnSync, log = console.log } = {}) {
  // Git hooks do not have npm_execpath. Unix distributions may put npm outside
  // Node's directory; follow the npm symlink on PATH in that case.
  const directories = [dirname(process.execPath), ...(process.env.PATH ?? '').split(delimiter).filter(Boolean)];
  const npm = process.env.npm_execpath ?? directories.flatMap(directory => [
    resolve(directory, 'node_modules/npm/bin/npm-cli.js'),
    ...(process.platform === 'win32' ? [] : [resolve(directory, 'npm')]),
  ]).filter(existsSync).map(path => realpathSync(path))[0];
  if (!npm) throw new Error('npm CLI not found. Install npm and make it available on PATH before validating.');
  const steps = [
    [process.execPath, [npm, 'test']],
    [process.execPath, [npm, 'run', 'check']],
    ...(quick ? [] : [
      ['cargo', ['fmt', '--manifest-path', 'src-tauri/Cargo.toml', '--check']],
      ['cargo', ['clippy', '--manifest-path', 'src-tauri/Cargo.toml', '--offline', '--all-targets', '--', '-D', 'warnings']],
      ['cargo', ['test', '--manifest-path', 'src-tauri/Cargo.toml', '--offline']],
      [process.execPath, [npm, 'run', 'build']],
      [process.execPath, ['tools/security/check-repo.mjs']],
    ]),
    ...(native ? [
      [process.execPath, ['tools/testing/build-native.mjs']],
      [process.execPath, ['tools/testing/native-smoke.mjs', ...(native === 'critical' ? ['--suite', 'critical'] : [])]],
    ] : []),
  ];
  for (const [binary, args] of steps) {
    log(`Validation: ${binary} ${args.join(' ')}`);
    const result = run(binary, args, { stdio: 'inherit', windowsHide: true });
    if (result.error || result.status !== 0) throw new Error(`Validation failed: ${binary} ${args.join(' ')}.`, { cause: result.error });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  if (args.length > 1 || args.some(arg => !['--native', '--critical', '--quick'].includes(arg))) throw new Error('Usage: npm run validate -- [--native | --critical | --quick]');
  runValidation({ native: args.includes('--critical') ? 'critical' : args.includes('--native'), quick: args.includes('--quick') });
}
