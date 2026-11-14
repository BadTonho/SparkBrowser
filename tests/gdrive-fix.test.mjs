import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { gdriveFix, gdriveReason } from '../src/lib/services/backup/gdrive-fix.js';

const code = 'backup.sync_gdrive_login_required';

test('the reason after the colon picks the fix, and a bare code falls back to the generic hint', () => {
  assert.equal(gdriveReason(`${code}: account-not-linked`), 'account-not-linked');
  assert.equal(gdriveFix(`${code}: account-not-linked`), 'link');
  assert.equal(gdriveFix(`${code}: google-session-cookies-missing (SID=true, SAPISID=false)`), 'sign_in');
  assert.equal(gdriveFix(`${code}: account-email-not-detected`), 'confirm_account');
  assert.equal(gdriveFix(`${code}: account-email-changed`), 'different_account');
  assert.equal(gdriveFix(`${code}: no-drive-api-key-found`), 'page_changed');
  assert.equal(gdriveFix(`${code}: no-open-tab-in-the-linked-profile`), 'open_tab');
  assert.equal(gdriveFix(`${code}: tab-is-not-a-drive-page`), 'link_failed');
  assert.equal(gdriveFix(code), 'generic');
  assert.equal(gdriveFix(`${code}: something-new`), 'generic');
  assert.equal(gdriveFix(undefined), 'generic');
});

test('only a missing link is shown as guidance, so a linked account never gets told to link again', () => {
  for (const reason of ['account-email-not-detected', 'google-session-cookies-missing', 'no-open-tab-in-the-linked-profile', 'binding-not-saved']) {
    assert.notEqual(gdriveFix(`${code}: ${reason}`), 'link', reason);
  }
});

test('every fix has a hint in both languages', () => {
  const fixes = ['link', 'open_tab', 'sign_in', 'confirm_account', 'different_account', 'page_changed', 'link_failed', 'generic'];
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(readFileSync(new URL(`../src/lib/i18n/catalogs/${language}/backup.json`, import.meta.url), 'utf8'));
    for (const fix of fixes) assert.ok(catalog[`backup.fix_${fix}`], `${language} ${fix}`);
  }
});

test('every sign-in reason the browser-side path can report maps to the sign-in hint', () => {
  for (const reason of ['drive-page-shows-sign-in', 'hidden-page-shows-sign-in', 'no-sapisid-cookie-readable', 'drive-api-401', 'drive-download-401']) {
    assert.equal(gdriveFix(`${code}: ${reason}`), 'sign_in', reason);
  }
});
