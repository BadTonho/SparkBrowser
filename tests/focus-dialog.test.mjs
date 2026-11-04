import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

test('focus wraps through the summary while closed details fields stay outside the cycle', () => {
  const listeners = new Map(), document = { activeElement: null, addEventListener: (name, fn) => listeners.set(name, fn), removeEventListener: name => listeners.delete(name) };
  class Element {
    isConnected = true; details = null;
    focus() { document.activeElement = this; }
    contains(node) { return node === this; }
    getClientRects() { return [1]; }
    closest(selector) { return selector.startsWith('details') ? this.details : null; }
  }
  const previous = new Element(), close = new Element(), url = new Element(), submit = new Element(), summary = new Element(), hash = new Element();
  document.activeElement = previous;
  const root = new Element(); root.querySelectorAll = () => [close, url, submit, summary, hash];
  const details = { querySelector: () => summary, parentElement: root };
  summary.details = details; hash.details = details;
  const exports = {};
  runInNewContext(ts.transpileModule(readFileSync('src/lib/actions/focusDialog.ts', 'utf8'), { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText,
    { exports, document, HTMLElement: Element, Node: Element, queueMicrotask: fn => fn() });
  const action = exports.focusDialog(root, { onEscape() {} });
  const tab = shiftKey => listeners.get('keydown')({ key: 'Tab', shiftKey, preventDefault() {} });
  summary.focus(); tab(false); assert.equal(document.activeElement, close, 'closed summary is the final control');
  tab(true); assert.equal(document.activeElement, summary, 'Shift+Tab reaches the summary rather than an undisplayed hash input');
  summary.details = null; hash.details = null; // Expanded disclosure.
  hash.focus(); tab(false); assert.equal(document.activeElement, close);
  tab(true); assert.equal(document.activeElement, hash);
  action.destroy(); assert.equal(document.activeElement, previous);
});
