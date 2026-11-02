(() => {
  const token = __MEDIA_TOKEN__;
  const documentId = crypto.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
  let target = __MEDIA_VOLUME__;
  let status = 'ready';
  let applied = 100;
  let context;
  let busy = Promise.resolve();
  const routes = new Map();
  const volume = Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype, 'volume');
  const muted = Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype, 'muted');
  const media = () => [...document.querySelectorAll('audio,video')];
  const level = n => Math.max(0, Math.min(200, Math.round(n)));
  function restore(el, route) {
    route.source?.disconnect(); route.gain?.disconnect();
    route.stream?.getTracks().forEach(t => t.stop());
    if (route.wrapped) {
      delete el.volume; delete el.muted;
      volume.set.call(el, route.volume); muted.set.call(el, route.muted);
    }
    routes.delete(el);
  }
  function wrap(el, route) {
    if (route.wrapped) return;
    if (Object.hasOwn(el, 'volume') || Object.hasOwn(el, 'muted')) throw Error('custom-player');
    Object.defineProperties(el, {
      volume: {configurable:true, get:()=>route.volume, set:n=>{ if (!Number.isFinite(n)||n<0||n>1) throw new RangeError('volume'); route.volume=n; update(el,route); }},
      muted: {configurable:true, get:()=>route.muted, set:n=>{route.muted=Boolean(n);update(el,route);}}
    });
    route.wrapped = true;
  }
  function update(el, route) {
    if (route.gain) {
      route.gain.gain.setValueAtTime(route.muted ? 0 : route.volume * target / 100, context.currentTime);
      muted.set.call(el, true);
    } else {
      volume.set.call(el, Math.min(1, route.volume * target / 100));
      muted.set.call(el, route.muted);
    }
  }
  async function amplify(el, route, user) {
    if (route.gain) return;
    if (typeof el.captureStream !== 'function' || el.mediaKeys || el.readyState < 2) throw Error('unsupported');
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
    const elements = media();
    for (const [el,route] of routes) if (!el.isConnected || target === 100) restore(el,route);
    if (target === 100) {if(context){void context.close();context=undefined;}applied=100;status='ready';return;}
    let failure = false, pending = false;
    for (const el of elements) {
      if (el.readyState < 1) {pending=true;continue;}
      let route = routes.get(el);
      if (!route) {route={volume:el.volume,muted:el.muted};routes.set(el,route);}
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
  new MutationObserver(schedule).observe(document, {childList:true,subtree:true});
  for (const event of ['loadedmetadata','playing','emptied','durationchange']) document.addEventListener(event,schedule,true);
  document.addEventListener('pointerdown',()=>schedule(),true);
  document.addEventListener('keydown',()=>schedule(),true);
  window.addEventListener('pagehide',()=> {for(const [el,route] of routes)restore(el,route);void context?.close();});
  Object.defineProperty(window,'__veloceMedia',{value:async (key,action)=> {
    if (key!==token || (action.document && action.document!==documentId)) return {error:'media.stale'};
    if (action.kind==='volume') {
      if (!Number.isInteger(action.value)||action.value<0||action.value>200) return {error:'media.invalid_volume'};
      target=level(action.value);
    }
    if (!['report','volume'].includes(action.kind)) return {error:'media.error'};
    busy=busy.catch(()=>{}).then(()=>apply(action.kind==='volume')); await busy;
    return {document:documentId,count:media().length,applied_volume:applied,status};
  },configurable:false,writable:false});
  if (target!==100) schedule();
})();
