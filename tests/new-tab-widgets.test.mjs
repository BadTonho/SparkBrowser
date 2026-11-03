import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';

function fixture(){
  const listeners=new Map(),actions=[],nodes=new Map();
  const document={activeElement:null,getElementById:id=>nodes.get(id),querySelectorAll:()=>[]};
  class Node{
    constructor(){this.children=[];this.listeners={};this.value='';this.dataset={};this.attributes={};this.hidden=false;this.open=false;this.classList={contains:()=>false};}
    addEventListener(t,fn){this.listeners[t]=fn;}
    setAttribute(k,v){this.attributes[k]=v;}
    append(...items){this.children.push(...items);}
    replaceChildren(){this.children=[];}
    focus(){document.activeElement=this;}
    click(){this.listeners.click?.({target:this});}
    close(){this.open=false;this.listeners.close?.();}
    showModal(){this.open=true;}
    getBoundingClientRect(){return {left:300,right:680,top:0,bottom:600};}
  }
  for(const [,id] of readFileSync('public/new-tab.html','utf8').matchAll(/id="([^"]+)"/g))nodes.set(id,new Node());
  document.createElement=()=>new Node();
  const window={addEventListener:(t,fn)=>listeners.set(t,fn),dispatchEvent:e=>actions.push(e.detail)};
  runInNewContext(readFileSync('public/new-tab/widgets.js','utf8'),{window,document,Intl,Date,CustomEvent:class{constructor(t,o){this.type=t;this.detail=o.detail;}}});
  listeners.get('veloce-new-tab')({detail:{labels:JSON.parse(readFileSync('src/lib/i18n/catalogs/en/newtab.json','utf8')),preferences:{language:'en'}}});
  const settings={weather_enabled:false,gmail_enabled:false,automatic_location:false,fahrenheit:false,city:null};
  let current={settings,revision:0,weather:{data:null,error:null,busy:false},gmail:{data:null,email:null,error:null,busy:false,connecting:false,configured:true}};
  const update=patch=>{current={...current,...patch};listeners.get('veloce-new-tab-widgets')({detail:current});};update({});
  return {nodes,actions,update,settings,document};
}
const change=(node,key,value)=>{node[key]=value;node.listeners.change({target:node});};
test('widget settings use a focus-trapped dialog, close outside/Escape and preserve drafts',()=>{
  const f=fixture(),gear=f.nodes.get('widget-settings'),panel=f.nodes.get('settings-panel');
  f.nodes.get('search').value='unfinished';gear.click();
  assert.equal(panel.open,true);assert.equal(f.document.activeElement,f.nodes.get('settings-close'));
  assert.equal(f.actions.at(-1).operation,'panel');
  panel.listeners.cancel({preventDefault(){}});assert.equal(panel.open,false);assert.equal(f.document.activeElement,gear);
  gear.click();panel.listeners.click({target:panel,clientX:10,clientY:20});assert.equal(panel.open,false);
  assert.equal(f.nodes.get('search').value,'unfinished');
});
test('rapid settings edits serialize and retain the last requested value, failures restore confirmed state',()=>{
  const f=fixture();change(f.nodes.get('weather-enabled'),'checked',true);change(f.nodes.get('temperature-unit'),'value','fahrenheit');
  assert.equal(f.actions.filter(a=>a.operation==='save').length,1);
  f.update({settings:{...f.settings,weather_enabled:true},revision:1});
  assert.equal(f.actions.filter(a=>a.operation==='save').length,2);assert.equal(f.actions.at(-1).data.settings.fahrenheit,true);
  f.update({settings:{...f.settings,weather_enabled:true,fahrenheit:true},revision:2});
  assert.equal(f.nodes.get('temperature-unit').value,'fahrenheit');
  change(f.nodes.get('weather-enabled'),'checked',false);
  f.update({action_error:'settings_unavailable',operation:'save'});
  assert.equal(f.nodes.get('weather-enabled').checked,true);assert.match(f.nodes.get('settings-status').textContent,/save settings/);
});
test('weather converts units locally and Gmail renders only safe text with independent status',()=>{
  const f=fixture();
  const data={temperature:20,feels_like:19,code:0,updated:1000,days:[{date:'2026-11-03',min:10,max:20,code:61}]};
  f.update({settings:{...f.settings,weather_enabled:true,gmail_enabled:true,city:{name:'Test',region:'Region',country:'Country'},fahrenheit:true},weather:{data,error:null,busy:false},gmail:{configured:true,email:'test@example.test',data:{unread:2,updated:1000,messages:[{sender:'<img src=x>',subject:'<script>alert(1)</script>',date:1000,unread:true}]}}});
  assert.equal(f.nodes.get('weather-content').children[0].children[1].textContent,'68°F');
  assert.equal(f.nodes.get('weather-content').children[0].children[0].src,'/new-tab/weather/clear.svg');
  const message=f.nodes.get('gmail-messages').children[0];assert.equal(message.children[0].textContent,'<img src=x>');assert.equal(message.children[1].textContent,'<script>alert(1)</script>');
  const previous=f.nodes.get('gmail-messages').children[0];f.update({});assert.equal(f.nodes.get('gmail-messages').children[0],previous);
  f.nodes.get('gmail-open').click();assert.equal(f.actions.at(-1).operation,'open_gmail');
});
test('city results never overwrite a new query and selecting a city persists its explicit coordinates',()=>{
  const f=fixture(),query=f.nodes.get('city-query');query.value='Springfield';f.nodes.get('city-search').click();
  query.value='Porto';query.listeners.input();
  const city={name:'Springfield',region:'Illinois',country:'US',latitude:39,longitude:-89};
  f.update({cities:[city],city_query:'Springfield'});assert.equal(f.nodes.get('city-results').children.length,0);
  f.update({cities:[{...city,name:'Porto',country:'Portugal'}],city_query:'Porto'});
  assert.equal(f.nodes.get('city-results').children[0].textContent,'Porto, Illinois, Portugal');f.nodes.get('city-results').children[0].click();
  assert.equal(f.actions.at(-1).operation,'save');assert.equal(f.actions.at(-1).data.settings.city.name,'Porto');
});
