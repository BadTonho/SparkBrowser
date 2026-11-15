import {createServer} from 'node:http';

export async function widgetFixture(){
  const state={requests:[],authorizations:[],grants:[],pending:[],delayForecast:false,failForecast:false,delayMail:false,mailPending:[],deny:false,holdAuthorization:false,expireNextAccess:false};
  state.spotify={authorizations:[],grants:[],commands:[],playing:true,title:'Artificial Spotify track',delay:false,pending:[],status:200};
  const forecast={current:{temperature_2m:20,apparent_temperature:19,weather_code:0},daily:{time:['2026-11-03','2026-11-04','2026-11-05'],weather_code:[0,3,61],temperature_2m_min:[10,11,12],temperature_2m_max:[20,21,22]}};
  const json=(response,value,status=200)=>response.writeHead(status,{'Content-Type':'application/json'}).end(JSON.stringify(value));
  const playback=()=>({device:{name:'Artificial phone',is_restricted:false},is_playing:state.spotify.playing,progress_ms:12000,item:{type:'track',name:state.spotify.title,duration_ms:180000,artists:[{name:'Artificial artist'}],album:{images:[]}},actions:{disallows:{}}});
  state.spotify.release=()=>{for(const response of state.spotify.pending.splice(0))json(response,playback());};
  state.releaseWeather=()=>{for(const response of state.pending.splice(0))json(response,forecast);};
  const messages=Array.from({length:5},(_,i)=>({id:(0xabc+i).toString(16)}));
  state.releaseMail=()=>{for(const response of state.mailPending.splice(0))json(response,{messages});};
  state.complete=async(authorize,wrong=false)=>{
    const callback=new URL(authorize.searchParams.get('redirect_uri'));
    callback.searchParams.set('state',wrong?'invalid':authorize.searchParams.get('state'));
    callback.searchParams.set(state.deny?'error':'code',state.deny?'access_denied':'artificial-code');
    return fetch(callback,{signal:AbortSignal.timeout(2000)});
  };
  const server=createServer(async(request,response)=>{
    const url=new URL(request.url,'http://127.0.0.1');state.requests.push(url.pathname);
    if(url.pathname==='/spotify/authorize'){
      let body='';for await(const part of request)body+=part;
      const authorize=new URL(body);state.spotify.authorizations.push(authorize);response.end('OK');
      setTimeout(()=>state.complete(authorize).catch(()=>{}),150);return;
    }
    if(url.pathname==='/spotify/token'){
      let body='';for await(const part of request)body+=part;
      state.spotify.grants.push(new URLSearchParams(body));
      return json(response,{access_token:'artificial-spotify-access',refresh_token:'artificial-spotify-refresh',expires_in:3600,scope:'user-read-playback-state user-modify-playback-state'});
    }
    if(url.pathname.startsWith('/spotify/v1/me/player')){
      if(state.spotify.status!==200){response.setHeader('Retry-After','2');return json(response,{},state.spotify.status);}
      if(request.method==='GET'){
        if(state.spotify.delay){state.spotify.pending.push(response);return;}
        return json(response,playback());
      }
      state.spotify.commands.push(url.pathname);
      if(url.pathname.endsWith('/pause'))state.spotify.playing=false;
      if(url.pathname.endsWith('/play'))state.spotify.playing=true;
      if(url.pathname.endsWith('/next'))state.spotify.title='Artificial next track';
      response.writeHead(204).end();return;
    }
    if(url.pathname==='/cities')return json(response,{results:[{name:'Springfield',admin1:'Illinois',country:'United States',latitude:39.8,longitude:-89.6},{name:'Springfield',admin1:'Massachusetts',country:'United States',latitude:42.1,longitude:-72.6}]});
    if(url.pathname==='/forecast'){
      if(state.delayForecast){state.pending.push(response);return;}
      return json(response,forecast,state.failForecast?503:200);
    }
    if(url.pathname==='/authorize'){
      let body='';for await(const part of request)body+=part;
      const authorize=new URL(body);state.authorizations.push(authorize);response.end('OK');
      if(!state.holdAuthorization)setTimeout(()=>state.complete(authorize).catch(()=>{}),150);
      return;
    }
    if(url.pathname==='/token'){
      let body='';for await(const part of request)body+=part;
      const form=new URLSearchParams(body);state.lastGrant=form;state.grants.push(form);
      const expires=state.expireNextAccess?0:3600;state.expireNextAccess=false;
      return json(response,{access_token:'artificial-access',refresh_token:'artificial-refresh',expires_in:expires,scope:'https://www.googleapis.com/auth/gmail.metadata'});
    }
    if(url.pathname==='/revoke')return json(response,{});
    if(url.pathname==='/gmail/users/me/profile')return json(response,{emailAddress:'test@example.test'});
    if(url.pathname==='/gmail/users/me/labels/INBOX')return json(response,{messagesUnread:7});
    if(url.pathname==='/gmail/users/me/messages'){
      if(state.delayMail){state.mailPending.push(response);return;}
      return json(response,{messages});
    }
    if(/^\/gmail\/users\/me\/messages\/ab[c-f0-9]$/.test(url.pathname)||url.pathname==='/gmail/users/me/messages/ac0')return json(response,{internalDate:'1793700000000',labelIds:['INBOX','UNREAD'],payload:{headers:[{name:'From',value:'<script>Artificial sender</script>'},{name:'Subject',value:'Artificial subject'}]}});
    return json(response,{},404);
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  state.origin=`http://127.0.0.1:${server.address().port}/`;
  state.close=async()=>{state.releaseWeather();state.releaseMail();state.spotify.release();server.closeAllConnections();await new Promise(resolve=>server.close(resolve));};
  return state;
}
