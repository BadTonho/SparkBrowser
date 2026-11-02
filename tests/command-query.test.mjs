import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createCommandQuery } from '../src/lib/services/command-query.js';
test('command query ignores late results, late errors and replies after closing', async () => {
  const requests = new Map(); const results=[]; const errors=[];
  const runner=createCommandQuery({delay:0,search:query=>new Promise((resolve,reject)=>requests.set(query,{resolve,reject})),result:value=>results.push(value),error:e=>errors.push(e)});
  const tick=()=>new Promise(resolve=>setTimeout(resolve,5));
  runner.set('first');await tick();runner.set('second');await tick();
  requests.get('second').resolve('current');requests.get('first').resolve('obsolete');await tick();
  assert.deepEqual(results,['current']);
  runner.set('third');await tick();runner.set('fourth');await tick();
  requests.get('third').reject(new Error('obsolete error'));runner.stop();requests.get('fourth').resolve('closed');await tick();
  assert.deepEqual(errors,[]);assert.deepEqual(results,['current']);
});
