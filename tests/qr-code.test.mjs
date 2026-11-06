import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';
import QRCode from 'qrcode';
import { PNG } from 'pngjs';

function loadService(globals = {}) {
  const source = ts.transpileModule(readFileSync('src/lib/services/qr-code.ts', 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  runInNewContext(source, {
    exports,
    require: id => id === 'qrcode' ? { default: QRCode } : id === '@tauri-apps/api/core' ? { invoke: globals.invoke ?? (async () => true) } : (() => { throw Error(`Unexpected import: ${id}`); })(),
    Uint8Array, Blob, atob, Promise, ...globals,
  });
  return exports;
}

test('generates a valid local PNG that encodes the exact active URL', async () => {
  const url = 'https://example.test/a?x=1&y=two#part';
  const { makeQrPng } = loadService();
  const dataUrl = await makeQrPng(url);
  const bytes = Buffer.from(dataUrl.slice(dataUrl.indexOf(',') + 1), 'base64');
  assert.match(dataUrl, /^data:image\/png;base64,/);
  const decoded = PNG.sync.read(bytes);
  assert.equal(decoded.width, 288);
  assert.equal(decoded.height, 288);

  let passedUrl;
  const wrapped = {
    toDataURL: (text, options) => { passedUrl = text; return QRCode.toDataURL(text, options); },
  };
  const source = ts.transpileModule(readFileSync('src/lib/services/qr-code.ts', 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  runInNewContext(source, { exports, require: id => id === 'qrcode' ? { default: wrapped } : { invoke: async () => true }, Uint8Array, Blob, atob, Promise });
  await exports.makeQrPng(url);
  assert.equal(passedUrl, url);
});

test('copies PNG bytes as an image and never falls back to copying the URL', async () => {
  let written;
  class ClipboardImage { constructor(items) { this.items = items; } }
  const service = loadService({
    ClipboardItem: ClipboardImage,
    navigator: { clipboard: { write: async items => { written = items; } } },
  });
  const png = await QRCode.toDataURL('https://example.test/');
  await service.copyPngImage(png);
  assert.equal(written[0].items['image/png'].type, 'image/png');
});

test('saving sends only PNG bytes to the shell command and preserves dialog cancellation', async () => {
  let invocation;
  const service = loadService({ invoke: async (...args) => { invocation = args; return false; } });
  const png = await QRCode.toDataURL('https://example.test/');
  assert.equal(await service.savePngImage(png), false);
  assert.equal(invocation[0], 'save_qr_code_png');
  assert.equal(invocation[1].suggestedName, 'qr-code.png');
  assert.equal(Buffer.from(invocation[1].pngBase64, 'base64').subarray(0, 8).toString('hex'), '89504e470d0a1a0a');
});
