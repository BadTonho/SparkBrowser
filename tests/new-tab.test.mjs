import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';

function fixture() {
  const listeners=new Map(), actions=[];
  const document={activeElement:null,documentElement:{lang:'en'}};
  class Node {
    constructor(){this.children=[];this.dataset={};this.value='';this.hidden=true;this.attributes={};}
    addEventListener(type,fn){this.listeners??={};this.listeners[type]=fn;}
    setAttribute(k,v){this.attributes[k]=v;}
    replaceChildren(){this.children=[];}
    append(...nodes){this.children.push(...nodes);}
    contains(node){return this.children.includes(node);}
    focus(){document.activeElement=this;}
  }
  const nodes=new Map(['#search','#recent','#search-form','#submit','#empty','#error-text','#error','#retry','.content'].map(id=>[id,new Node()]));
  document.querySelector=id=>nodes.get(id);document.createElement=()=>new Node();
  const window={addEventListener:(t,fn)=>listeners.set(t,fn),dispatchEvent:event=>actions.push(event.detail)};
  runInNewContext(readFileSync('public/new-tab/page.js','utf8'),{window,document,CustomEvent:class {constructor(type,init){this.type=type;this.detail=init.detail;}}});
  const labels=JSON.parse(readFileSync('src/lib/i18n/catalogs/en/newtab.json','utf8'));
  const update=patch=>listeners.get('veloce-new-tab')({detail:{labels,preferences:{language:'en',engine:'google'},searches:[],error:false,autofocus:true,...patch}});
  return {document,nodes,actions,update};
}

test('new tab submits only nonempty input and repeats cards with their own engine as plain text',()=>{
  const f=fixture();const input=f.nodes.get('#search');
  f.update({searches:[{query:'<img src=x onerror=alert(1)>',engine:'duckduckgo'}]});
  assert.equal(f.document.activeElement,input);
  const card=f.nodes.get('#recent').children[0];
  assert.equal(card.children[0].textContent,'<img src=x onerror=alert(1)>');
  assert.equal(card.children[1].textContent,'DuckDuckGo');
  card.listeners.click();assert.equal(f.actions[0].engine,'duckduckgo');
  input.value='   ';f.nodes.get('#search-form').listeners.submit({preventDefault(){}});assert.equal(f.actions.length,1);
  input.value='  rust & svelte  ';f.nodes.get('#search-form').listeners.submit({preventDefault(){}});
  assert.equal(f.actions.at(-1).action,'navigate'); assert.equal(f.actions.at(-1).input,'rust & svelte');
});

test('history refresh and errors preserve draft and focus, clear cards, and allow retry',()=>{
  const f=fixture();const input=f.nodes.get('#search'),recent=f.nodes.get('#recent');
  const searches=[{query:'one',engine:'google'}];f.update({searches});
  input.value='unfinished input';recent.children[0].focus();const original=recent.children[0];
  f.update({searches});assert.equal(recent.children[0],original);assert.equal(f.document.activeElement,original);
  f.update({error:true,searches:null});assert.equal(input.value,'unfinished input');assert.equal(f.nodes.get('#error').hidden,false);
  f.nodes.get('#retry').listeners.click();assert.equal(f.actions.at(-1).action,'read');
  f.update({autofocus:false});assert.equal(f.nodes.get('#empty').hidden,false);assert.equal(recent.children.length,0);
});

test('new tab bridge rejects delayed document and request replies and captures monotonic actions',()=>{
  const listeners=new Map(),received=[],sent=[];
  const location={get href(){return 'http://tauri.localhost/new-tab.html';},set href(value){sent.push(new URL(value));}};
  const window={addEventListener:(type,fn)=>listeners.set(type,fn),dispatchEvent:e=>received.push(e.detail)};window.top=window;
  const document={addEventListener:(type,fn)=>listeners.set(type,fn)};
  const source=readFileSync('src-tauri/src/window/new_tab/bridge.js','utf8').replaceAll('__NEW_TAB_URL__',JSON.stringify(location.href)).replaceAll('__NEW_TAB_TOKEN__',JSON.stringify('artificial-token')).replaceAll('__NEW_TAB_ID__',JSON.stringify('tab-1'));
  runInNewContext(source,{window,document,location,URLSearchParams,crypto:{randomUUID:()=> 'test-document'},CustomEvent:class{constructor(type,init){this.type=type;this.detail=init.detail;}}});
  listeners.get('DOMContentLoaded')();window.__veloceRefreshNewTab();
  window.__veloceNewTabReply('test-document',1,{old:true});window.__veloceNewTabReply('old-document',2,{old:true});assert.equal(received.length,0);
  window.__veloceNewTabReply('test-document',2,{current:true});assert.equal(received.length,1);
  listeners.get('veloce-new-tab-action')({detail:{action:'navigate',input:'a & b',engine:'duckduckgo'}});
  assert.deepEqual(sent.map(url=>url.searchParams.get('sequence')),['1','2','3']);
  assert.equal(sent.at(-1).searchParams.get('input'),'a & b');
  listeners.get('pageshow')({persisted:true});
  assert.equal(sent.at(-1).pathname,'/read');
  assert.equal(sent.at(-1).searchParams.get('request'),'3');
  window.__veloceNewTabReply('test-document',2,{old:true});assert.equal(received.length,1);
});
