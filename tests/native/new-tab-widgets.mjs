import assert from 'node:assert/strict';
import {mkdirSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {widgetFixture} from './new-tab-widgets-fixture.mjs';
import {withScenarioDatabase} from './persistence/database.mjs';

export async function checkNewTabWidgets(context,{inside,ready,active}){
  const {restart,script,click,until,invoke,resizeWindow,wait}=context;
  const provider=await widgetFixture();
  const event=(operation,data={})=>inside(()=>script('window.dispatchEvent(new CustomEvent("veloce-new-tab-action",{detail:{action:"widgets",operation:arguments[0],data:arguments[1]}}))',operation,data));
  const open=()=>inside(async()=>{if(!await script('return document.querySelector("#settings-panel").open'))await click('#widget-settings');});
  const closed=()=>inside(()=>script('document.querySelector("#settings-close").click()'));
  const checked=async(id,value)=>inside(()=>script('const n=document.getElementById(arguments[0]);n.checked=arguments[1];n.dispatchEvent(new Event("change"))',id,value));
  try{
    await restart({beforeStart:directory=>{
      const config=resolve(directory,'config');mkdirSync(resolve(config,'oauth'),{recursive:true});
      writeFileSync(resolve(config,'widget-fixture-url'),provider.origin);
      writeFileSync(resolve(config,'oauth/google-desktop.credentials.json'),JSON.stringify({installed:{client_id:'artificial.apps.googleusercontent.com'}}));
    }});await ready();
    await context.focusWindow();
    await script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}));document.activeElement?.blur()');
    await resizeWindow(1440,1000);
    await inside(()=>until(()=>script('return !document.querySelector("#widget-settings").disabled'),'customization available'));
    await inside(()=>script('window.addEventListener("veloce-new-tab-widgets",e=>window.widgetLastReply=e.detail);window.addEventListener("error",e=>window.widgetLastError=e.message)'));
    assert.equal(provider.requests.length,0,'disabled widgets make no provider requests');
    await open();
    await inside(()=>script('document.querySelector("#search").value="keep my draft"'));
    await checked('weather-enabled',true);
    await inside(()=>until(()=>script('return !document.querySelector("#weather-options").hidden'),'weather enabled'));
    await inside(()=>script('document.querySelector("#city-query").value="Springfield";document.querySelector("#city-search").click()'));
    await inside(()=>until(()=>script('return document.querySelectorAll("#city-results button").length===2'),'ambiguous cities'));
    await inside(()=>click('#city-results button'));
    await inside(()=>until(()=>script('return document.querySelector(".weather-temperature")?.textContent.includes("20°C")'),'weather forecast'));
    assert.equal(provider.requests.filter(p=>p==='/forecast').length,1);
    await inside(()=>script('const n=document.querySelector("#temperature-unit");n.value="fahrenheit";n.dispatchEvent(new Event("change"))'));
    await inside(()=>until(()=>script('return document.querySelector(".weather-temperature")?.textContent.includes("68°F")'),'local unit conversion'));
    assert.equal(provider.requests.filter(p=>p==='/forecast').length,1,'units do not refetch weather');
    await checked('gmail-enabled',true);
    await inside(()=>until(()=>script('return !document.querySelector("#gmail-options").hidden&&!document.querySelector("#gmail-connect").disabled'),'Gmail enabled'));
    await inside(()=>click('#gmail-connect'));
    await inside(()=>until(()=>script('return document.querySelectorAll(".gmail-message").length===5'),'authenticated inbox'));
    assert.equal(provider.authorizations[0].searchParams.get('code_challenge_method'),'S256');
    assert.equal(createHash('sha256').update(provider.lastGrant.get('code_verifier')).digest('base64url'),provider.authorizations[0].searchParams.get('code_challenge'));
    await inside(async()=>{
      assert.match(await script('return document.querySelector("#gmail-unread").textContent'),/7/);
      assert.equal(await script('return document.querySelector(".mail-sender").textContent'),'<script>Artificial sender</script>');
      assert.equal(await script('return document.querySelector("#gmail-messages script")===null'),true);
      assert.equal(await script('return document.querySelector("#search").value'),'keep my draft');
      await click('#settings-close');assert.equal(await script('return document.activeElement.id'),'widget-settings');
      assert.equal(await script('const w=document.querySelector("#weather-widget").getBoundingClientRect(),m=document.querySelector("#gmail-widget").getBoundingClientRect();return w.left<m.left&&Math.abs(w.top-m.top)<2'),true,'widgets occupy top corners');
    });
    await inside(()=>context.screenshot('new-tab-widgets'));
    await resizeWindow(1000,620);
    await inside(()=>script('const a=document.querySelector("#weather-widget").getBoundingClientRect(),b=document.querySelector("#gmail-widget").getBoundingClientRect(),s=document.querySelector("#search-form").getBoundingClientRect();if(s.top<Math.max(a.bottom,b.bottom)+20)throw Error("five-message widget overlaps search")'));
    await resizeWindow(1440,1000);
    await open();
    await inside(()=>context.request('POST',`/session/${context.sessionId()}/actions`,{actions:[{type:'key',id:'widget-keys',actions:[{type:'keyDown',value:'\uE00C'},{type:'keyUp',value:'\uE00C'}]}]}));
    await inside(()=>until(()=>script('return !document.querySelector("#settings-panel").open&&document.activeElement.id==="widget-settings"'),'native Escape closes customization and restores focus'));
    await wait(2100);provider.failForecast=true;await event('refresh_weather');
    await inside(()=>until(()=>script('return /connect|conectar/i.test(document.querySelector("#weather-status").textContent)'),'weather failure is explained'));
    assert.equal(await inside(()=>script('return document.querySelector(".weather-temperature").textContent.includes("68°F")')),true,'failed weather keeps the last forecast');
    provider.failForecast=false;await wait(2100);await event('refresh_weather');
    await inside(()=>until(()=>script('return /Updated|Atualizado/.test(document.querySelector("#weather-status").textContent)'),'weather retry recovers'));
    provider.delayForecast=true;await wait(2100);await event('refresh_weather');
    await until(()=>provider.pending.length===1,'delayed forecast in flight');
    await open();await checked('weather-enabled',false);
    await until(()=>withScenarioDatabase(context.directory,db=>!JSON.parse(db.prepare('SELECT value FROM new_tab_settings WHERE id=1').get().value).weather_enabled),'weather disable committed');
    provider.releaseWeather();provider.delayForecast=false;await wait(200);
    assert.equal(await inside(()=>script('return document.querySelector("#weather-widget").hidden')),true,'late forecast cannot restore a disabled widget');
    await checked('weather-enabled',true);
    await inside(()=>until(()=>script('return document.querySelector(".weather-temperature")?.textContent.includes("68°F")'),'weather re-enabled'));
    await closed();
    const source=(await active()).id;
    const second=(await invoke('create_tab')).tabs.find(t=>t.is_active);await ready();
    await invoke('set_split_view',{leftTabId:source,rightTabId:second.id});
    await inside(()=>until(()=>script('return document.querySelectorAll(".gmail-message").length===5'),'shared mailbox in split'));
    assert.equal(provider.requests.filter(p=>p==='/gmail/users/me/messages').length,1,'mail cache shared between panes');
    await resizeWindow(680,480);
    for(const id of [source,second.id])await inside(async()=>{
      assert.equal(await script('return document.documentElement.scrollWidth<=innerWidth'),true,'narrow widgets do not overflow');
      assert.equal(await script('const a=document.querySelector("#weather-widget").getBoundingClientRect(),b=document.querySelector("#gmail-widget").getBoundingClientRect();return a.bottom<=b.top'),true,'narrow widgets stack');
    },id);
    await resizeWindow(1440,1000);
    await open();await checked('weather-enabled',false);
    await inside(()=>until(()=>script('return document.querySelector("#weather-widget").hidden'),'disabled weather'));
    await inside(()=>until(()=>script('return document.querySelector("#weather-widget").hidden'),'settings synchronize across panes'),source);
    await withScenarioDatabase(context.directory,async db=>{
      db.exec('ALTER TABLE new_tab_settings RENAME TO unavailable_widget_settings');
      try{
        await checked('weather-enabled',true);
        await inside(()=>until(()=>script('return !document.querySelector("#weather-enabled").checked&&document.querySelector("#settings-status").textContent.length>0'),'failed settings save restores confirmed state'));
      }finally{db.exec('ALTER TABLE unavailable_widget_settings RENAME TO new_tab_settings');}
    });
    await inside(()=>click('#settings-back'));
    await inside(()=>click('#menu-item-gmail'));
    await inside(()=>click('#gmail-disconnect'));
    await inside(()=>until(()=>script('return document.querySelectorAll(".gmail-message").length===0'),'disconnect clears metadata'));
    provider.holdAuthorization=true;
    await inside(()=>click('#gmail-connect'));await until(()=>provider.authorizations.length===2,'second OAuth begins');
    assert.equal((await provider.complete(provider.authorizations[1],true)).status,400,'wrong state rejected');
    await inside(()=>click('#gmail-cancel'));
    await inside(()=>until(()=>script('return document.querySelector("#gmail-cancel").hidden'),'OAuth cancelled'));
    await assert.rejects(()=>provider.complete(provider.authorizations[1]));
    await inside(()=>click('#gmail-connect'));await until(()=>provider.authorizations.length===3,'replacement OAuth');
    provider.deny=true;await provider.complete(provider.authorizations[2]);provider.deny=false;
    await inside(()=>until(()=>script('return /denied|negado/i.test(document.querySelector("#gmail-config-status").textContent)'),'OAuth denial is explained'));
    await inside(()=>click('#gmail-connect'));await until(()=>provider.authorizations.length===4,'OAuth retry');
    provider.expireNextAccess=true;await provider.complete(provider.authorizations[3]);
    await inside(()=>until(()=>script('return document.querySelectorAll(".gmail-message").length===5'),'replacement OAuth succeeds'));
    assert.ok(provider.grants.some(form=>form.get('grant_type')==='refresh_token'),'expired access is renewed in the backend');
    // A delayed inbox response must not repopulate a disconnected account.
    await wait(2100);provider.delayMail=true;
    await event('refresh_gmail');await until(()=>provider.mailPending.length===1,'delayed inbox in flight');
    await inside(()=>click('#gmail-disconnect'));provider.releaseMail();provider.delayMail=false;
    await wait(200);
    await inside(()=>until(()=>script('return document.querySelectorAll(".gmail-message").length===0'),'old inbox discarded'));
    await checked('gmail-enabled',false);await closed();
    await withScenarioDatabase(context.directory,db=>{
      const value=JSON.parse(db.prepare('SELECT value FROM new_tab_settings WHERE id=1').get().value);
      assert.equal(value.fahrenheit,true);assert.equal(value.city.region,'Illinois');
      assert.ok(!JSON.stringify(value).includes('artificial-access'));
    });
    await restart();await ready();await open();
    await inside(async()=>{assert.equal(await script('return document.querySelector("#temperature-unit").value'),'fahrenheit');assert.match(await script('return document.querySelector("#selected-city").textContent'),/Illinois/);});
    await closed();
  }catch(error){
    const page=await inside(()=>script('return {reply:window.widgetLastReply,error:window.widgetLastError,status:document.querySelector("#settings-status").textContent,cityStatus:document.querySelector("#city-status").textContent,query:document.querySelector("#city-query").value,results:document.querySelector("#city-results").textContent}')).catch(()=>null);
    writeFileSync(resolve(context.directory,'artifacts/widget-debug.json'),JSON.stringify({page,requests:provider.requests,tabs:await invoke('get_tabs')},null,2));
    throw error;
  }finally{await provider.close();}
}
