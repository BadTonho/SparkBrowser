import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync } from 'node:fs';
import { buildIssueUrl, ISSUES_URL } from '../src/lib/services/error-report.js';

const report = {
  id: 'error-1', level: 'critical', kind: 'panic', summary: 'Internal error in thread main: boom',
  details: 'at src/lib.rs:1\nstack', version: '0.1.2', os: 'windows', time: '2026-11-16 20:00:00',
};

test('the issue address points at the repository and carries the error', () => {
  const url = new URL(buildIssueUrl(report));
  assert.equal(`${url.origin}${url.pathname}`, ISSUES_URL);
  assert.match(url.searchParams.get('title'), /^\[panic\] Internal error/);
  const body = url.searchParams.get('body');
  for (const text of ['0.1.2', 'windows', 'critical', report.summary, 'at src/lib.rs:1']) assert.ok(body.includes(text), text);
});

test('a huge error is cut so the address stays short enough for GitHub', () => {
  const url = buildIssueUrl({ ...report, details: 'line of a long backtrace\n'.repeat(2000) });
  assert.ok(url.length <= 7000, String(url.length));
  assert.match(new URL(url).searchParams.get('body'), /the full text is in spark-errors\.log/);
});

test('the notification is wired into the shell, every interface and the new-tab page', () => {
  assert.match(readFileSync('src/App.svelte', 'utf8'), /<ErrorReportToast \/>/);
  assert.match(readFileSync('src/main.ts', 'utf8'), /report_frontend_error/);
  assert.match(readFileSync('src-tauri/src/window/new_tab/bridge.js', 'utf8'), /send\('error'/);
  assert.match(readFileSync('src-tauri/capabilities/default.json', 'utf8'), /allow-report-frontend-error/);
});

test('silent failures are logged as warnings instead of being swallowed', () => {
  assert.match(readFileSync('src/lib/services/ignored-error.ts', 'utf8'), /level: "warning"/);
  assert.match(readFileSync('src-tauri/src/commands/error_report.rs', 'utf8'), /Level::Warning/);
  assert.doesNotMatch(readFileSync('src/lib/stores/downloadsStore.ts', 'utf8'), /\.catch\(\(\) => \{\}\)/);
});

test('the settings can list the latest entries of the error log', () => {
  assert.match(readFileSync('src-tauri/src/commands/manifest.rs', 'utf8'), /"get_recent_errors"/);
  assert.match(readFileSync('src-tauri/src/lib.rs', 'utf8'), /get_recent_errors/);
  assert.match(readFileSync('src-tauri/capabilities/default.json', 'utf8'), /"allow-get-recent-errors"/);
  assert.match(readFileSync('src/lib/services/bridge/diagnostics.ts', 'utf8'), /invoke<RecentErrors>\("get_recent_errors"\)/);
  assert.match(readFileSync('src/lib/components/settings/DiagnosticsSettings.svelte', 'utf8'), /fetchRecentErrors/);
});

test('expected command answers are not logged as failures, real ones still are', () => {
  const runtime = readFileSync('src/lib/services/bridge/runtime.ts', 'utf8');
  assert.match(runtime, /"refresh_tab_media:media\.unavailable"/);
  assert.match(runtime, /"set_settings_overlay:settings_tab\.not_ready"/);
  assert.match(runtime, /!isExpectedFailure\(command, error\)/);
  assert.doesNotMatch(runtime, /"refresh_tab_media:media\.error"/);
});

test('copying falls back to the copy command when the document has no focus', () => {
  assert.match(readFileSync('src/lib/services/clipboard.ts', 'utf8'), /execCommand\("copy"\)/);
  assert.match(readFileSync('src/lib/components/settings/DiagnosticsSettings.svelte', 'utf8'), /copyText/);
});
