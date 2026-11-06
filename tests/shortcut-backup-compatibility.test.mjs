import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { backupPreferences } from './native/backup.mjs';

test('all default shortcut actions are accepted by the backup contract', () => {
  const source = readFileSync('src-tauri/src/backup/validation.rs', 'utf8');
  const ids = [...source.match(/pub const SHORTCUT_IDS:.*?=\s*&\[([\s\S]*?)\];/)[1].matchAll(/"([^"]+)"/g)].map(match => match[1]);
  const defaults = backupPreferences().keyboard_shortcuts;
  assert.deepEqual(Object.keys(defaults).sort(), ids.sort());
  const signatures = Object.values(defaults).map(([binding]) => JSON.stringify([binding.key.toLowerCase(), binding.ctrl, binding.alt, binding.shift, binding.meta]));
  assert.equal(new Set(signatures).size, signatures.length);
});
