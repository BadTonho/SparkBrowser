(() => {
  const token = __MEDIA_TOKEN__;
  // WebKitGTK echoes the audio sink's remembered stream volume back as `volumechange`; an engine
  // change without a user gesture is not a player control and must not become the page's volume.
  const engineEcho = __MEDIA_ENGINE_ECHO__;
  const documentId = crypto.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
  let target = __MEDIA_VOLUME__;
  let status = 'ready';
  let applied = 100;
  let context;
  let busy = Promise.resolve();
  const routes = new Map();
  // WebKit has no captureStream(): amplification routes the element through WebAudio instead. A
  // media element source can never be released, so its graph outlives the volume setting.
  const graphs = new WeakMap();
  let graphCount = 0;
  const videoIds = new WeakMap();
  let videoCounter=0;
  let floating=null;
  let signalSequence=0;
  const createFloatingPlayer = __PIP_PLAYER__;
  const pipElement=()=>floating?.element ?? document.pictureInPictureElement;
  const videoId=el=> {if(!videoIds.has(el))videoIds.set(el,`${documentId}-${++videoCounter}`);return videoIds.get(el);};
  // WebKit has no media-element capture (and cannot display a MediaStream): the player copies frames.
  const mirrorByCanvas=engineEcho&&typeof HTMLMediaElement.prototype.captureStream!=='function';
  // WebKitGTK navigates the related popup view again after window.open(), replacing the document
  // the opener would have built. Wait for that replacement (never needed, or waited for, on Windows).
  async function popupReady(popup,initial) {
    if(!engineEcho)return;
    const end=Date.now()+4000;
    while(Date.now()<end) {
      try {if(popup.document!==initial&&popup.document.readyState==='complete')return;} catch {}
      await new Promise(resolve=>setTimeout(resolve,25));
    }
  }
  function candidates() {return [...document.querySelectorAll('video')].slice(0,64).map(el=>({id:videoId(el),label:(el.title||el.getAttribute('aria-label')||document.title||'').slice(0,160),width:el.videoWidth,height:el.videoHeight,eligible:Boolean((document.pictureInPictureEnabled||mirrorByCanvas) && !el.disablePictureInPicture && !el.mediaKeys && el.readyState>=2 && el.videoWidth>0)}));}
  // Video download panel: addresses the page already uses. Nothing is fetched here.
  // The engine's resource list stops filling on heavy pages (250 entries by default), but an observer
  // still hears every request: remember the latest playlists for when the panel asks.
  const playlists=new Set();
  const isPlaylist=raw=> {try {return new URL(raw,location.href).pathname.toLowerCase().endsWith('.m3u8');} catch {return false;}};
  function remember(entries) {
    for(const entry of entries) if(isPlaylist(entry.name)) {
      playlists.delete(entry.name);playlists.add(entry.name);
      if(playlists.size>32)playlists.delete(playlists.values().next().value);
    }
  }
  if(typeof PerformanceObserver==='function') {
    try {new PerformanceObserver(list=>remember(list.getEntries())).observe({type:'resource',buffered:true});} catch {}
  }
  // Many players are web components: their video lives in a shadow tree that the document cannot query.
  function elements() {
    const found=[];
    const walk=root=> {for(const el of root.querySelectorAll('*')) {if(found.length>=50000)return;found.push(el);if(el.shadowRoot)walk(el.shadowRoot);}};
    walk(document);
    return found;
  }
  // What tells one video from another: where it is, how long it is and a picture of it.
  function onScreen(el) {
    const r=el.getBoundingClientRect?.();
    if(!r)return 0;
    return Math.round(Math.max(0,Math.min(r.right,innerWidth)-Math.max(r.left,0))*Math.max(0,Math.min(r.bottom,innerHeight)-Math.max(r.top,0)));
  }
  // The poster may be stated by the video or by the player component around it.
  function posterOf(el) {
    for(let node=el,depth=0;node&&depth<6;depth++,node=node.parentElement??node.getRootNode?.().host) {
      const raw=node.getAttribute?.('poster');
      if(!raw)continue;
      try {const url=new URL(raw,location.href);if(['http:','https:'].includes(url.protocol)&&url.href.length<=4096)return url.href;} catch {}
    }
    return '';
  }
  // Fails for a video of another origin served without CORS; the poster is the fallback then.
  function frameOf(el) {
    if(!(el.readyState>=2&&el.videoWidth>0&&el.videoHeight>0))return '';
    try {
      const canvas=document.createElement('canvas');
      canvas.width=160;canvas.height=Math.max(1,Math.min(284,Math.round(160*el.videoHeight/el.videoWidth)));
      canvas.getContext('2d').drawImage(el,0,0,canvas.width,canvas.height);
      const data=canvas.toDataURL('image/jpeg',0.7);
      return data.length<=65536?data:'';
    } catch {return '';}
  }
  function sources() {
    const found=new Map();
    const add=(raw,type,height,group)=> {
      let url; try {url=new URL(raw,location.href);} catch {return;}
      if(!['http:','https:'].includes(url.protocol)||url.href.length>4096)return;
      const path=url.pathname.toLowerCase();
      if(path.endsWith('.mpd')||/dash\+xml/i.test(type))return;
      const kind=path.endsWith('.m3u8')||/mpegurl/i.test(type)?'hls':'file';
      const known=found.get(url.href);
      if(known) {known.group||=group;return;}
      if(found.size<96)found.set(url.href,{url:url.href,kind,height,group});
    };
    const all=elements();
    // What is on the screen comes first, so it is never the part left out.
    const videos=all.filter(el=>el.localName==='video').map(el=>({el,area:onScreen(el)})).sort((a,b)=>b.area-a.area).slice(0,64);
    const groups=new Map();
    const describe=(group,el,area,video)=> {
      if(groups.has(group)||groups.size>=32)return;
      groups.set(group,{group,area,duration:video&&Number.isFinite(video.duration)?Math.round(video.duration):0,poster:posterOf(video??el),frame:video&&groups.size<12?frameOf(video):''});
    };
    for(const {el,area} of videos) {
      if(el.mediaKeys)continue;
      const group=videoId(el);
      describe(group,el,area,el);
      const listed=[...el.querySelectorAll('source')];
      if(el.currentSrc)add(el.currentSrc,listed.find(s=>s.src===el.currentSrc)?.type??'',el.videoHeight||0,group);
      for(const s of listed)if(s.src)add(s.src,s.type??'',0,group);
    }
    // A player component often states its playlist on its own element.
    let players=0;
    for(const el of all) {
      const src=el.getAttribute?.('src');
      if(!src||!isPlaylist(src))continue;
      const inner=el.localName==='video'?el:el.shadowRoot?.querySelector?.('video')??el.querySelector?.('video');
      if(inner?.mediaKeys)continue;
      const group=inner?videoId(inner):`${documentId}-player-${++players}`;
      describe(group,el,onScreen(inner??el),inner);
      add(src,'',0,group);
    }
    // Players built on Media Source show a blob: address; their playlist is among the requests made.
    remember(performance.getEntriesByType('resource'));
    for(const name of playlists)add(name,'',0,'');
    const rank=source=>groups.get(source.group)?.area??-1;
    const listed=[...found.values()].sort((a,b)=>rank(b)-rank(a));
    return {page:location.href,protected:videos.some(({el})=>Boolean(el.mediaKeys)),sources:listed,videos:[...groups.values()].filter(video=>listed.some(source=>source.group===video.group))};
  }
  function progress() {
    const el=media().find(m=>!m.paused&&!m.ended)??media()[0];
    if(!el||!Number.isFinite(el.duration)||el.duration<=0||!Number.isFinite(el.currentTime))return {};
    return {position:Math.round(el.currentTime*10)/10,duration:Math.round(el.duration*10)/10};
  }
  function signal() {
    if(typeof window.__sparkMediaSignal!=='function')return;
    window.__sparkMediaSignal(JSON.stringify({token,document:documentId,sequence:++signalSequence,playing:media().some(el=>!el.paused&&!el.ended),picture_in_picture:Boolean(pipElement()),pip_video:pipElement()?videoId(pipElement()):null,pip_mode:floating?'custom':document.pictureInPictureElement?'native':null,pip_session:floating?.session??null,...progress()}));
  }
  const volume = Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype, 'volume');
  const muted = Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype, 'muted');
  const media = () => [...document.querySelectorAll('audio,video')];
  const level = n => Math.max(0, Math.min(200, Math.round(n)));
  function restore(el, route) {
    if (route.graph) {route.graph.gain.gain.setValueAtTime(1, context?.currentTime ?? 0);routes.delete(el);return;}
    route.source?.disconnect(); route.gain?.disconnect();
    route.stream?.getTracks().forEach(t => t.stop());
    if (route.wrapped) {
      delete el.volume; delete el.muted;
      volume.set.call(el, route.volume); muted.set.call(el, route.muted);
    }
    routes.delete(el);
  }
  function wrap(el, route) {
    if (route.wrapped || route.graph) return;
    if (Object.hasOwn(el, 'volume') || Object.hasOwn(el, 'muted')) throw Error('custom-player');
    Object.defineProperties(el, {
      volume: {configurable:true, get:()=>route.volume, set:n=>{ if (!Number.isFinite(n)||n<0||n>1) throw new RangeError('volume'); route.volume=n; update(el,route); }},
      muted: {configurable:true, get:()=>route.muted, set:n=>{route.muted=Boolean(n);update(el,route);}}
    });
    route.wrapped = true;
  }
  function update(el, route) {
    if (route.graph) {route.graph.gain.gain.setValueAtTime(target / 100, context.currentTime);return;}
    route.physicalVolume = route.gain ? route.volume : Math.min(1, route.volume * target / 100);
    route.physicalMuted = route.gain ? true : route.muted;
    volume.set.call(el, route.physicalVolume);
    if (route.gain) {
      route.gain.gain.setValueAtTime(route.muted ? 0 : route.volume * target / 100, context.currentTime);
    }
    muted.set.call(el, route.physicalMuted);
  }
  // Only elements whose audio WebAudio may read: otherwise the graph would silence them for good.
  function readable(el) {
    try {
      if (el.srcObject || el.crossOrigin !== null) return true;
      const source = el.currentSrc || el.src;
      if (!source) return false;
      const url = new URL(source, location.href);
      return url.protocol === 'blob:' || url.protocol === 'data:' || url.origin === location.origin;
    } catch {return false;}
  }
  async function amplifyThroughGraph(el, route, user) {
    if (el.mediaKeys || el.readyState < 2 || typeof AudioContext !== 'function') throw Error('unsupported');
    if (!graphs.has(el) && !readable(el)) throw Error('unsupported');
    if (route.wrapped) {
      // The graph's gain now owns the level; hand the element back to the page first.
      delete el.volume; delete el.muted;
      volume.set.call(el, route.volume); muted.set.call(el, route.muted);
      route.wrapped = false;
    }
    let graph = graphs.get(el);
    if (!graph) {
      context ??= new AudioContext();
      if (context.state !== 'running' && user) await Promise.race([context.resume(),new Promise(resolve=>setTimeout(resolve,400))]);
      if (context.state !== 'running') throw Error('pending');
      const source = context.createMediaElementSource(el);
      const gain = context.createGain(); gain.gain.value = 1;
      source.connect(gain).connect(context.destination);
      graph = {source, gain}; graphs.set(el, graph); graphCount++;
    }
    route.graph = graph;
  }
  async function amplify(el, route, user) {
    // Native controls read the engine's physical mute, which capture routing must own.
    // Retain their behavior by leaving amplification unavailable for these players.
    if (el.controls) throw Error('native-controls');
    if (route.gain || route.graph) return;
    if (typeof el.captureStream !== 'function') return amplifyThroughGraph(el, route, user);
    if (el.mediaKeys || el.readyState < 2) throw Error('unsupported');
    const stream = el.captureStream();
    const tracks = stream.getAudioTracks();
    if (!tracks.length || tracks.some(t=>t.muted || t.readyState !== 'live')) {stream.getTracks().forEach(t=>t.stop());throw Error('unsupported');}
    try {
      context ??= new AudioContext();
      if (context.state !== 'running' && user) await Promise.race([context.resume(),new Promise(resolve=>setTimeout(resolve,400))]);
      if (context.state !== 'running') throw Error('pending');
      const source = context.createMediaStreamSource(new MediaStream(tracks));
      const gain = context.createGain(); gain.gain.value=0;
      source.connect(gain).connect(context.destination);
      wrap(el,route);
      route.stream=stream;route.source=source;route.gain=gain;
      tracks.forEach(t=> {t.addEventListener('ended',()=>schedule());t.addEventListener('mute',()=>schedule());});
    } catch (e) {stream.getTracks().forEach(t=>t.stop());throw e;}
  }
  async function apply(user = false) {
    for (const [el,route] of routes) if (!el.isConnected || target === 100) restore(el,route);
    if (target === 100) {if(context&&!graphCount){void context.close();context=undefined;}applied=100;status='ready';return;}
    const elements = media();
    let failure = false, pending = false;
    for (const el of elements) {
      if (el.readyState < 1) {pending=true;continue;}
      let route = routes.get(el);
      if (!route) {route={volume:el.volume,muted:el.muted,graph:graphs.get(el)};routes.set(el,route);}
      try {
        if (target>100) await amplify(el,route,user); else wrap(el,route);
        if (route.stream?.getAudioTracks().some(t=>t.muted || t.readyState!=='live')) throw Error('unsupported');
        update(el,route);
      } catch(e) {if(e.message==='pending')pending=true;else failure=true;restore(el,route);}
    }
    if (failure || pending) {
      for(const [el,route] of routes)restore(el,route);
      applied=100;status=pending?'pending':target>100?'limited':'unavailable';
      if (failure) target=100;
    } else {applied=pending ? 100 : target;status=pending?'pending':failure?'unavailable':elements.length?'ready':'unavailable';}
  }
  let scheduled=false;
  function schedule() { if(scheduled)return;scheduled=true;setTimeout(()=>{scheduled=false;busy=busy.catch(()=>{}).then(()=>apply(Boolean(navigator.userActivation?.isActive)));},60); }
  new MutationObserver(() => { if (target !== 100) schedule(); }).observe(document, {childList:true,subtree:true,attributes:true,attributeFilter:['controls']});
  document.addEventListener('volumechange',event=> {
    const el=event.target,route=routes.get(el);
    if(!route?.wrapped)return;
    const actualVolume=volume.get.call(el),actualMuted=muted.get.call(el);
    let changed=false;
    const trusted=!engineEcho||Boolean(navigator.userActivation?.isActive);
    if(Math.abs(actualVolume-route.physicalVolume)>0.000001){if(trusted)route.volume=actualVolume;changed=true;}
    if(actualMuted!==route.physicalMuted){if(trusted)route.muted=actualMuted;changed=true;}
    if(changed)update(el,route);
  },true);
  for (const event of ['loadedmetadata','playing','emptied','durationchange']) document.addEventListener(event,()=>{if(target!==100)schedule();},true);
  for (const event of ['play','pause','ended','enterpictureinpicture','leavepictureinpicture']) document.addEventListener(event,signal,true);
  // The new-tab YouTube card asks for a fresh position sample; the answer only describes this document.
  window.addEventListener('spark-media-probe',signal);
  document.addEventListener('pointerdown',()=>{if(target!==100)schedule();},true);
  document.addEventListener('keydown',()=>{if(target!==100)schedule();},true);
  window.addEventListener('pagehide',()=> {floating?.popup.close();floating?.cleanup();for(const [el,route] of routes)restore(el,route);void context?.close();});
  Object.defineProperty(window,'__sparkMedia',{value:async (key,action)=> {
    if (key!==token || (action.document && action.document!==documentId)) return {error:'media.stale'};
    if(action.kind==='sources')return sources();
    if(action.kind==='pip-exit') {try{if(floating){floating.cleanup();floating=null;}if(document.pictureInPictureElement)await document.exitPictureInPicture();signal();return {ok:true};}catch{return {error:'media.pip_failed'};}}
    if(action.kind==='pip-enter') {
      const candidate=candidates().find(v=>v.id===action.video&&v.eligible);
      const el=[...document.querySelectorAll('video')].find(v=>videoId(v)===candidate?.id);
      if(!el)return {error:'media.pip_unavailable'};
      try {
        let stream,popup;
        if(action.session && (typeof el.captureStream==='function'||mirrorByCanvas)) {
          try {
            if(!mirrorByCanvas) {
              stream=el.captureStream();
              stream.getAudioTracks().forEach(t=>t.stop());
              const tracks=stream.getVideoTracks();
              if(!tracks.length||tracks.some(t=>t.readyState!=='live'||(t.muted&&!el.paused)))throw Error('capture');
            }
            popup=window.open(`about:blank#spark-pip-${action.session}`,'','popup,width=400,height=225');
            if(!popup)throw Error('popup');
            const initialDocument=popup.document;
            await popupReady(popup,initialDocument);
            let sequence=0;
            const send=(name,value)=>{const seq=++sequence;popup.location.href=`spark-internal://pip-player/${name}?session=${encodeURIComponent(action.session)}&sequence=${seq}${value===undefined?'':`&value=${value}`}`;return seq;};
            const cleanup=createFloatingPlayer(popup,el,stream,{...action,title:document.title,hostname:location.hostname,volume:target,canvas:mirrorByCanvas},send,()=>{floating=null;signal();});
            floating={element:el,popup,cleanup,session:action.session};signal();return {ok:true,custom:true};
          } catch {stream?.getTracks().forEach(t=>t.stop());popup?.close();}
        }
        await el.requestPictureInPicture();signal();return {ok:true,custom:false};
      }
      catch(e){return {error:e.name==='SecurityError'||e.name==='NotAllowedError'?'media.pip_blocked':'media.pip_failed'};}
    }
    if (action.kind==='volume') {
      if (!Number.isInteger(action.value)||action.value<0||action.value>200) return {error:'media.invalid_volume'};
      target=level(action.value);
    }
    if (!['report','volume'].includes(action.kind)) return {error:'media.error'};
    busy=busy.catch(()=>{}).then(()=>apply(action.kind==='volume')); await busy;
    return {document:documentId,count:media().length,applied_volume:applied,status,videos:candidates(),playing:media().some(el=>!el.paused&&!el.ended),picture_in_picture:Boolean(pipElement()),pip_video:pipElement()?videoId(pipElement()):null,pip_mode:floating?'custom':document.pictureInPictureElement?'native':null,pip_session:floating?.session??null};
  },configurable:false,writable:false});
  if (target!==100) schedule();
})();
