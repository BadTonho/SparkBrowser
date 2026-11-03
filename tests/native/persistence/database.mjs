import assert from 'node:assert/strict';
import { DatabaseSync } from 'node:sqlite';
import { realpathSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { validateRunDirectory } from '../run-directory.mjs';

// The harness verifies app identity/data path before passing this isolated directory.
// Live connections only install fixture triggers or inspect committed artificial data.
export async function withScenarioDatabase(directory, action) {
  const root = validateRunDirectory(directory);
  const data = realpathSync(resolve(root, 'data'));
  const file = realpathSync(resolve(data, 'veloce_browser.db'));
  assert.equal(relative(data, file), 'veloce_browser.db');
  const db = new DatabaseSync(file);
  try { db.exec('PRAGMA busy_timeout=5000'); return await action(db); }
  finally { db.close(); }
}
