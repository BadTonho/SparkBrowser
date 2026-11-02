import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createMediaRefresher } from '../src/lib/services/media-refresher.js';

test('reports share a pending request per tab, allow independent tabs and recover after failure', async () => {
  const calls=[], applied=[], releases=new Map();
  const refresh=createMediaRefresher(id=>{calls.push(id);return new Promise((resolve,reject)=>releases.set(id,{resolve,reject}));},value=>applied.push(value));
  const a=refresh('a'), again=refresh('a'), b=refresh('b');
  assert.equal(a,again,'a click must await the report already requested by the timer');
  await Promise.resolve();
  assert.deepEqual(calls,['a','b']);
  releases.get('b').resolve({tab:'b'});
  await b;
  assert.deepEqual(applied,[{tab:'b'}],'another tab need not wait for a');
  releases.get('a').reject(new Error('report failed'));
  await assert.rejects(a,/report failed/);
  const retry=refresh('a');
  await Promise.resolve();
  releases.get('a').resolve({tab:'a'});
  await retry;
  assert.deepEqual(calls,['a','b','a']);
  assert.deepEqual(applied,[{tab:'b'},{tab:'a'}]);
});
