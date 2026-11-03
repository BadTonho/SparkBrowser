import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';
import { writable, derived, get } from 'svelte/store';
import { createZoomWriter } from '../src/lib/services/zoom-writer.js';

function fixture(override) {
  const currentUrl = writable('https://first.test/a'), writes = [], saved = new Map();
  let revision = 0;
  const snapshot = () => ({ revision, sites:[...saved].map(([origin,factor])=>({origin,factor})) });
  const exports = {};
  const imports = {
    'svelte/store': { writable, derived, get }, './browserStore': { currentUrl },
    '../services/zoom-writer.js': { createZoomWriter },
    '../services/bridge/runtime': { isTauriRuntime: () => true },
    '../services/user-errors.js': { describeError: error => error.message },
    '../services/bridge/zoom': {
      setSiteZoom: async (origin, factor) => {
        writes.push([origin,factor]);
        await override?.(origin, factor);
        saved.set(origin,factor); revision++;
        return snapshot();
      }, fetchSiteZoom: async () => snapshot(), onSiteZoom: async () => () => {},
    },
  };
  const source = ts.transpileModule(readFileSync('src/lib/stores/zoomStore.ts','utf8'), { compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022} }).outputText;
  runInNewContext(source, { exports, require: name => { if (!(name in imports)) throw Error(name); return imports[name]; }, queueMicrotask, URL });
  return { ...exports, currentUrl, writes, saved };
}

test('rapid shortcuts use the latest requested level and coalesce without dropping the final value', async () => {
  const f=fixture();
  await Promise.all([f.changeZoom(1),f.changeZoom(1),f.changeZoom(1)]);
  assert.deepEqual(f.writes,[['https://first.test',1.5]]);
  assert.equal(get(f.currentZoom),1.5);
  assert.equal(get(f.zoomBusy),false);
  assert.equal(Object.keys(get(f.zoomDrafts)).length,0);
  await f.changeZoom(-1); assert.equal(get(f.currentZoom),1.25);
  await f.changeZoom(0); assert.equal(get(f.currentZoom),1);
});

test('in-flight writes retain the latest drag value and their captured origin after navigation', async () => {
  let release;
  const gate=new Promise(resolve=>{release=resolve;});
  let first=true;
  const f=fixture(async()=>{if(first){first=false;await gate;}});
  const initial=f.setZoomFactor(1.1);
  await Promise.resolve();
  const updates=[f.setZoomFactor(1.5),f.setZoomFactor(2)];
  f.currentUrl.set('https://second.test/b');
  updates.push(f.changeZoom(1));
  assert.equal(get(f.zoomDrafts)['https://first.test'],2);
  release(); await Promise.all([initial,...updates]);
  assert.deepEqual(f.writes,[['https://first.test',1.1],['https://first.test',2],['https://second.test',1.1]]);
  assert.equal(get(f.currentZoom),1.1);
  assert.equal(f.saved.get('https://first.test'),2);
});

test('failed writes refresh confirmed state, clear drafts and remain retryable', async () => {
  let fail=true;
  const f=fixture(async()=>{if(fail)throw Error('injected zoom failure');});
  await f.setZoomFactor(2);
  assert.equal(get(f.currentZoom),1);
  assert.equal(Object.keys(get(f.zoomDrafts)).length,0);
  assert.equal(get(f.zoomErrors)['https://first.test'],'injected zoom failure');
  fail=false; await f.setZoomFactor(2);
  assert.equal(get(f.currentZoom),2);
  assert.equal(get(f.zoomErrors)['https://first.test'],undefined);
  await f.setZoomFactor(NaN); await f.setZoomFactor(5.1);
  assert.equal(f.writes.length,2);
});

test('a delayed write reply cannot replace a newer published zoom revision', async()=>{
  let release;
  const gate=new Promise(resolve=>{release=resolve;});
  const f=fixture(async()=>gate);
  const write=f.setZoomFactor(1.1);
  await Promise.resolve();
  f.zoomState.set({revision:100,sites:[{origin:'https://first.test',factor:2}]});
  release();await write;
  assert.equal(get(f.currentZoom),2);
  assert.equal(get(f.zoomState).revision,100);
});
