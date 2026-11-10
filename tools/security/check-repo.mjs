import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';

const git = (...args) => execFileSync('git', args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
const sensitive = /(?:^|\/)(?:\.env(?:\..*)?|secrets|credentials|private|profiles|user-data|\.cache)(?:\/|$)|\.(?:pem|key|p12|pfx|cer|crt|jks|keystore|db|sqlite3?)(?:-(?:wal|shm|journal))?$|\.veloce-backup\.json(?:\..*)?$/i;
const secret = /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{50,}/;
const files = git('ls-files', '-z').split('\0').filter(Boolean);
const workingFiles = git('ls-files', '--cached', '--others', '--exclude-standard', '-z').split('\0').filter(Boolean);
const badPaths = workingFiles.filter(path => sensitive.test(path) && !path.endsWith('.env.example'));
if (badPaths.length) throw new Error(`Sensitive tracked paths: ${badPaths.length}`);
for (const file of files) {
  // Inspect tracked content without printing matching credentials.
  if (secret.test(git('show', `:${file}`))) throw new Error(`Potential credential found in tracked content: ${file}`);
}
for (const file of new Set(workingFiles)) {
  // Include unstaged changes and new modules: checking the index alone misses
  // credentials introduced in files that have not been added to Git yet.
  if (existsSync(file) && secret.test(readFileSync(file).toString('utf8'))) {
    throw new Error(`Potential credential found in working content: ${file}`);
  }
}
for (const path of ['.env', '.env.local', 'test.pem', 'test.key', 'test.p12', 'test.pfx', 'local.db', 'local.db-wal', 'local.sqlite-shm', 'local.sqlite3-journal', 'export.veloce-backup.json', 'export.veloce-backup.json.tmp', '.veloce-backup-example.tmp', '.cache/native-results.json', '.cache/native-runs/fixture/profile/Cookies', 'profiles/fixture/preferences.json', 'credentials/fixture.json']) {
  execFileSync('git', ['check-ignore', '--quiet', '--no-index', '--', path]);
}
git('diff', '--check');
git('diff', '--cached', '--check');
console.log(git('status', '--short').trim() || 'Working tree clean');
console.log(git('diff', '--stat').trim());
console.log(git('diff', '--cached', '--stat').trim());
console.log('Sensitive paths, tracked credential patterns, diffs and ignore rules checked.');
