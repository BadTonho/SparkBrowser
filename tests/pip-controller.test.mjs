import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createPipController, selectPipVideo } from '../src/lib/services/pip-controller.js';

const video=(id,width,height,eligible=true)=>({id,width,height,eligible});
const frame=(context,videos)=>({context,document:`document-${context}`,videos});
const media={frames:[frame('main',[video('small',320,180),video('blocked',1920,1080,false)]),frame('iframe',[video('large',640,360),video('tie',640,360)])]};
const original={tabId:'tab-a',url:'https://fixture.test/video',workspaceId:'workspace-a',documentKey:'generation-a'};

test('selects the largest eligible video across frames and keeps the first equal-area candidate',()=>{
  assert.deepEqual(selectPipVideo(media),{context:'iframe',document:'document-iframe',video:'large'});
  assert.equal(selectPipVideo(undefined),null);
  assert.equal(selectPipVideo({frames:[frame('main',[video('blocked',1920,1080,false)])]}),null);
});

function fixture(overrides={}) {
  const calls=[], errors=[], busy=[];
  let context={...original};
  const options={
    readContext:()=>context,hasPip:()=>false,isBusy:()=>false,
    refresh:async id=>{calls.push(['refresh',id]);return media;},
    closeVolume:async()=>{calls.push(['close-volume']);},
    enter:async(...args)=>{calls.push(['enter',...args]);},
    exit:async()=>{calls.push(['exit']);},
    onBusy:value=>busy.push(value),onError:error=>errors.push(error),...overrides,
  };
  return {calls,errors,busy,toggle:createPipController(options),setContext:value=>{context=value;}};
}

test('a toolbar click closes volume and opens the largest video from the fresh report',async()=>{
  const f=fixture();await f.toggle();
  assert.deepEqual(f.calls,[['close-volume'],['refresh','tab-a'],['enter','tab-a','iframe','document-iframe','large']]);
  assert.deepEqual(f.busy,[true,false]);assert.deepEqual(f.errors,[]);
});

for(const field of ['tabId','url','workspaceId','documentKey']) {
  test(`changing ${field} while reporting cancels entry and repeated clicks cannot start another operation`,async()=>{
    let release,started;
    const gate=new Promise(resolve=>{release=resolve;});
    const reporting=new Promise(resolve=>{started=resolve;});
    let reports=0;
    const f=fixture({refresh:async()=>{reports++;started();await gate;return media;}});
    const pending=f.toggle();await reporting;
    f.setContext({...original,[field]:'changed'});
    await f.toggle();release();await pending;
    assert.equal(reports,1);assert.deepEqual(f.calls,[['close-volume']]);
    assert.deepEqual(f.busy,[true,false]);assert.deepEqual(f.errors,[]);
  });
}

test('the same button exits an existing floating video even when the focused tab has no candidate',async()=>{
  const f=fixture({hasPip:()=>true});f.setContext(null);await f.toggle();
  assert.deepEqual(f.calls,[['close-volume'],['exit']]);assert.deepEqual(f.errors,[]);
});

test('missing candidates and failed reports release the busy state and allow a retry',async()=>{
  let report=0;
  const f=fixture({refresh:async()=>{if(++report===1)throw new Error('native failure');return {frames:[]};}});
  await f.toggle();await f.toggle();
  assert.equal(f.errors[0].message,'native failure');assert.equal(f.errors[1].code,'media.pip_unavailable');
  assert.deepEqual(f.busy,[true,false,true,false]);
  assert.equal(f.calls.some(call=>call[0]==='enter'),false);
});

test('an already busy native PiP operation ignores a new toolbar click',async()=>{
  const f=fixture({isBusy:()=>true});await f.toggle();
  assert.deepEqual(f.calls,[]);assert.deepEqual(f.busy,[]);
});
