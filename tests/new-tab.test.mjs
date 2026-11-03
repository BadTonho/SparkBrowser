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
    getAttribute(k){return k==='src'?this.src:this.attributes[k];}
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
  const update=patch=>listeners.get('veloce-new-tab')({detail:{labels,preferences:{language:'en',engine:'google'},sites:[],error:false,autofocus:true,...patch}});
  return {document,nodes,actions,update};
}

test('new tab submits only nonempty input and opens recent sites with safe text and local icons',()=>{
  const f=fixture();const input=f.nodes.get('#search');
  f.update({sites:[{site_key:'example.test',domain:'example.test',title:'<img src=x onerror=alert(1)>',url:'https://example.test/last?q=a%26b',icon:'https://remote.test/icon.png'}]});
  assert.equal(f.document.activeElement,input);
  const card=f.nodes.get('#recent').children[0];
  assert.equal(card.children[0].src,'/new-tab/site-fallback.svg');
  assert.equal(card.children[0].width,24);
  assert.equal(card.children[1].textContent,'example.test');
  assert.equal(card.children[2].textContent,'<img src=x onerror=alert(1)>');
  card.listeners.click();assert.equal(f.actions[0].action,'visit');assert.equal(f.actions[0].site_key,'example.test');assert.equal(f.actions[0].input,'https://example.test/last?q=a%26b');
  input.value='   ';f.nodes.get('#search-form').listeners.submit({preventDefault(){}});assert.equal(f.actions.length,1);
  input.value='  rust & svelte  ';f.nodes.get('#search-form').listeners.submit({preventDefault(){}});
  assert.equal(f.actions.at(-1).action,'navigate'); assert.equal(f.actions.at(-1).input,'rust & svelte');
});

test('history refresh and errors preserve draft and focus, clear cards, and allow retry',()=>{
  const f=fixture();const input=f.nodes.get('#search'),recent=f.nodes.get('#recent');
  const sites=[{site_key:'one.test',domain:'one.test',title:'One',url:'https://one.test'}];f.update({sites});
  input.value='unfinished input';recent.children[0].focus();const original=recent.children[0];
  f.update({sites});assert.equal(recent.children[0],original);assert.equal(f.document.activeElement,original);
  sites[0].icon='data:image/png;base64,aGVsbG8=';f.update({sites});
  const image=recent.children[0].children[0];assert.equal(image.src,sites[0].icon);assert.equal(f.document.activeElement,recent.children[0]);
  image.listeners.error();assert.equal(image.src,'/new-tab/site-fallback.svg');
  f.update({error:true,sites:null});assert.equal(input.value,'unfinished input');assert.equal(f.nodes.get('#error').hidden,false);
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
  listeners.get('veloce-new-tab-action')({detail:{action:'visit',input:'https://one.test/a?q=a&b',site_key:'one.test'}});
  assert.deepEqual(sent.map(url=>url.searchParams.get('sequence')),['1','2','3']);
  assert.equal(sent.at(-1).searchParams.get('input'),'https://one.test/a?q=a&b');assert.equal(sent.at(-1).pathname,'/visit');
  listeners.get('pageshow')({persisted:true});
  assert.equal(sent.at(-1).pathname,'/read');
  assert.equal(sent.at(-1).searchParams.get('request'),'3');
  window.__veloceNewTabReply('test-document',2,{old:true});assert.equal(received.length,1);
});

test('widget replies reject old documents, old operations and snapshots predating disconnect',()=>{
  const listeners=new Map(),received=[],sent=[];
  const location={get href(){return 'http://tauri.localhost/new-tab.html';},set href(value){sent.push(new URL(value));}};
  const window={addEventListener:(type,fn)=>listeners.set(type,fn),dispatchEvent:e=>received.push(e.detail)};window.top=window;
  const document={addEventListener:(type,fn)=>listeners.set(type,fn)};
  const source=readFileSync('src-tauri/src/window/new_tab/bridge.js','utf8').replaceAll('__NEW_TAB_URL__',JSON.stringify(location.href)).replaceAll('__NEW_TAB_TOKEN__',JSON.stringify('artificial-token')).replaceAll('__NEW_TAB_ID__',JSON.stringify('tab-1'));
  runInNewContext(source,{window,document,location,URLSearchParams,crypto:{randomUUID:()=> 'current-document'},CustomEvent:class{constructor(type,init){this.type=type;this.detail=init.detail;}}});
  window.__veloceRefreshWidgets();window.__veloceRefreshWidgets();
  window.__veloceWidgetsReply('current-document',1,{version:1});window.__veloceWidgetsReply('old-document',2,{version:2});assert.equal(received.length,0);
  window.__veloceWidgetsReply('current-document',2,{version:3,connected:true});assert.equal(received.length,1);
  listeners.get('veloce-new-tab-action')({detail:{action:'widgets',operation:'disconnect',data:{}}});
  window.__veloceWidgetsReply('current-document',3,{version:5,connected:false});
  window.__veloceWidgetsReply('current-document',2,{version:4,connected:true});
  assert.equal(received.length,2);assert.equal(received.at(-1).connected,false);
  assert.equal(sent.at(-1).pathname,'/widgets');assert.equal(sent.at(-1).searchParams.get('operation'),'disconnect');
});
