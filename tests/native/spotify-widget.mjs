import assert from 'node:assert/strict';

export async function checkSpotifyWidget(context,{inside},provider){
  const {script,until,click,resizeWindow}=context;
  const checked=(id,value)=>inside(()=>script('const n=document.getElementById(arguments[0]);n.checked=arguments[1];n.dispatchEvent(new Event("change"))',id,value));
  await inside(()=>click('#widget-settings'));
  await inside(()=>click('#menu-item-spotify'));
  await checked('spotify-enabled',true);
  await inside(()=>until(()=>script('return !document.querySelector("#spotify-options").hidden'),'Spotify settings enabled'));
  await inside(()=>script('const n=document.querySelector("#spotify-client-id");n.value="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";n.dispatchEvent(new Event("input"));document.querySelector("#spotify-save-client").click()'));
  await inside(()=>until(()=>script('return !document.querySelector("#spotify-connect").disabled'),'Spotify Client ID saved'));
  await inside(()=>click('#spotify-connect'));
  await inside(()=>until(()=>script('return document.querySelector("#spotify-title").textContent==="Artificial Spotify track"'),'Spotify playback loaded'));
  assert.equal(provider.spotify.authorizations.length,1);
  assert.equal(provider.spotify.authorizations[0].searchParams.get('scope'),'user-read-playback-state user-modify-playback-state');
  assert.equal(provider.spotify.grants[0].has('client_secret'),false);
  await inside(()=>click('#settings-close'));
  await inside(()=>click('#spotify-toggle'));
  await inside(()=>until(()=>script('return document.querySelector("#spotify-toggle").textContent==="▶"&&!document.querySelector("#spotify-toggle").disabled'),'Spotify paused'));
  await inside(()=>click('#spotify-next'));
  await inside(()=>until(()=>script('return document.querySelector("#spotify-title").textContent==="Artificial next track"'),'Spotify next track'));
  await inside(()=>click('#widget-settings'));
  await inside(()=>click('#menu-item-weather'));
  await checked('weather-enabled',true);
  await inside(()=>click('#settings-close'));
  await inside(()=>until(()=>script('return !document.querySelector("#weather-widget").hidden'),'Weather enabled above Spotify'));
  assert.equal(await inside(()=>script('return document.querySelector("#weather-widget").getBoundingClientRect().bottom<=document.querySelector("#spotify-widget").getBoundingClientRect().top')),true);
  await inside(()=>context.screenshot('spotify-widget'));
  await resizeWindow(680,600);
  await inside(async()=>{
    assert.equal(await script('return document.documentElement.scrollWidth<=innerWidth'),true);
    assert.equal(await script('return document.querySelector("#spotify-widget").getBoundingClientRect().bottom<=document.querySelector("#gmail-widget").getBoundingClientRect().top||document.querySelector("#gmail-widget").hidden'),true);
  });
  await resizeWindow(1440,1000);
  // A command-triggered read stays in flight while the connection is removed.
  provider.spotify.delay=true;
  await inside(()=>click('#spotify-toggle'));
  await until(()=>provider.spotify.pending.length>0,'Spotify delayed playback');
  await inside(()=>click('#widget-settings'));await inside(()=>click('#menu-item-spotify'));
  await inside(()=>click('#spotify-disconnect'));
  await inside(()=>until(()=>script('return document.querySelector("#spotify-disconnect").hidden'),'Spotify disconnected'));
  provider.spotify.release();provider.spotify.delay=false;
  await inside(()=>until(()=>script('return document.querySelector("#spotify-content").hidden'),'late Spotify data discarded'));
  await checked('spotify-enabled',false);
  await inside(()=>until(()=>script('return document.querySelector("#spotify-widget").hidden'),'Spotify disabled'));
  await inside(()=>click('#settings-close'));
}
