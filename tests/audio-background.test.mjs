import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';

function fixture(reduced=false){
  const events=new Map(),nodes=new Map(),actions=[],frames=new Map(),timers=new Map(),rects=[];
  let counter=0;
  const listen=(name,fn)=>{const previous=events.get(name);events.set(name,event=>{previous?.(event);fn(event);});};
  const context={clearRect(){rects.length=0;},fillRect(...r){rects.push(r);},createLinearGradient(){return {addColorStop(){}};}};
  for(const id of ['audio-background','audio-background-enabled','audio-background-options','audio-background-color','audio-background-intensity','audio-background-opacity','audio-background-intensity-value','audio-background-opacity-value','audio-background-style','audio-background-settings','audio-background-status'])nodes.set(id,{hidden:true,listeners:{},addEventListener(name,fn){this.listeners[name]=fn;},getContext(){return context;}});
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
test('every visual style draws finite shapes from real levels and the selector is saved',()=>{
  const source=readFileSync('public/new-tab/audio-styles.js','utf8');
  const calls=[];
  const handler={get:(target,name)=>name in target?target[name]:(...args)=>{if(args.some(v=>typeof v==='number'&&!Number.isFinite(v)))throw new Error(`${String(name)} got a non-finite value`);calls.push(String(name));return {addColorStop(){}};},set:(target,name,value)=>{target[name]=value;return true;}};
  const window={};runInNewContext(source,{window,Math:Object.create(Math,{random:{value:()=>0.1}}),Float32Array,Array,Object,Number});
  const ids=Object.keys(window.__sparkAudioStyles);
  const rust=readFileSync('src-tauri/src/database/new_tab_repo.rs','utf8');
  for(const id of ['bars',...ids])assert.match(rust,new RegExp(`"${id}"`),`${id} is accepted by the backend`);
  assert.equal(ids.length,19);
  const levels=Array.from({length:32},(_,i)=>i/31);
  for(const id of ids){
    const style=window.__sparkAudioStyles[id],store={};
    assert.ok(['band','full'].includes(style.layout));
    for(const gain of [Math.SQRT1_2,1,Math.SQRT2])for(let frame=0;frame<3;frame++){calls.length=0;style.draw(new Proxy({},handler),1000,400,levels,{color:'#1db954',gain,t:frame,dt:0.033,store});assert.ok(calls.length>0,`${id} draws`);}
  }
  const f=fixture();f.render({style:'wave'});
  f.nodes.get('audio-background-style').listeners.change({target:{value:'radial'}});
  assert.equal(JSON.stringify(f.actions.at(-1)),'{"style":"radial"}');
});

test('the intensity gain makes the shapes themselves bigger, not only more opaque',()=>{
  const source=readFileSync('public/new-tab/audio-styles.js','utf8');
  const window={};runInNewContext(source,{window,Math,Float32Array,Array,Object,Number});
  const reach=gain=>{
    let far=0;const noop=()=>{};
    const ctx=new Proxy({},{get:(_,name)=>name==='lineTo'?(x,y)=>{far=Math.max(far,Math.hypot(x-500,y-200));}:noop,set:()=>true});
    window.__sparkAudioStyles.radial.draw(ctx,1000,400,Array(32).fill(0.5),{color:'#1db954',gain,t:0,dt:0.033,store:{}});
    return far;
  };
  assert.ok(reach(2)>reach(1)&&reach(1)>reach(0.5));
});

test('the effect has its own opacity slider, saved apart from the intensity one',()=>{
  const f=fixture();f.render({opacity:0.4});
  assert.equal(f.nodes.get('audio-background-opacity').value,'0.4');
  f.nodes.get('audio-background-opacity').listeners.change({target:{value:'0.7'}});
  assert.equal(JSON.stringify(f.actions.at(-1)),'{"opacity":0.7}');
  const picker=readFileSync('public/new-tab/audio-style-picker.js','utf8');
  assert.match(picker,/gain:\s*1/,'the style previews pass a gain');
});

test('both sliders show their value as a percentage, 100% being where the slider used to end',()=>{
  const f=fixture();f.render({intensity:2,opacity:0.4});
  assert.equal(f.nodes.get('audio-background-intensity-value').textContent,'100%');
  assert.equal(f.nodes.get('audio-background-opacity-value').textContent,'40%');
  f.nodes.get('audio-background-intensity').listeners.input({target:{value:'4'}});
  assert.equal(f.nodes.get('audio-background-intensity-value').textContent,'200%');
});
