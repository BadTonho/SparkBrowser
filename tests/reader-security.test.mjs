import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { test } from 'node:test';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('article HTML from web pages passes through the allowlist sanitizer before rendering', () => {
  const store = read('src/lib/stores/readerStore.ts');
  assert.match(store, /import \{ sanitizeArticleHtml \} from "\.\.\/services\/reader-sanitizer"/);
  assert.match(store, /sanitizeArticleHtml\(article\.content_html, article\.images \?\? \{\}\)/);
  assert.match(store, /readerArticle\.set\(\{ \.\.\.article, content_html: content, images: \{\} \}\)/, 'only the sanitized HTML is stored');
  assert.match(read('src/lib/components/reader/ReaderOverlay.svelte'), /\{@html \$readerArticle\.content_html\}/, 'store must be the only sanitization boundary');
});

test('sanitizer is default-deny: scripts, frames, forms, styles, handlers and unsafe URLs never survive', () => {
  const source = read('src/lib/services/reader-sanitizer.ts');
  for (const tag of ['script', 'iframe', 'object', 'embed', 'form', 'style', 'svg', 'template']) {
    assert.match(source, new RegExp(`"${tag}"`), `${tag} must be dropped`);
  }
  assert.match(source, /if \(!ALLOWED_TAGS\.has\(tag\)\)/, 'unknown elements must be unwrapped, not kept');
  assert.match(source, /if \(!rule \|\| !rule\(attribute\.value\)\)/, 'attributes must be allowlisted per tag');
  assert.doesNotMatch(source, /"(style|href|src|on[a-z]+)"\s*:/, 'no global style, href or event attribute may be allowed');
  assert.match(source, /SAFE_IMAGE_SRC = \/\^data:image\\\/\(png\|jpe\?g\|gif\|webp\);base64,\//, 'image sources restricted to raster data:image URLs');
});

test('reader images come from the tab cache as data: URLs because the shell CSP blocks remote images', () => {
  const csp = JSON.parse(read('src-tauri/tauri.conf.json')).app.security.csp;
  assert.match(csp, /img-src 'self' data: blob:;/, 'the shell must keep refusing remote images');
  const images = read('src-tauri/src/window/reader/images.rs');
  assert.match(images, /"Page\.getResourceContent"/, 'images are read from the page cache, not downloaded again');
  assert.doesNotMatch(images, /reqwest/, 'the reader must not fetch article images itself');
  const sanitizer = read('src/lib/services/reader-sanitizer.ts');
  assert.match(sanitizer, /if \(!source\) \{\s*child\.remove\(\);/, 'images missing from the cache are removed instead of shown broken');
});

test('Windows reader extraction has a timeout and other systems report a translated code', () => {
  const reader = read('src-tauri/src/window/reader/mod.rs');
  assert.match(reader, /tokio::time::timeout\(std::time::Duration::from_secs\(10\), receiver\)/);
  assert.match(reader, /#\[cfg\(not\(windows\)\)\]\s*\{[^}]*Err\("reader\.unsupported"\.into\(\)\)/);
  assert.match(read('src/lib/services/bridge/runtime.ts'), /"get_reader_article:reader\.unsupported"/, 'an expected limitation is not logged as a failure');
});

test('reader messages follow the interface language', () => {
  const store = read('src/lib/stores/readerStore.ts');
  assert.doesNotMatch(store, /throw new Error\("[^"]*[ãçéõ]/, 'no hardcoded Portuguese error text');
  assert.match(store, /get\(t\)\(key\)/);
  const overlay = read('src/lib/components/reader/ReaderOverlay.svelte');
  assert.match(overlay, /\$t\("reader\.byline", \{ name: \$readerArticle\.byline \}\)/);
  assert.doesNotMatch(overlay, />Por \{/);
});

test('reader documentation describes the content safety behavior in both languages', () => {
  assert.match(read('docs/en/features/reader-mode.md'), /Content Safety/);
  assert.match(read('docs/pt-BR/features/reader-mode.md'), /Segurança do Conteúdo/);
});
