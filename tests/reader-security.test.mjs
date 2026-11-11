import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { test } from 'node:test';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('article HTML from web pages passes through the allowlist sanitizer before rendering', () => {
  const store = read('src/lib/stores/readerStore.ts');
  assert.match(store, /import \{ sanitizeArticleHtml \} from "\.\.\/services\/reader-sanitizer"/);
  assert.match(store, /content_html: sanitizeArticleHtml\(article\.content_html\)/);
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
  assert.match(source, /SAFE_IMAGE_SRC = \/\^\(https\?:\\\/\\\/\|data:image/, 'image sources restricted to http(s) and data:image');
});

test('Windows reader extraction has a timeout and the Linux stub escapes the page URL', () => {
  const reader = read('src-tauri/src/window/reader/mod.rs');
  assert.match(reader, /tokio::time::timeout\(std::time::Duration::from_secs\(10\), receiver\)/);
  assert.match(reader, /escape_html\(&tab\.url\)/);
  assert.match(reader, /fn escape_html\(value: &str\) -> String/);
});

test('reader documentation describes the content safety behavior in both languages', () => {
  assert.match(read('docs/en/features/reader-mode.md'), /Content Safety/);
  assert.match(read('docs/pt-BR/features/reader-mode.md'), /Segurança do Conteúdo/);
});
