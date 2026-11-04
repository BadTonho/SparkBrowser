import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

function fixture() {
  let listener, release, stops = 0, calls = 0, resolveRestart;
  const writable = value => ({ set: next => { value = next; }, subscribe: fn => { fn(value); return () => {}; } });
  const get = store => { let value; store.subscribe(v => value = v)(); return value; };
  const shutdown = [];
  const unavailable = { supported: false, currentVersion: null, availableVersion: null, restartRequired: false, revision: 0, error: null };
  const imports = {
    'svelte/store': { get, writable },
    '../services/bridge/webview-runtime': {
      unavailableRuntime: unavailable,
      onRuntimeStatus: async fn => { listener = fn; return () => stops++; },
      fetchRuntimeStatus: () => new Promise(resolve => release = resolve),
      requestRuntimeRestart: () => { calls++; return new Promise(resolve => resolveRestart = resolve); },
    },
    './shutdownStore': { applyShutdownState: state => shutdown.push(state) }, '../services/user-errors.js': { describeError: String },
  };
  const exports = {};
  runInNewContext(ts.transpileModule(readFileSync('src/lib/stores/webviewRuntimeStore.ts', 'utf8'), { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText, { exports, require: name => imports[name] });
  return { ...exports, get, shutdown, emit: value => listener(value), release: value => release(value), resolveRestart: value => resolveRestart(value), calls: () => calls, stops: () => stops };
}
const status = revision => ({ supported: true, currentVersion: '156.0.0.1', availableVersion: '156.0.0.2', restartRequired: true, revision, error: null });
test('a delayed initial runtime read cannot replace a newer update event; disposal releases its listener', async () => {
  const f = fixture(), stop = f.initializeRuntime(); await new Promise(setImmediate);
  f.emit(status(2)); f.release({ ...status(1), restartRequired: false }); await new Promise(setImmediate);
  assert.equal(f.get(f.runtimeStatus).revision, 2); assert.equal(f.get(f.runtimeStatus).restartRequired, true);
  stop(); f.emit(status(3)); assert.equal(f.stops(), 1); assert.equal(f.get(f.runtimeStatus).revision, 2);
});
test('restart requires a pending runtime and duplicate clicks share the protected shutdown request', async () => {
  const f = fixture(); await f.restartRuntime(); assert.equal(f.calls(), 0);
  f.runtimeStatus.set(status(1)); const restart = f.restartRuntime(); await f.restartRuntime();
  assert.equal(f.calls(), 1); assert.equal(f.get(f.runtimeBusy), true);
  const state = { phase: 'flushing', purpose: 'restart_runtime' }; f.resolveRestart(state); await restart;
  assert.equal(f.shutdown[0], state); assert.equal(f.get(f.runtimeBusy), false);
  f.dismissedRuntimeVersion.set(status(1).availableVersion); assert.equal(f.get(f.runtimeStatus).restartRequired, true, 'Later does not clear backend pending state');
});
