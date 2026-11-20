import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync } from 'node:fs';
import { canSkipExitReview, hasIrreversibleRisk, isEmptyReview, reviewNeedsAttention, shutdownPresentation, SHUTDOWN_REVEAL_DELAY_MS } from '../src/lib/services/shutdown-skip.js';

const tab = (overrides = {}) => ({ id: 't', title: 'Tab', url: 'https://example.test', form: false, media: false, native_download: false, ...overrides });
const state = (review, overrides = {}) => ({ purpose: 'exit', update_version: null, review: { revision: 1, tabs: [], downloads: [], native_downloads: [], ...review }, ...overrides });

test('media and resumable transfers are skipped only when the preference is on', () => {
  const light = state({ tabs: [tab({ media: true })], downloads: ['transfer'] });
  assert.equal(canSkipExitReview(light, true), true);
  assert.equal(canSkipExitReview(light, false), false);
});

test('edited forms and non-resumable downloads always keep the review', () => {
  for (const review of [{ tabs: [tab({ form: true })] }, { tabs: [tab({ native_download: true })] }, { native_downloads: ['d'] }]) {
    assert.equal(hasIrreversibleRisk(state(review).review), true);
    assert.equal(canSkipExitReview(state(review), true), false);
  }
});

test('updates, engine restarts and missing reviews are never skipped', () => {
  assert.equal(canSkipExitReview(state({}, { update_version: '1.2.3' }), true), false);
  assert.equal(canSkipExitReview(state({}, { purpose: 'install_update' }), true), false);
  assert.equal(canSkipExitReview(state({}, { purpose: 'restart_runtime' }), true), false);
  assert.equal(canSkipExitReview({ purpose: 'exit', update_version: null, review: null }, true), false);
});

test('an empty review never needs attention, and "don\'t ask again" covers only what can resume', () => {
  assert.equal(isEmptyReview(state({}).review), true);
  assert.equal(isEmptyReview(state({ tabs: [tab()] }).review), false);
  assert.equal(reviewNeedsAttention(state({}), false), false);
  assert.equal(reviewNeedsAttention(state({ tabs: [tab({ media: true })] }), false), true);
  assert.equal(reviewNeedsAttention(state({ tabs: [tab({ media: true })] }), true), false);
  assert.equal(reviewNeedsAttention(state({ tabs: [tab({ form: true })] }), true), true, 'an edited form is always asked about');
  assert.equal(reviewNeedsAttention(state({ downloads: ['x'] }, { purpose: 'restart_runtime' }), true), true, 'engine restarts always ask');
  assert.equal(reviewNeedsAttention({ purpose: 'exit', update_version: null, review: null }, false), false);
});

test('the dialog appears at once only for a decision or an error; saving stays hidden unless it takes long', () => {
  const phase = (name, review = null, overrides = {}) => ({ phase: name, purpose: 'exit', update_version: null, review, ...overrides });
  assert.equal(shutdownPresentation(phase('idle'), false), 'hidden');
  assert.equal(shutdownPresentation(phase('flushing'), false), 'working', 'saving nothing visible yet');
  assert.equal(shutdownPresentation(phase('committing', state({}).review), false), 'working');
  assert.equal(shutdownPresentation(phase('review', state({}).review), false), 'working', 'an empty review is confirmed without being shown');
  assert.equal(shutdownPresentation(phase('review', state({ tabs: [tab({ form: true })] }).review), true), 'attention');
  assert.equal(shutdownPresentation(phase('review', state({ tabs: [tab({ media: true })] }).review), true), 'working', 'skipped by preference');
  assert.equal(shutdownPresentation(phase('error'), true), 'attention');
  assert.ok(SHUTDOWN_REVEAL_DELAY_MS >= 300 && SHUTDOWN_REVEAL_DELAY_MS <= 1500);
});

test('closing skips the cloud upload when nothing changed and the dialog follows the presentation', () => {
  const backup = readFileSync('src/lib/stores/backupStore.ts', 'utf8');
  const exitFlush = backup.slice(backup.indexOf('export async function flushBackupSyncBeforeShutdown'), backup.indexOf('function startBackupStatus'));
  assert.match(exitFlush, /if \(!hasPendingChangesForCloud\) return;/);
  assert.ok(exitFlush.indexOf('!hasPendingChangesForCloud') < exitFlush.indexOf('pushBackupSync'), 'checked before the network call');
  assert.match(backup, /notesRevision\.subscribe\(notifyPersistentDataChanged\)/, 'note edits count as changes');
  const store = readFileSync('src/lib/stores/shutdownStore.ts', 'utf8');
  assert.match(store, /shutdownPresentation\(state, get\(skipExitReview\)\)/);
  assert.doesNotMatch(store, /setOverlayVisibility\(state\.phase !== "idle"/, 'the overlay follows the dialog, not the phase');
  assert.match(readFileSync('src/lib/components/shutdown/ShutdownDialog.svelte', 'utf8'), /\{#if \$shutdownVisible && \$shutdownState\.phase !== "idle"\}/);
});

test('closing never builds the exit backup when nothing changed since the last one', () => {
  const backup = readFileSync('src/lib/stores/backupStore.ts', 'utf8');
  const exitFlush = backup.slice(backup.indexOf('export async function flushBackupSyncBeforeShutdown'), backup.indexOf('function startBackupStatus'));
  assert.match(exitFlush, /schedule\.on_exit && localChanges !== localSavedChanges/);
  assert.ok(exitFlush.indexOf('localChanges !== localSavedChanges') < exitFlush.indexOf('createLocal("exit"'), 'checked before the backup is built');
  // Every change counts, and a backup only clears the changes that existed when it started.
  assert.match(backup, /function notifyPersistentDataChanged\(\): void \{\s*if \(!autoSyncReady\) return;\s*localChanges \+= 1;/);
  assert.match(backup, /const changesAtStart = localChanges;[\s\S]*localSavedChanges = changesAtStart;/);
});
