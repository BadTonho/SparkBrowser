import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runValidation } from '../testing/validate.mjs';
import { parseTag, synchronizeVersion } from '../release/shared.mjs';

export function validatePush(input, { cwd = process.cwd(), env = process.env, validate = runValidation } = {}) {
  const git = (...args) => execFileSync('git', args, { cwd, env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  const updates = input.trim() ? input.trim().split(/\r?\n/).map(line => {
    const fields = line.trim().split(/\s+/);
    if (fields.length !== 4 || ![fields[1], fields[3]].every(oid => /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(oid))) throw new Error('Invalid pre-push reference input.');
    return { local: fields[0], oid: fields[1], remote: fields[2] };
  }).filter(ref => !/^0+$/.test(ref.oid)) : [];
  if (!updates.length) return;
  const head = git('rev-parse', 'HEAD');
  for (const ref of updates) {
    if (git('rev-parse', `${ref.oid}^{commit}`) !== head) throw new Error(`Push blocked: ${ref.local} does not point to HEAD. Check out that commit and validate it before pushing.`);
  }
  const releases = updates.filter(ref => ref.remote.startsWith('refs/tags/v'));
  if (releases.length) {
    if (git('status', '--porcelain')) throw new Error('Release push blocked: commit or remove pending changes before validating the tagged commit.');
    for (const ref of releases) {
      const tag = ref.remote.slice('refs/tags/'.length);
      parseTag(tag);
      synchronizeVersion(cwd, tag, false);
    }
    validate({ native: 'critical' });
    if (git('rev-parse', 'HEAD') !== head || git('status', '--porcelain')) throw new Error('Release push blocked: validation changed the checkout. Review changes and retry.');
  } else {
    validate({ quick: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { validatePush(readFileSync(0, 'utf8')); }
  catch (error) { console.error(`Pre-push: ${error.message}`); process.exitCode = 1; }
}
