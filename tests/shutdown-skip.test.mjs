import assert from 'node:assert/strict';
import { test } from 'node:test';
import { canSkipExitReview, hasIrreversibleRisk } from '../src/lib/services/shutdown-skip.js';

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
