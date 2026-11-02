import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createMediaVolumeWriter } from '../src/lib/services/media-volume-writer.js';
test('volume edits coalesce per document, serialize writes and retain later edits after failures', async()=>{
  const calls=[],values=[],errors=[],settled=[];let release;
  const writer=createMediaVolumeWriter(async item=>{calls.push(item);if(calls.length===1)await new Promise(r=>release=r);if(item.value===150)throw Error('stale');return item.value;},v=>values.push(v),e=>errors.push(e.message),(i,p)=>settled.push([i.value,p]));
  writer.schedule({id:'a',document:'old',value:110});writer.schedule({id:'a',document:'old',value:120});
  const done=writer.flush();await Promise.resolve();
  writer.schedule({id:'a',document:'old',value:150});writer.schedule({id:'a',document:'new',value:200});
  release();await done;
  assert.deepEqual(calls.map(i=>i.value),[120,150,200]);assert.deepEqual(values,[120,200]);assert.deepEqual(errors,['stale']);assert.deepEqual(settled,[[120,true],[150,false],[200,false]]);
});
