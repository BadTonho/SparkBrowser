import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync } from 'node:fs';
import { buildIssueUrl, ISSUES_URL } from '../src/lib/services/error-report.js';

const report = {
  id: 'error-1', kind: 'panic', summary: 'Internal error in thread main: boom',
  details: 'at src/lib.rs:1\nstack', version: '0.1.2', os: 'windows', time: '2026-11-16 20:00:00',
};

test('the issue address points at the repository and carries the error', () => {
  const url = new URL(buildIssueUrl(report));
  assert.equal(`${url.origin}${url.pathname}`, ISSUES_URL);
  assert.match(url.searchParams.get('title'), /^\[panic\] Internal error/);
  const body = url.searchParams.get('body');
  for (const text of ['0.1.2', 'windows', report.summary, 'at src/lib.rs:1']) assert.ok(body.includes(text), text);
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
