import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';

function fixture(reduced=false){
  const events=new Map(),nodes=new Map(),actions=[],frames=new Map(),timers=new Map(),rects=[];
  let counter=0;
  const listen=(name,fn)=>{const previous=events.get(name);events.set(name,event=>{previous?.(event);fn(event);});};
  const context={clearRect(){rects.length=0;},fillRect(...r){rects.push(r);},createLinearGradient(){return {addColorStop(){}};}};
  for(const id of ['audio-background','audio-background-enabled','audio-background-options','audio-background-color','audio-background-intensity','audio-background-settings','audio-background-status'])nodes.set(id,{hidden:true,listeners:{},addEventListener(name,fn){this.listeners[name]=fn;},getContext(){return context;}});
  const document={hidden:false,getElementById:id=>nodes.get(id),addEventListener:listen};
  const window={innerWidth:1000,innerHeight:800,devicePixelRatio:1,addEventListener:listen,
    matchMedia:()=>({matches:reduced,addEventListener(){}}),
    requestAnimationFrame:fn=>{const id=++counter;frames.set(id,fn);return id;},cancelAnimationFrame:id=>frames.delete(id),
    setTimeout:fn=>{const id=++counter;timers.set(id,fn);return id;},clearTimeout:id=>timers.delete(id),
    dispatchEvent:event=>actions.push(event.detail)};
  runInNewContext(readFileSync('public/new-tab/audio-background.js','utf8'),{window,document,CustomEvent:class{constructor(type,options){this.type=type;this.detail=options.detail;}}});
  const emit=(name,detail)=>events.get(name)?.({detail});
  const render=value=>window.__sparkAudioRender({enabled:true,color:'#1db954',intensity:1,...value});
  const tick=now=>{const pending=[...frames.values()];frames.clear();for(const fn of pending)fn(now);};
  emit('spark-new-tab',{labels:JSON.parse(readFileSync('src/lib/i18n/catalogs/en/newtab.json','utf8')),private:false});
  return {nodes,actions,document,render,emit,tick,frames,timers,rects};
}
test('audio bars use real spectral values, expire to silence and never intercept page controls',()=>{
  const f=fixture();f.render();assert.equal(f.actions.at(-1).operation,'audio_subscribe');
  f.emit('spark-audio-frame',{status:'ready',bands:Array(32).fill(0.8)});f.tick(100);
  assert.equal(f.nodes.get('audio-background').hidden,false);assert.equal(f.rects.length,32);
  assert.ok(f.rects.every(r=>r[3]>0&&r[3]<=200));
  for(const fn of [...f.timers.values()])fn();
  for(let t=150;t<1500;t+=50)f.tick(t);
  assert.equal(f.nodes.get('audio-background').hidden,true);assert.equal(f.frames.size,0);
  assert.match(readFileSync('public/new-tab/audio-background.css','utf8'),/pointer-events:none/);
});
test('disabled, hidden and private new tabs stop drawing and unsubscribe',()=>{
  const f=fixture();f.render();f.emit('spark-audio-frame',{status:'ready',bands:Array(32).fill(1)});f.tick(100);
  f.document.hidden=true;f.emit('visibilitychange');assert.equal(f.actions.at(-1).operation,'audio_unsubscribe');assert.equal(f.frames.size,0);
  f.document.hidden=false;f.emit('visibilitychange');assert.equal(f.actions.at(-1).operation,'audio_subscribe');
  f.emit('spark-new-tab',{private:true,labels:{}});assert.equal(f.actions.at(-1).operation,'audio_unsubscribe');
  f.emit('spark-audio-frame',{status:'ready',bands:Array(32).fill(1)});assert.equal(f.frames.size,0);assert.equal(f.nodes.get('audio-background-settings').hidden,true);
});
test('reduced motion uses discrete frames and invalid spectra cannot drive the canvas',()=>{
  const f=fixture(true);f.render();f.tick(100);
  f.emit('spark-audio-frame',{status:'ready',bands:[Infinity]});assert.equal(f.frames.size,0);
  f.emit('spark-audio-frame',{status:'ready',bands:Array(32).fill(0.5)});f.tick(200);
  assert.equal(f.frames.size,0);assert.equal(f.rects.length,32);
  f.emit('spark-audio-frame',{status:'audio_unavailable',bands:Array(32).fill(0)});f.tick(400);
  assert.equal(f.nodes.get('audio-background').hidden,true);assert.match(f.nodes.get('audio-background-status').textContent,/unavailable/);
});
test('the document bridge rejects stale generations, replayed frames, wrong documents and unsolicited audio',()=>{
  const listeners=new Map(),received=[],posts=[];
  const location={href:'app://local/new-tab.html'};
  const window={addEventListener:(t,f)=>listeners.set(t,f),dispatchEvent:e=>received.push(e),__sparkPostContent:u=>posts.push(u)};window.top=window;
  const document={addEventListener(){}};
  const send=operation=>listeners.get('spark-new-tab-action')({detail:{action:'widgets',operation,data:{}}});
  runInNewContext(readFileSync('src-tauri/src/window/new_tab/bridge.js','utf8'),{window,document,location,crypto:{randomUUID:()=> 'doc-a'},URLSearchParams,
    __NEW_TAB_URL__:location.href,__NEW_TAB_TOKEN__:'artificial-token',__NEW_TAB_ID__:'tab-a',CustomEvent:class{constructor(type,o){this.type=type;this.detail=o.detail;}}});
  const payload={generation:2,sequence:3,status:'ready',bands:Array(32).fill(0.5)};
  window.__sparkAudioFrame('doc-a',payload);assert.equal(received.length,0);
  send('audio_subscribe');window.__sparkAudioFrame('wrong',payload);window.__sparkAudioFrame('doc-a',payload);
  assert.equal(received.length,1);
  window.__sparkAudioFrame('doc-a',payload);window.__sparkAudioFrame('doc-a',{...payload,generation:1,sequence:100});
  window.__sparkAudioFrame('doc-a',{...payload,sequence:4,bands:Array(32).fill(NaN)});assert.equal(received.length,1);
  window.__sparkAudioFrame('doc-a',{...payload,generation:3,sequence:1});assert.equal(received.length,2);
  send('audio_unsubscribe');window.__sparkAudioFrame('doc-a',{...payload,generation:4});assert.equal(received.length,2);
  assert.equal(posts.length,2);
});
