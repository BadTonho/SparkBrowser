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
