import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';
function load(path) {
  const exports = {};
  runInNewContext(ts.transpileModule(readFileSync(path, 'utf8'), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText, { exports, atob });
  return exports;
}
const { isDownloadThumbnail, createDownloadVisualCache, downloadFileKind, formatDownloadBytes, downloadTaskSize } = load('src/lib/services/download-visual.ts');
const image = (width = 1, height = 1) => {
  const bytes = Buffer.alloc(45); Buffer.from([137,80,78,71,13,10,26,10]).copy(bytes); bytes.writeUInt32BE(13, 8); bytes.write('IHDR', 12); bytes.writeUInt32BE(width,16); bytes.writeUInt32BE(height,20);
  return 'data:image/png;base64,' + bytes.toString('base64');
};
test('previews allow only bounded small PNG data and reject remote, local and executable image sources', () => {
  assert.ok(isDownloadThumbnail(image())); assert.ok(isDownloadThumbnail(image(96,96)));
  const invalidHeader = Buffer.from(image().split(',')[1],'base64'); invalidHeader.write('FAIL',12);
  for (const value of [image(97,1), image(0,1), 'data:image/svg+xml,<svg/>', 'https://example.test/icon.png', 'file:///local/file.png', 'data:image/png;base64,' + 'A'.repeat(100_000), 'data:image/png;base64,'+invalidHeader.toString('base64'), 'data:image/png;base64,AAAA']) {
    assert.equal(isDownloadThumbnail(value), false);
  }
});
test('the cache coalesces reads, limits concurrency and applies the aggregate image budget', async () => {
  const pending = []; let calls = 0;
  const cache = createDownloadVisualCache(() => { calls++; return new Promise(resolve => pending.push(resolve)); }, image().length);
  const a = cache.get({kind:'task',id:'a'}, '1'); assert.equal(cache.get({kind:'task',id:'a'}, '1'), a);
  const b = cache.get({kind:'task',id:'b'}, '1'), c = cache.get({kind:'task',id:'c'}, '1');
  assert.equal(calls, 2);
  pending.shift()({size_bytes: 12, thumbnail_png:image()}); assert.ok((await a).thumbnail_png); assert.equal(calls,3);
  pending.shift()({size_bytes: 12, thumbnail_png:image()}); assert.equal((await b).thumbnail_png,null);
  pending.shift()({size_bytes: 12, thumbnail_png:'https://remote.test/image.png'}); assert.equal((await c).thumbnail_png,null);
  const fail = createDownloadVisualCache(async () => { throw Error('missing'); });
  assert.equal((await fail.get({kind:'history',id:1}, '1')).size_bytes,null);
});
test('file icons and readable byte labels preserve completed, partial, zero and unknown sizes', () => {
  for(const [name,kind] of [['Photo.JPG','image'],['movie.webm','video'],['song.flac','audio'],['archive.7z','archive'],['report.pdf','document'],['setup.exe','application'],['unknown','file']]) assert.equal(downloadFileKind(name),kind);
  assert.equal(formatDownloadBytes(0,String,'?'),'0 B'); assert.equal(formatDownloadBytes(1536,String,'?'),'1.5 KB');
  assert.equal(formatDownloadBytes(null,String,'?'),'?'); assert.equal(formatDownloadBytes(Infinity,String,'?'),'?');
  assert.equal(downloadTaskSize({status:'Completed',received:2048,total:2048},String,'?'),'2 KB');
  assert.equal(downloadTaskSize({status:'Paused',received:512,total:2048},String,'?'),'512 B / 2 KB');
  assert.equal(downloadTaskSize({status:'Queued',received:0,total:null},String,'?'),'?');
});
test('native focus follows form expansion and collapse without stealing focus on progress updates', () => {
  const { syncDownloadFocus } = load('src/lib/services/download-panel-focus.ts');
  let input = null, focused = '';
  const button = {focus:()=> focused='button'}, field = {focus:()=> focused='input'};
  const root = {querySelector: selector => selector.includes('download-url') ? input : button};
  syncDownloadFocus(root,true,false); assert.equal(focused,'button');
  input=field; syncDownloadFocus(root,false,false); assert.equal(focused,'input');
  focused='other'; syncDownloadFocus(root,false,true); assert.equal(focused,'other');
  input=null; syncDownloadFocus(root,false,true); assert.equal(focused,'button');
  input=field; syncDownloadFocus(root,false,false);
  button.disabled=true; input=null;
  syncDownloadFocus(root,false,false); assert.equal(focused,'input', 'a superseded render cannot lose the busy submit focus request');
  button.disabled=false;
  syncDownloadFocus(root,false,false); assert.equal(focused,'button', 'enabling the confirmed control completes the pending focus request');
  focused='other'; syncDownloadFocus(root,false,false); assert.equal(focused,'other');
});
