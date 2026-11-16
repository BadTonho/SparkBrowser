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
    removeAttribute(k){delete this.attributes[k];if(k==='src')this.src='';}
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
  const window={addEventListener:(t,fn)=>{const previous=listeners.get(t);listeners.set(t,e=>{previous?.(e);fn(e);});},dispatchEvent:e=>{if(e.type==='spark-new-tab-action')actions.push(e.detail);listeners.get(e.type)?.(e);}};
  const context={window,document,Intl,Date,CustomEvent:class{constructor(t,o){this.type=t;this.detail=o.detail;}}};
  for(const file of ['widgets','spotify'])runInNewContext(readFileSync(`public/new-tab/${file}.js`,'utf8'),context);
  listeners.get('spark-new-tab')({detail:{labels:JSON.parse(readFileSync('src/lib/i18n/catalogs/en/newtab.json','utf8')),preferences:{language:'en'}}});
  const settings={audio_background:{enabled:false,color:'#1db954',intensity:1,style:'bars'},weather_enabled:false,gmail_enabled:false,spotify_enabled:false,spotify_client_id:'',automatic_location:false,fahrenheit:false,city:null};
  let current={settings,revision:0,weather:{data:null,error:null,busy:false},gmail:{data:null,email:null,error:null,busy:false,connecting:false,configured:true}};
  const update=patch=>{current={...current,...patch};listeners.get('spark-new-tab-widgets')({detail:current});};update({});
  return {nodes,actions,update,settings,document,audio:patch=>listeners.get('spark-audio-setting')({detail:patch}),preferences:patch=>listeners.get('spark-new-tab')({detail:{labels:JSON.parse(readFileSync('src/lib/i18n/catalogs/en/newtab.json','utf8')),preferences:{language:'en'},...patch}})};
}
const change=(node,key,value)=>{node[key]=value;node.listeners.change({target:node});};
test('audio preference edits share the save queue without losing nested fields or other widgets',()=>{
  const f=fixture();f.audio({enabled:true});f.audio({color:'#abcdef',intensity:1.5});
  assert.equal(f.actions.filter(a=>a.operation==='save').length,1);
  f.update({settings:{...f.settings,audio_background:{...f.settings.audio_background,enabled:true}},revision:1});
  const saved=f.actions.at(-1).data.settings;
  assert.equal(saved.audio_background.enabled,true);assert.equal(saved.audio_background.color,'#abcdef');assert.equal(saved.audio_background.intensity,1.5);
  assert.equal(saved.spotify_enabled,false);assert.equal(saved.gmail_enabled,false);
});
test('Spotify settings validate Client ID, save through the shared queue and preserve a draft',()=>{
  const f=fixture(),id=f.nodes.get('spotify-client-id');
  f.nodes.get('menu-item-spotify').click();assert.equal(f.nodes.get('spotify-section').hidden,false);
  change(f.nodes.get('spotify-enabled'),'checked',true);
  assert.equal(f.actions.at(-1).data.settings.spotify_enabled,true);
  f.update({settings:{...f.settings,spotify_enabled:true,spotify_client_id:''},revision:1});
  id.value='invalid';id.listeners.input();f.nodes.get('spotify-save-client').click();
  assert.match(f.nodes.get('spotify-config-status').textContent,/32 hexadecimal/);
  f.update({});assert.equal(id.value,'invalid','polling must preserve a draft');
  id.value='a'.repeat(32);f.nodes.get('spotify-save-client').click();
  assert.equal(f.actions.at(-1).data.settings.spotify_client_id,'a'.repeat(32));
  f.update({settings:{...f.settings,spotify_enabled:true,spotify_client_id:'a'.repeat(32)},revision:2});
  assert.equal(f.nodes.get('spotify-widget-connect').disabled,false);
  f.nodes.get('spotify-widget-connect').click();assert.equal(f.actions.at(-1).operation,'spotify_connect');
});
test('Spotify renders safe metadata, toggles playback and honors busy/restricted controls',()=>{
  const f=fixture();const settings={...f.settings,spotify_enabled:true,spotify_client_id:'b'.repeat(32)};
  const data={title:'<script>track</script>',artist:'Artist',device:'Phone',playing:true,progress:12000,duration:180000,can_play:true,can_pause:true,can_next:false,can_previous:true,image:'https://evil.test/cover'};
  f.update({settings,spotify:{connected:true,data}});
  assert.equal(f.nodes.get('spotify-title').textContent,data.title);
  assert.equal(f.nodes.get('spotify-cover').hidden,true);
  assert.equal(f.nodes.get('spotify-next').disabled,true);
  assert.equal(f.nodes.get('spotify-previous').disabled,false);
  assert.equal(f.nodes.get('spotify-progress').value,12000);
  assert.equal(f.nodes.get('widgets-left').hidden,false);
  f.nodes.get('spotify-toggle').click();assert.equal(f.actions.at(-1).operation,'spotify_pause');
  f.update({spotify:{connected:true,busy:true,data}});assert.equal(f.nodes.get('spotify-toggle').disabled,true);
  f.update({spotify:{connected:true,data:{...data,playing:false,image:'data:image/png;base64,YQ=='}}});
  assert.equal(f.nodes.get('spotify-cover').hidden,false);
  f.nodes.get('spotify-toggle').click();assert.equal(f.actions.at(-1).operation,'spotify_play');
  f.nodes.get('spotify-disconnect').click();assert.equal(f.actions.at(-1).operation,'spotify_disconnect');
  f.update({spotify:{connected:true,error:'rate_limited',data}});assert.equal(f.nodes.get('spotify-toggle').disabled,true);
  f.update({spotify:{connected:true,error:'reconnect'}});
  assert.equal(f.nodes.get('spotify-connect').hidden,false);
  assert.match(f.nodes.get('spotify-status').textContent,/Connect Spotify again/);
  assert.doesNotMatch(f.nodes.get('spotify-status').textContent,/Gmail/);
  f.preferences({private:true});assert.equal(f.nodes.get('spotify-widget').hidden,true);assert.equal(f.nodes.get('menu-item-spotify').hidden,true);
});
test('widget settings use a focus-trapped dialog, close outside/Escape and preserve drafts',()=>{
  const f=fixture(),gear=f.nodes.get('widget-settings'),panel=f.nodes.get('settings-panel');
  f.nodes.get('search').value='unfinished';gear.click();
  assert.equal(panel.open,true);assert.equal(f.document.activeElement,f.nodes.get('settings-close'));
  assert.equal(f.actions.at(-1).operation,'panel');
  f.nodes.get('settings-close').click();assert.equal(panel.open,false);
  gear.click();
  panel.listeners.cancel({preventDefault(){}});assert.equal(panel.open,false);assert.equal(f.document.activeElement,gear);
  gear.click();panel.listeners.click({target:panel,clientX:10,clientY:20});assert.equal(panel.open,false);
  assert.equal(f.nodes.get('search').value,'unfinished');
});
test('settings categories slide in, return to the menu and keep the panel layout fixed',()=>{
  const f=fixture(),panel=f.nodes.get('settings-panel'),main=f.nodes.get('settings-main-view');
  const clock=f.nodes.get('menu-item-clock'),section=f.nodes.get('clock-section'),back=f.nodes.get('settings-back');
  clock.click();
  assert.equal(main.hidden,true);assert.equal(section.hidden,false);
  assert.equal(panel.attributes['data-panel-motion'],'forward');
  back.click();
  assert.equal(main.hidden,false);assert.equal(section.hidden,true);
  assert.equal(panel.attributes['data-panel-motion'],'back');
  assert.equal(f.document.activeElement,clock);

  const html=readFileSync('public/new-tab.html','utf8'),css=readFileSync('public/new-tab/widgets.css','utf8');
  const menu=html.match(/<nav class="settings-menu-list"[\s\S]*?<\/nav>/)?.[0]??'';
  assert.equal([...menu.matchAll(/class="menu-item-icon"[^>]*><svg/g)].length,5);
  assert.match(html,/id="settings-content"/);
  assert.match(css,/#settings-panel\s*\{[^}]*height:min\(420px, calc\(100vh - 90px\)\)/);
  assert.match(css,/#settings-panel:not\(\[open\]\)\s*\{\s*display:none/);
  assert.match(css,/#settings-content\s*\{[^}]*overflow-y:auto/);
  assert.match(css,/@media \(prefers-reduced-motion:reduce\)/);
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
