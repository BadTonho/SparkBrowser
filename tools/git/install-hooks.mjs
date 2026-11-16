import { execFileSync } from 'node:child_process';
import { chmodSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const source = new URL('../../.githooks/pre-push', import.meta.url);
const marker = '# Spark managed pre-push hook v1';

export function installHooks({ cwd = process.cwd(), env = process.env, log = console.log } = {}) {
  if ((env.CI && env.CI !== 'false') || env.GITHUB_ACTIONS === 'true') return 'ci';
  const git = (...args) => execFileSync('git', args, { cwd, env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  try { git('rev-parse', '--git-dir'); } catch { return 'no-git'; }
  let custom;
  try { custom = git('config', '--get', 'core.hooksPath'); }
  catch (error) { if (error.status !== 1) throw error; }
  return installAtDefaultPath(custom);

  function installAtDefaultPath(customPath) {
    if (customPath !== undefined) {
      log('Custom core.hooksPath preserved. Integrate node tools/git/pre-push.mjs in your pre-push hook, forwarding stdin and its exit status. See docs/en/contributing/local-hooks.md.');
      return 'custom';
    }
    const target = resolve(cwd, git('rev-parse', '--git-path', 'hooks/pre-push'));
    if (existsSync(target) && !readFileSync(target, 'utf8').includes(marker)) {
      log('Existing pre-push hook preserved. Integrate node tools/git/pre-push.mjs, forwarding stdin and its exit status. See docs/en/contributing/local-hooks.md.');
      return 'existing';
    }
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, readFileSync(source, 'utf8').replaceAll('\r\n', '\n'));
    chmodSync(target, 0o755);
    log('Spark pre-push hook installed. Push checks run locally; GitHub CI remains manual.');
    return 'installed';
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) installHooks();
