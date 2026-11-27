import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const read = (path) => readFileSync(path, 'utf8').replace(/\r\n/g, '\n');
const COMMANDS = ['get_github_backup_state', 'connect_github_backup', 'set_github_backup_auto', 'disconnect_github_backup',
  'push_github_backup', 'review_github_backup', 'list_github_backup_versions', 'review_github_backup_version'];

/** `githubDue` and the interval, taken from the store without its imports. */
function loadDue() {
  const source = read('src/lib/stores/githubBackupStore.ts');
  const constant = source.match(/export const GITHUB_INTERVAL_MS = [^;]+;/)[0];
  const fn = source.match(/export function githubDue[\s\S]*?\n}\n/)[0];
  const js = ts.transpileModule(`${constant}\n${fn}`, { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText.replaceAll('export ', '');
  return new Function(`${js}; return { githubDue, GITHUB_INTERVAL_MS };`)();
}

test('the GitHub backup commands are registered end to end', () => {
  for (const command of COMMANDS) {
    assert.match(read('src-tauri/src/commands/manifest.rs'), new RegExp(`"${command}"`));
    assert.match(read('src-tauri/capabilities/default.json'), new RegExp(`"allow-${command.replaceAll('_', '-')}"`));
    assert.match(read('src-tauri/src/lib.rs'), new RegExp(`\\b${command},\\n`));
    assert.match(read('src/lib/services/bridge/github-backup.ts'), new RegExp(`"${command}"`));
  }
  assert.match(read('src/App.svelte'), /initializeGithubBackup\(\)/, 'the hourly backup runs even with Settings closed');
});

test('GitHub is its own card, next to the sync target, not one of its modes', () => {
  const page = read('src/lib/components/settings/BackupSettings.svelte');
  assert.doesNotMatch(page, /<option value="github">/);
  assert.match(page, /class="sync-slot secondary"/);
  assert.match(page, /<GitHubIcon size=\{30\} \/>/);
  assert.doesNotMatch(read('src-tauri/src/backup/sync.rs'), /GitHub/, 'the sync target is untouched');
});

test('an hourly send, counted from the last send on disk or the last attempt', () => {
  const { githubDue, GITHUB_INTERVAL_MS } = loadDue();
  const now = 1_800_000_000_000;
  const state = (extra = {}) => ({ target: { repository: 'a/b', branch: 'spark-sync', auto_save: true }, has_token: true, last_synced_at: null, last_error: null, ...extra });
  const due = (s, lastAttemptMs = 0, paused = false) => githubDue(s, { nowMs: now, lastAttemptMs, paused });
  assert.equal(GITHUB_INTERVAL_MS, 3_600_000);
  assert.equal(due(state()), true, 'never sent');
  assert.equal(due(state({ last_synced_at: (now - 59 * 60_000) / 1000 })), false);
  assert.equal(due(state({ last_synced_at: (now - 60 * 60_000) / 1000 })), true);
  assert.equal(due(state(), now - 10 * 60_000), false, 'a failed attempt waits an hour too');
  assert.equal(due(state(), 0, true), false, 'a pause stops it');
  assert.equal(due(state({ target: { repository: 'a/b', branch: 'x', auto_save: false } })), false);
  assert.equal(due(state({ has_token: false })), false);
  assert.equal(due(state({ target: null })), false);
});

test('a repository is only written when it is private, encrypted and on Spark\'s own branch', () => {
  const github = read('src-tauri/src/backup/github.rs');
  assert.match(github, /if !info\.private \{\n\s*return Err\("backup\.github_repo_public"\.into\(\)\);/, 'public repositories are refused');
  assert.match(github, /if info\.default_branch == branch \{/, 'the default branch is never replaced');
  assert.match(github, /fn push_checked\(&self, branch: &str, bytes: &\[u8\], keep_version: bool\) -> Result<\(\), String> \{\n\s*self\.check\(branch\)\?;/, 'checked before every send');
  assert.match(github, /self\.logged\("send", self\.push_checked\(/, 'a failed send is written to the error log');
  assert.match(github, /"parents": \[\]/, 'one commit without history, so the repository does not grow');
  const service = read('src-tauri/src/backup/github_backup.rs');
  assert.match(service, /if !encryption::enabled\(conn\)\? \{\n\s*return Err\("backup\.github_encryption_required"\.into\(\)\);/);
  assert.match(service, /password\.ok_or\("backup\.github_encryption_required"\)\?/, 'a send without encryption is refused');
  assert.match(service, /format!\("\{\}\/github", super::credentials::key\(conn\)\)/, 'its token is kept apart from the sync password');
});

test('every GitHub error has a message in both languages', () => {
  const errors = read('src/lib/i18n/errors.ts');
  const catalogs = ['en', 'pt-BR'].map((lang) => JSON.parse(read(`src/lib/i18n/catalogs/${lang}/backup.json`)));
  const sources = ['src-tauri/src/backup/github.rs', 'src-tauri/src/backup/github_backup.rs', 'src-tauri/src/commands/backup_github.rs'].map(read).join('\n');
  const codes = new Set([...sources.matchAll(/"backup\.github_([a-z_]+)"/g)].map((match) => match[1]));
  assert.ok(codes.size > 5);
  for (const code of codes) {
    assert.match(errors, new RegExp(`"backup.github_${code}": "backup.error_github_${code}"`), code);
    for (const catalog of catalogs) assert.ok(catalog[`backup.error_github_${code}`], `${code} message`);
  }
  for (const key of ['github_title', 'github_card_desc', 'github_connect', 'github_disconnect', 'github_push_desc', 'github_review_desc']) {
    for (const catalog of catalogs) assert.ok(catalog[`backup.${key}`], key);
  }
});
