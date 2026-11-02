import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';
import {test} from 'node:test';
import assert from 'node:assert/strict';

const code=readFileSync(new URL('../src-tauri/src/media/pip_player.js',import.meta.url),'utf8')
  .replace('__PIP_PLAYER_STYLE__',JSON.stringify(readFileSync(new URL('../src-tauri/src/media/player/style.css',import.meta.url),'utf8')))
  .replace('__PIP_ICONS__',readFileSync(new URL('../src-tauri/src/media/player/icons.js',import.meta.url),'utf8'))
  // The real stream lifecycle has its own regression suite; these tests isolate UI.
  .replace('__PIP_CAPTURE__','(popup,source,image,stream)=>{image.srcObject=stream;return ()=>{stream.getTracks().forEach(t=>t.stop());image.srcObject=null;};}');
function fixture() {
  const elements=[],timeouts=new Map(),intervals=new Map(),sent=[];
  let next=0,stopped=0,closed=0,now=0;
  const deadlines=new Map();
  const eventTarget=()=>({listeners:new Map(),addEventListener(name,callback){this.listeners.set(name,callback);},removeEventListener(name){this.listeners.delete(name);},fire(name,event={}){this.listeners.get(name)?.(event);}});
  const doc={...eventTarget(),documentElement:{},createElement(tag){const node={...eventTarget(),tag,children:[],attributes:{},style:{setProperty(name,value){this[name]=value;}},value:'',append(child){child.parent=this;this.children.push(child);},replaceChildren(...nodes){this.children=[];nodes.forEach(n=>this.append(n));},getAttribute(name){return this.attributes[name];},setAttribute(name,value){this.attributes[name]=value;},contains(child){return child===this||this.children.some(n=>n.contains(child));},focus(){doc.activeElement=this;let n=this;while(n){n.fire('focusin');n=n.parent;}},blur(){doc.activeElement=null;doc.fire('focusout');},play(){return Promise.resolve();}};elements.push(node);return node;}};
  doc.createElementNS=(_,tag)=>doc.createElement(tag);
  doc.body=doc.createElement('body');
  doc.body.parent=doc;
  const popup={...eventTarget(),document:doc,CSSStyleSheet:class{replaceSync(){} }};
  const source={...eventTarget(),currentTime:30,duration:120,paused:false,isConnected:true,seekable:{length:1,start:()=>0,end:()=>120},play(){this.paused=false;return Promise.resolve();},pause(){this.paused=true;}};
  const stream={getTracks:()=>[{stop(){stopped++;}}]};
  const clock={setTimeout:(callback,delay)=>{timeouts.set(++next,callback);deadlines.set(next,now+delay);return next;},clearTimeout:id=>{timeouts.delete(id);deadlines.delete(id);},setInterval:callback=>{intervals.set(++next,callback);return next;},clearInterval:id=>intervals.delete(id)};
  Object.assign(popup,clock);
  const create=runInNewContext(`(${code})`,{});
  const cleanup=create(popup,source,stream,{session:'test',hostname:'example.test',volume:100,presentation:{locale:'pt-BR',labels:{volume:'Volume',minimize:'Minimizar'}}},(action,value)=>{sent.push({action,value});return sent.length;},()=>closed++);
  const find=id=>elements.find(node=>node.attributes['data-testid']===id||node.id===id);
  return {doc,popup,source,cleanup,sent,find,timeouts,intervals,stats:()=>({stopped,closed}),tick(){const callbacks=[...timeouts.values()];timeouts.clear();deadlines.clear();callbacks.forEach(callback=>callback());},advance(ms){const end=now+ms;while(true){const due=[...deadlines].filter(([,at])=>at<=end).sort((a,b)=>a[1]-b[1])[0];if(!due)break;const [id,at]=due;now=at;const callback=timeouts.get(id);timeouts.delete(id);deadlines.delete(id);callback?.();}now=end;}};
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
  for(const value of [0,100,200]){slider.value=String(value);slider.fire('input');assert.equal(percent.textContent,`${value}%`);assert.equal(slider.style['--volume-progress'],`${value/2}%`);}
  f.popup.__velocePipState({session:'wrong',sequence:99,volume:20});assert.equal(slider.value,'200');
  f.popup.__velocePipState({session:'test',sequence:1,volume:20});assert.equal(slider.value,'200');
  f.popup.__velocePipState({session:'test',sequence:f.sent.length,volume:100});assert.equal(slider.value,'100');assert.equal(percent.textContent,'100%');assert.equal(slider.style['--volume-progress'],'50%');
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

test('minimal controls hide after two seconds and reappear on pointer or keyboard activity',()=>{
  const f=fixture(),chrome=f.find('pip-chrome');
  f.advance(1999);assert.equal(chrome.getAttribute('data-visible'),'true');
  f.advance(1);assert.equal(chrome.getAttribute('data-visible'),'false');
  f.doc.fire('pointermove');assert.equal(chrome.getAttribute('data-visible'),'true');
  f.advance(2000);assert.equal(chrome.getAttribute('data-visible'),'false');
  f.doc.fire('keydown',{key:'Tab'});assert.equal(chrome.getAttribute('data-visible'),'true');
  f.find('pip-play').focus();f.advance(4000);assert.equal(chrome.getAttribute('data-visible'),'true');
  f.find('pip-play').blur();f.advance(2000);assert.equal(chrome.getAttribute('data-visible'),'false');
});

test('pause, seek drag, window drag and the volume panel keep minimal controls visible',()=>{
  const f=fixture(),chrome=f.find('pip-chrome');
  f.source.paused=true;f.source.fire('pause');f.advance(4000);assert.equal(chrome.getAttribute('data-visible'),'true');
  f.source.paused=false;f.source.fire('play');f.advance(2000);assert.equal(chrome.getAttribute('data-visible'),'false');
  f.find('pip-progress').fire('pointerdown');f.advance(4000);assert.equal(chrome.getAttribute('data-visible'),'true');
  f.popup.fire('pointerup');f.advance(2000);assert.equal(chrome.getAttribute('data-visible'),'false');
  f.find('pip-drag').fire('pointerdown',{button:0,preventDefault(){}});assert.equal(f.sent.at(-1).action,'drag');
  f.advance(4000);assert.equal(chrome.getAttribute('data-visible'),'true');
  f.popup.__velocePipDragFinished();f.advance(2000);assert.equal(chrome.getAttribute('data-visible'),'false');
  f.find('pip-speaker').parent.fire('pointerenter');f.advance(4000);assert.equal(chrome.getAttribute('data-visible'),'true');
});

test('header displays the source domain and uses accessible SVG buttons for window actions',()=>{
  const f=fixture();assert.equal(f.find('pip-origin').textContent,'example.test');
  for(const id of ['pip-play','pip-back','pip-forward','pip-speaker','pip-return','pip-minimize','pip-close']){
    const button=f.find(id);assert.equal(button.children[0].tag,'svg');assert.ok(button.getAttribute('aria-label'));
  }
  assert.equal(f.find('pip-minimize').getAttribute('aria-label'),'Minimizar');
  f.find('pip-minimize').fire('click');assert.equal(f.sent.at(-1).action,'minimize');
});
