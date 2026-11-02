import { execFileSync } from 'node:child_process';

const git = (...args) => execFileSync('git', args, { encoding: 'utf8' });
const sensitive = /(?:^|\/)(?:\.env(?:\..*)?|secrets|credentials|private)(?:\/|$)|\.(?:pem|key|p12|pfx|db|sqlite3?)(?:-(?:wal|shm|journal))?$|\.veloce-backup\.json(?:\..*)?$/i;
const secret = /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{50,}/;
const files = git('ls-files', '-z').split('\0').filter(Boolean);
const badPaths = files.filter(path => sensitive.test(path) && !path.endsWith('.env.example'));
if (badPaths.length) throw new Error(`Sensitive tracked paths: ${badPaths.length}`);
for (const file of files) {
  // Inspect tracked content without printing matching credentials.
  if (secret.test(git('show', `:${file}`))) throw new Error(`Potential credential found in tracked content: ${file}`);
}
for (const path of ['.env', '.env.local', 'test.pem', 'test.key', 'local.db', 'local.db-wal', 'local.sqlite-shm', 'local.sqlite3-journal', 'export.veloce-backup.json', 'export.veloce-backup.json.tmp', '.veloce-backup-example.tmp']) {
  execFileSync('git', ['check-ignore', '--quiet', '--no-index', '--', path]);
}
git('diff', '--check');
git('diff', '--cached', '--check');
console.log(git('status', '--short').trim() || 'Working tree clean');
console.log(git('diff', '--stat').trim());
console.log(git('diff', '--cached', '--stat').trim());
console.log('Sensitive paths, tracked credential patterns, diffs and ignore rules checked.');
