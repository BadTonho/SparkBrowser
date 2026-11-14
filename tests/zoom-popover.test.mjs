import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

function fixture() {
  const listeners=new Map(),frames=new Map(),sent=[];
  let frameId=0;
  const elements=new Map();
  const document={documentElement:{},getElementById:id=>elements.get(id),addEventListener:(type,fn)=>listeners.set('document:'+type,fn)};
  for(const id of ['zoom-popover','page-zoom','zoom-percentage','zoom-out','zoom-in','zoom-reset','zoom-error'])elements.set(id,{
    id,value:'7',disabled:false,hidden:true,offsetWidth:240,offsetHeight:130,attributes:{},style:{setProperty(){}},
    setAttribute(name,value){this.attributes[name]=value;},addEventListener(type,fn){listeners.set(id+':'+type,fn);},
    contains(target){return [...elements.values()].includes(target);},focus(){document.activeElement=this;},
    querySelectorAll(){return ['zoom-out','zoom-in','page-zoom','zoom-reset'].map(id=>elements.get(id)).filter(e=>!e.disabled);},
  });
  const context={document,innerWidth:800,innerHeight:600,URL,Intl,
    requestAnimationFrame:fn=>{frames.set(++frameId,fn);return frameId;},cancelAnimationFrame:id=>frames.delete(id),
    addEventListener:(type,fn)=>listeners.set('window:'+type,fn),location:{set href(value){sent.push(new URL(value));}},
  };context.window=context;
  runInNewContext(readFileSync('public/overlays/zoom-popover.js','utf8'),context);
  const initial={visible:true,token:'artificial-zoom-token',factor:1,levels:[.25,.5,.75,1,1.1,1.25,1.5,2,3,4,5],pending:false,error:'',locale:'en',label:'Page zoom',inLabel:'Increase',outLabel:'Decrease',resetLabel:'Reset 100%',anchor:{left:400,top:10,width:25,height:25},theme:{}};
  const update=patch=>context.__sparkSetZoomPopover({...initial,...patch});
  const fire=(id,type,event={})=>listeners.get(id+':'+type)?.(event);
  const key=(key,extra={})=>fire('window','keydown',{key,preventDefault(){},stopImmediatePropagation(){},...extra});
  const frame=()=>{const callbacks=[...frames.values()];frames.clear();callbacks.forEach(fn=>fn());};
  return {elements,document,sent,update,fire,key,frame};
}

test('zoom panel anchors below its icon and preserves nonstandard saved factors on opening',()=>{
  const f=fixture();f.update({factor:1.37});
  assert.equal(f.elements.get('zoom-popover').style.top,'41px');
  assert.equal(f.elements.get('zoom-popover').style.left,'292.5px');
  assert.equal(f.elements.get('zoom-percentage').textContent,'137%');
  assert.equal(f.document.activeElement,f.elements.get('page-zoom'));
  assert.equal(f.sent.length,0);
  f.update({anchor:{left:790,top:10,width:25,height:25}});
  assert.equal(f.elements.get('zoom-popover').style.left,'552px');
});

test('slider coalesces input, ignores intermediate replies and flushes its final value on dismissal',()=>{
  const f=fixture();f.update();f.fire('page-zoom','pointerdown');
  for(const index of [5,7,10]){f.elements.get('page-zoom').value=String(index);f.fire('page-zoom','input');}
  assert.equal(f.elements.get('zoom-percentage').textContent,'500%');
  f.update({factor:1.25,pending:true});
  assert.equal(f.elements.get('zoom-percentage').textContent,'500%');
  f.frame();assert.equal(f.sent[0].searchParams.get('value'),'5');
  f.elements.get('page-zoom').value='6';f.fire('page-zoom','input');f.key('Escape');
  assert.equal(f.sent.at(-1).pathname,'/dismiss');
  assert.equal(f.sent.at(-1).searchParams.get('value'),'1.5');
  f.update({factor:1.5});f.frame();
  assert.equal(f.sent.length,2,'late updates cannot revive a closing panel');
});

test('buttons and keyboard follow levels, failures restore confirmed values and Tab exits only at a boundary',()=>{
  const f=fixture();f.update({factor:1.37});f.fire('zoom-in','click');
  assert.equal(f.sent.at(-1).searchParams.get('value'),'1.5');
  f.update({factor:1.37,error:'Artificial failure'});
  assert.equal(f.elements.get('zoom-percentage').textContent,'137%');
  assert.equal(f.elements.get('zoom-error').hidden,false);
  f.key('0',{ctrlKey:true});assert.equal(f.sent.at(-1).searchParams.get('value'),'1');
  f.document.activeElement=f.elements.get('page-zoom');f.key('Tab');
  assert.equal(f.sent.length,2);
  f.document.activeElement=f.elements.get('zoom-reset');f.key('Tab');
  assert.equal(f.sent.at(-1).searchParams.get('reason'),'tab');
});
