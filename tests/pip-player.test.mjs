import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';
import {test} from 'node:test';
import assert from 'node:assert/strict';

const code=readFileSync(new URL('../src-tauri/src/media/pip_player.js',import.meta.url),'utf8');
function fixture() {
  const elements=[],timeouts=new Map(),intervals=new Map(),sent=[];
  let next=0,stopped=0,closed=0;
  const eventTarget=()=>({listeners:new Map(),addEventListener(name,callback){this.listeners.set(name,callback);},removeEventListener(name){this.listeners.delete(name);},fire(name,event={}){this.listeners.get(name)?.(event);}});
  const doc={...eventTarget(),documentElement:{},createElement(tag){const node={...eventTarget(),tag,children:[],attributes:{},value:'',append(child){child.parent=this;this.children.push(child);},setAttribute(name,value){this.attributes[name]=value;},contains(child){return child===this||this.children.some(n=>n.contains(child));},focus(){doc.activeElement=this;let n=this;while(n){n.fire('focusin');n=n.parent;}},play(){return Promise.resolve();}};elements.push(node);return node;}};
  doc.body=doc.createElement('body');
  const popup={...eventTarget(),document:doc,CSSStyleSheet:class{replaceSync(){} }};
  const source={...eventTarget(),currentTime:30,duration:120,paused:false,isConnected:true,seekable:{length:1,start:()=>0,end:()=>120},play(){this.paused=false;return Promise.resolve();},pause(){this.paused=true;}};
  const stream={getTracks:()=>[{stop(){stopped++;}}]};
  const clock={setTimeout:callback=>{timeouts.set(++next,callback);return next;},clearTimeout:id=>timeouts.delete(id),setInterval:callback=>{intervals.set(++next,callback);return next;},clearInterval:id=>intervals.delete(id)};
  Object.assign(popup,clock);
  const create=runInNewContext(`(${code})`,{});
  const cleanup=create(popup,source,stream,{session:'test',volume:100,presentation:{locale:'pt-BR',labels:{volume:'Volume'}}},(action,value)=>{sent.push({action,value});return sent.length;},()=>closed++);
  const find=id=>elements.find(node=>node.attributes['data-testid']===id||node.id===id);
  return {doc,popup,source,cleanup,sent,find,timeouts,intervals,stats:()=>({stopped,closed}),tick(){const callbacks=[...timeouts.values()];timeouts.clear();callbacks.forEach(callback=>callback());}};
}

test('floating volume opens on hover, focus and click; hides after leaving and preserves dragging',()=>{
  const f=fixture(),speaker=f.find('pip-speaker'),area=speaker.parent,panel=f.find('pip-volume-panel'),slider=f.find('pip-volume');
  assert.equal(panel.hidden,true);
  area.fire('pointerenter');assert.equal(panel.hidden,false);
  area.fire('pointerleave');assert.equal(panel.hidden,false);f.tick();assert.equal(panel.hidden,true);
  speaker.focus();assert.equal(panel.hidden,false);
  f.doc.activeElement=null;area.fire('pointerleave');slider.fire('pointerdown');f.tick();assert.equal(panel.hidden,false);
  f.popup.fire('pointerup');f.tick();assert.equal(panel.hidden,true);
  speaker.fire('click');assert.equal(panel.hidden,false);
});

test('percentage is immediate at 0, 100 and 200 and stale acknowledgments cannot overwrite edits',()=>{
  const f=fixture(),slider=f.find('pip-volume'),percent=f.find('pip-volume-percentage');
  for(const value of [0,100,200]){slider.value=String(value);slider.fire('input');assert.equal(percent.textContent,`${value}%`);}
  f.popup.__velocePipState({session:'wrong',sequence:99,volume:20});assert.equal(slider.value,'200');
  f.popup.__velocePipState({session:'test',sequence:1,volume:20});assert.equal(slider.value,'200');
  f.popup.__velocePipState({session:'test',sequence:f.sent.length,volume:100});assert.equal(slider.value,'100');assert.equal(percent.textContent,'100%');
  slider.fire('pointerdown');slider.value='51';slider.fire('input');
  f.popup.__velocePipState({session:'test',sequence:99,volume:100});assert.equal(slider.value,'51');
  f.popup.fire('pointerup');assert.deepEqual(f.sent.at(-1),{action:'flush',value:51});
  f.popup.__velocePipState({session:'test',sequence:99,volume:50});assert.equal(slider.value,'50');
  f.popup.__velocePipState({session:'test',sequence:99,volume:50,disabled:true});
  const count=f.sent.length;slider.fire('input');assert.equal(f.sent.length,count);
});

test('Escape hides volume and focuses its trigger without reopening; media controls act on the source',()=>{
  const f=fixture(),speaker=f.find('pip-speaker');speaker.focus();
  f.doc.fire('keydown',{key:'Escape',preventDefault(){},stopPropagation(){}});
  assert.equal(f.find('pip-volume-panel').hidden,true);assert.equal(f.doc.activeElement,speaker);
  f.find('pip-back').fire('click');assert.equal(f.source.currentTime,20);
  f.find('pip-forward').fire('click');assert.equal(f.source.currentTime,30);
  f.find('pip-play').fire('click');assert.equal(f.source.paused,true);
  f.find('pip-return').fire('click');assert.equal(f.sent.at(-1).action,'return');
  f.find('pip-close').fire('click');assert.deepEqual(f.sent.at(-1),{action:'close',value:100});
});

test('closing stops only the image copy, removes source listeners and timers, and keeps playback running',()=>{
  const f=fixture();f.cleanup();f.cleanup();
  assert.deepEqual(f.stats(),{stopped:1,closed:1});assert.equal(f.source.paused,false);
  assert.equal(f.source.listeners.size,0);assert.equal(f.intervals.size,0);assert.equal(f.timeouts.size,0);
  assert.equal(f.find('pip-image').srcObject,null);
});
