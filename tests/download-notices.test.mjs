import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';
const exports = {};
runInNewContext(ts.transpileModule(readFileSync('src/lib/services/download-notices.ts', 'utf8'), {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
}).outputText, { exports });
const { createDownloadNotices } = exports;
const row = (id, revision, status) => ({ id, revision, status, url: 'https://private.example/file', save_path: 'C:/private/file' });
function storage(raw = null) {
  let value = raw, writes = 0;
  return { getItem: () => value, setItem(key, next) { assert.equal(key, 'veloce_download_notices'); value = next; writes++; }, value: () => value, writes: () => writes };
}
const counts = ledger => ({ ...ledger.summary() });

test('existing history establishes a quiet baseline and subsequent results create notices once', () => {
  const saved = storage(), ledger = createDownloadNotices(saved);
  ledger.observe([row('old', 3, 'Completed'), row('failed', 2, 'Failed')], 'snapshot');
  assert.deepEqual(counts(ledger), { completed: 0, error: 0 });
  ledger.observe([row('one', 1, 'Downloading')], 'event');
  const writes = saved.writes();
  ledger.observe([row('one', 2, 'Downloading')], 'event');
  assert.equal(saved.writes(), writes, 'progress alone does not rewrite local storage');
  ledger.observe([row('one', 3, 'Completed')], 'event');
  assert.equal(ledger.summary().completed, 1);
  ledger.acknowledge();
  for (const revision of [1, 3, 4]) ledger.observe([row('one', revision, 'Completed')], 'event');
  assert.equal(ledger.summary().completed, 0, 'duplicates and same-state revisions do not rearm');
  ledger.observe([row('one', 5, 'Verifying')], 'event');
  ledger.observe([row('one', 6, 'VerificationFailed')], 'event');
  assert.equal(ledger.summary().error, 1);
  const json = saved.value();
  assert.ok(!json.includes('private.example') && !json.includes('C:/private') && !json.includes('url'));
});

test('notices and acknowledgements survive restart; old snapshots and closed-session events cannot resurrect them', () => {
  const saved = storage(); let ledger = createDownloadNotices(saved);
  ledger.observe([], 'snapshot');
  ledger.observe([row('done', 1, 'Completed'), row('bad', 1, 'Failed')], 'event');
  ledger = createDownloadNotices(saved);
  assert.deepEqual(counts(ledger), { completed: 1, error: 1 });
  ledger.observe([row('done', 1, 'Completed'), row('bad', 1, 'Failed')], 'snapshot');
  ledger.acknowledge(); ledger = createDownloadNotices(saved);
  ledger.observe([row('done', 1, 'Completed'), row('bad', 1, 'Failed')], 'snapshot');
  assert.deepEqual(counts(ledger), { completed: 0, error: 0 });
  ledger.observe([row('bad', 0, 'Completed')], 'event');
  assert.deepEqual(counts(ledger), { completed: 0, error: 0 });
});

test('snapshot catches missed transitions without treating the initial history as new', () => {
  const saved = storage(); let ledger = createDownloadNotices(saved);
  ledger.observe([row('running', 1, 'Downloading')], 'snapshot');
  ledger = createDownloadNotices(saved);
  ledger.observe([row('running', 3, 'Completed')], 'snapshot');
  assert.equal(ledger.summary().completed, 1);
  ledger.observe([row('running', 3, 'Completed'), row('new', 2, 'Failed')], 'snapshot');
  assert.equal(ledger.summary().error, 1);
  ledger.observe([row('running', 3, 'Completed')], 'snapshot');
  assert.equal(ledger.summary().error, 0, 'removed tasks do not leave ghost notices');
});

test('cancelled tasks create no error; a later result while the panel is open creates a new notice', () => {
  const ledger = createDownloadNotices(storage());
  ledger.observe([], 'snapshot');
  for (const status of ['Cancelled', 'Cancelling', 'Interrupted', 'RestartRequired', 'Paused']) ledger.observe([row(status, 1, status)], 'event');
  assert.deepEqual(counts(ledger), { completed: 0, error: 0 });
  ledger.acknowledge(); ledger.observe([row('one', 1, 'Completed')], 'event');
  assert.equal(ledger.summary().completed, 1);
});

test('malformed preferences reset the initial baseline and unavailable storage retains session state', () => {
  const good = { id: 'one', revision: 1, status: 'Completed', notice: 'completed' };
  for (const raw of ['{', 'null', JSON.stringify({ version: 2, entries: [] }), ...[
    [good, good], [{ ...good, revision: -1 }], [{ ...good, notice: 'surprise' }], [{ ...good, status: 'Unknown' }], [{ ...good, id: 'x'.repeat(129) }],
  ].map(entries => JSON.stringify({ version: 1, initialized: true, entries }))]) {
    const ledger = createDownloadNotices(storage(raw));
    ledger.observe([row('old', 3, 'Completed')], 'snapshot');
    assert.deepEqual(counts(ledger), { completed: 0, error: 0 });
  }
  const ledger = createDownloadNotices({ getItem() { throw Error('read'); }, setItem() { throw Error('write'); } });
  ledger.observe([row('one', 1, 'Failed')], 'event');
  assert.equal(ledger.summary().error, 1);
  ledger.acknowledge(); assert.equal(ledger.summary().error, 0);
});
