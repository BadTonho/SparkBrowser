import { readdirSync, readFileSync, statSync } from 'node:fs';
import { gzipSync } from 'node:zlib';
import { execFileSync } from 'node:child_process';
import { join } from 'node:path';

const command = (binary, args) => execFileSync(binary, args, { encoding: 'utf8' }).trim();
const files = readdirSync('dist/assets').filter(file => /\.(js|css)$/.test(file)).map(file => {
  const content = readFileSync(join('dist/assets', file));
  return { file, bytes: content.length, gzip_bytes: gzipSync(content).length };
});
const binary = join('src-tauri/target/release', process.platform === 'win32' ? 'spark-browser.exe' : 'spark-browser');
console.log(JSON.stringify({
  commit: command('git', ['rev-parse', 'HEAD']),
  tracked_changes: Boolean(command('git', ['status', '--porcelain', '--untracked-files=no'])),
  node: process.version,
  rust: command('rustc', ['--version']),
  platform: process.platform,
  architecture: process.arch,
  frontend_assets: files,
  binary_bytes: statSync(binary).size,
  binary_modified_utc: statSync(binary).mtime.toISOString(),
  // Measurements intentionally do not label file size as a memory/startup benchmark.
}, null, 2));
