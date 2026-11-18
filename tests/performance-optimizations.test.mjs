import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { test } from 'node:test';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('cached filter lists are refreshed only once they are due, not on every launch', () => {
  const filters = read('src-tauri/src/privacy/filter_lists.rs');
  assert.match(filters, /const REFRESH_INTERVAL: Duration = Duration::from_secs\(24 \* 60 \* 60\);/);
  assert.match(filters, /std::thread::sleep\(refresh_delay\(&service\.cache\)\);/);
  assert.match(filters, /\.map_or\(Duration::ZERO, \|age\| REFRESH_INTERVAL\.saturating_sub\(age\)\)/);
});

test('panel host reads theme variables only after root style changes, not on every frame', () => {
  const host = read('src/lib/components/panels/panel-host.ts');
  assert.match(host, /if \(themeChanged\) \{\s*const style = getComputedStyle\(document\.documentElement\);/);
  assert.doesNotMatch(host, /const r = session\.anchor\.getBoundingClientRect\(\), style = getComputedStyle/);
  assert.match(host, /attributeFilter: \["style", "class", "data-theme", "data-palette"\]/);
  assert.match(host, /themeObserver\.disconnect\(\);/);
});

test('media polling keeps video discovery but slows down when the focused tab has no media', () => {
  const media = read('src/lib/stores/mediaLifecycle.ts');
  assert.doesNotMatch(media, /setInterval\(refreshFocused/);
  assert.match(media, /POLL_WITH_MEDIA_MS = 1000, POLL_WITHOUT_MEDIA_MS = 3000/);
  assert.match(media, /clearTimeout\(pollTimer\);/);
});

test('SQLite sets bounded storage pragmas and caches hot read statements', () => {
  const connection = read('src-tauri/src/database/connection.rs');
  for (const pragma of ['busy_timeout = 5000', 'temp_store = MEMORY', 'cache_size = -8000', 'mmap_size = 268435456']) {
    assert.match(connection, new RegExp(`PRAGMA ${pragma.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')};`));
  }
  assert.match(read('src-tauri/src/database/history_repo.rs'), /\.prepare_cached\(/);
  assert.match(read('src-tauri/src/database/bookmarks_repo.rs'), /\.prepare_cached\("SELECT id, title, url, icon, folder_id/);
});
