import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createPipShadowPreference,readPipShadow,PIP_SHADOW_STORAGE_KEY} from '../src/lib/services/pip-shadow-preference.js';

function fixture(saved=null,apply=async value=>value,runtime=true) {
  const values=new Map(saved===null?[]:[[PIP_SHADOW_STORAGE_KEY,saved]]),states=[],errors=[];
  const storage={getItem:key=>values.get(key)??null,setItem:(key,value)=>values.set(key,value)};
  const preference=createPipShadowPreference({storage,apply,runtime,changed:state=>states.push({...state}),failed:error=>errors.push(error),unavailable:error=>error?.code==='media.unavailable'});
  return {preference,values,states,errors,state:()=>states.at(-1)};
}
test('missing, invalid and denied shadow storage defaults off; only literal true opts in',()=>{
  for(const value of [null,'','false','1','yes','TRUE','{}'])assert.equal(readPipShadow({getItem:()=>value}),false);
  assert.equal(readPipShadow({getItem:()=>{throw Error('denied');}}),false);
  assert.equal(readPipShadow({getItem:()=> 'true'}),true);
});
test('startup and later edits serialize, and entry waits for the pending preference',async()=>{
  const calls=[],releases=[];
  const f=fixture('true',value=>new Promise(resolve=>{calls.push(value);releases.push(()=>resolve(value));}));
  const startup=f.preference.initialize();await Promise.resolve();assert.deepEqual(calls,[true]);
  const changed=f.preference.set(false);let ready=false;void f.preference.initialize().then(()=>{ready=true;});
  assert.equal(f.state().busy,true);assert.equal(f.values.get(PIP_SHADOW_STORAGE_KEY),'true');
  releases.shift()();await new Promise(resolve=>setImmediate(resolve));assert.equal(ready,false);
  // A subsequent entry waits for edits queued after initial startup as well.
  let nextReady=false;const entry=f.preference.initialize().then(()=>{nextReady=true;});
  assert.equal(nextReady,false);assert.deepEqual(calls,[true,false]);releases.shift()();await changed;await entry;await startup;assert.equal(ready,true);
  assert.equal(f.state().value,false);assert.equal(f.state().busy,false);assert.equal(f.values.get(PIP_SHADOW_STORAGE_KEY),'false');
});
test('native failures retain confirmed value and storage, and do not poison later edits',async()=>{
  let fail=false;
  const f=fixture(null,async value=>{if(fail)throw Error('native failed');return value;});
  await f.preference.initialize();await f.preference.set(true);fail=true;
  await assert.rejects(f.preference.set(false),/native failed/);
  assert.equal(f.state().value,true);assert.equal(f.values.get(PIP_SHADOW_STORAGE_KEY),'true');assert.equal(f.errors.length,1);
  fail=false;await f.preference.set(false);assert.equal(f.state().value,false);assert.equal(f.state().busy,false);
});
test('unsupported engines and preview remain unavailable but can retain imported preferences',async()=>{
  for(const runtime of [false,true]) {
    const f=fixture(null,async()=>{throw {code:'media.unavailable'};},runtime);
    await f.preference.initialize();await f.preference.set(true);
    assert.equal(f.state().available,false);assert.equal(f.state().value,true);assert.equal(f.values.get(PIP_SHADOW_STORAGE_KEY),'true');assert.equal(f.errors.length,0);
  }
});
test('failed initialization can retry, and simultaneous initialization shares one call',async()=>{
  let calls=0;
  const f=fixture('true',async value=>{if(++calls===1)throw Error('startup failed');return value;});
  await assert.rejects(f.preference.initialize(),/startup failed/);assert.equal(f.state().value,false);
  await Promise.all([f.preference.initialize(),f.preference.initialize()]);assert.equal(calls,2);assert.equal(f.state().value,true);
});
