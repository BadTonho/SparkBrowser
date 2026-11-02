// Executed in the source frame; only the silent video track enters the popup.
(popup, source, stream, config, send, closed) => {
  const doc = popup.document;
  // Background source tabs throttle their timers; the visible player owns UI time.
  const setTimeout=popup.setTimeout.bind(popup),clearTimeout=popup.clearTimeout.bind(popup);
  const setInterval=popup.setInterval.bind(popup),clearInterval=popup.clearInterval.bind(popup);
  const labels = {play:'Play',pause:'Pause',back:'Back 10 seconds',forward:'Forward 10 seconds',progress:'Playback position',volume:'Volume',return:'Return to tab',close:'Close',fallback:'Custom floating controls are unavailable for this player.',...config.presentation?.labels};
  doc.documentElement.lang = config.presentation?.locale || 'en';
  doc.title = config.title || 'Floating video';
  const sheet = new popup.CSSStyleSheet();
  sheet.replaceSync(`
    *{box-sizing:border-box} html,body{margin:0;width:100%;height:100%;overflow:hidden;background:#080808;color:#fff;font:12px system-ui,sans-serif}
    video{width:100%;height:100%;object-fit:contain;display:block}button{height:30px;min-width:32px;border:0;border-radius:5px;background:#262626;color:white;cursor:pointer;font:14px system-ui}
    button:hover{background:#444}button:focus-visible,input:focus-visible{outline:2px solid #8cbcff;outline-offset:2px}
    .controls{position:absolute;bottom:0;left:0;right:0;padding:32px 10px 8px;background:linear-gradient(transparent,#000c);display:flex;align-items:center;gap:6px;flex-wrap:wrap}
    .progress{width:100%;height:16px;margin:0;accent-color:#6da9ff}.time{margin-right:auto;font-variant-numeric:tabular-nums}
    .volume-area{position:relative;margin-left:auto}.volume-panel{position:absolute;bottom:36px;right:0;width:180px;max-width:calc(100vw - 20px);height:64px;padding:8px 12px;background:#1c1c1c;border:1px solid #555;border-radius:8px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:4px}
    .volume-panel[hidden]{display:none}.volume-panel input{width:100%;height:24px;margin:0;accent-color:#6da9ff}.percentage{font:12px/16px system-ui;color:#d0d0d4;font-variant-numeric:tabular-nums}input:disabled{opacity:.45}
  `);
  doc.adoptedStyleSheets = [sheet];
  function el(tag, parent, properties={}) { const node=doc.createElement(tag);Object.assign(node,properties);parent.append(node);return node; }
  const video=el('video',doc.body,{muted:true,autoplay:true,playsInline:true,srcObject:stream});
  video.setAttribute('data-testid','pip-image');
  const controls=el('div',doc.body,{className:'controls'});
  const progress=el('input',controls,{className:'progress',type:'range',min:'0',max:'1',step:'0.1',value:'0',disabled:true});
  progress.setAttribute('aria-label',labels.progress);
  const time=el('span',controls,{className:'time'});
  function button(parent,text,name,testid) {const b=el('button',parent,{type:'button',textContent:text,title:name});b.setAttribute('aria-label',name);if(testid)b.setAttribute('data-testid',testid);return b;}
  const back=button(controls,'−10',labels.back,'pip-back');
  const play=button(controls,'▶',labels.play,'pip-play');
  const forward=button(controls,'+10',labels.forward,'pip-forward');
  const area=el('div',controls,{className:'volume-area'});
  const speaker=button(area,'♪',labels.volume,'pip-speaker');
  speaker.innerText='🔊';
  speaker.setAttribute('aria-expanded','false');speaker.setAttribute('aria-controls','pip-volume-panel');
  const panel=el('div',area,{id:'pip-volume-panel',className:'volume-panel',hidden:true});
  panel.setAttribute('role','group');panel.setAttribute('aria-label',labels.volume);
  const volume=el('input',panel,{type:'range',min:'0',max:'200',step:'1',value:String(config.volume ?? 100)});
  volume.setAttribute('data-testid','pip-volume');volume.setAttribute('aria-label',labels.volume);
  const percentage=el('span',panel,{className:'percentage'});percentage.setAttribute('aria-hidden','true');percentage.setAttribute('data-testid','pip-volume-percentage');
  const returnButton=button(controls,'↗',labels.return,'pip-return');
  const closeButton=button(controls,'×',labels.close,'pip-close');
  let disposed=false,dragging=false,editing=false,timer,latest=Number(volume.value),sent=0,acknowledged=0,hover=false,suppressFocus=false;
  const show=()=>{clearTimeout(timer);panel.hidden=false;speaker.setAttribute('aria-expanded','true');};
  const hide=()=>{clearTimeout(timer);panel.hidden=true;speaker.setAttribute('aria-expanded','false');};
  const scheduleHide=()=>{clearTimeout(timer);timer=setTimeout(()=>{if(!dragging&&!hover&&!area.contains(doc.activeElement))hide();},300);};
  function renderVolume(){percentage.textContent=`${volume.value}%`;volume.setAttribute('aria-valuetext',percentage.textContent);}
  function change(final=false){if(volume.disabled)return;latest=Number(volume.value);editing=true;renderVolume();sent=send(final?'flush':'volume',latest);}
  area.addEventListener('pointerenter',()=>{hover=true;show();});
  area.addEventListener('pointerleave',()=>{hover=false;scheduleHide();});
  area.addEventListener('focusin',()=>{if(!suppressFocus)show();});
  area.addEventListener('focusout',scheduleHide);
  speaker.addEventListener('click',show);
  volume.addEventListener('pointerdown',()=>{dragging=true;show();});
  volume.addEventListener('input',()=>change());
  volume.addEventListener('change',()=>change(true));
  const release=()=>{if(!dragging)return;dragging=false;change(true);scheduleHide();};
  popup.addEventListener('pointerup',release);popup.addEventListener('pointercancel',release);
  doc.addEventListener('keydown',event=>{if(event.key==='Escape'&&!panel.hidden){event.preventDefault();event.stopPropagation();hide();suppressFocus=true;speaker.focus();suppressFocus=false;}});
  popup.__velocePipState = state => {
    if(disposed||state.session!==config.session)return;
    acknowledged=Math.max(acknowledged,state.sequence??0);
    volume.disabled=Boolean(state.disabled);
    if(!dragging&&(!editing||acknowledged>=sent)){volume.value=String(state.volume);latest=Number(volume.value);editing=false;renderVolume();}
    speaker.innerText=state.muted||Number(volume.value)===0?'🔇':'🔊';
  };
  const format=t=>Number.isFinite(t)?`${Math.floor(t/60)}:${String(Math.floor(t%60)).padStart(2,'0')}`:'0:00';
  function update(){
    if(disposed)return;
    play.textContent=source.paused?'▶':'Ⅱ';play.title=source.paused?labels.play:labels.pause;play.setAttribute('aria-label',play.title);
    const ranges=source.seekable;
    const start=ranges.length?ranges.start(0):0,end=ranges.length?ranges.end(ranges.length-1):Number.isFinite(source.duration)?source.duration:0;
    progress.disabled=end<=start;progress.min=String(start);progress.max=String(Math.max(start,end));progress.value=String(source.currentTime);
    time.textContent=`${format(source.currentTime)} / ${format(source.duration)}`;
    if(!source.isConnected)send('close');
  }
  function seek(value){const n=Number(value);if(Number.isFinite(n)&&!progress.disabled)source.currentTime=Math.max(Number(progress.min),Math.min(Number(progress.max),n));}
  back.addEventListener('click',()=>seek(source.currentTime-10));forward.addEventListener('click',()=>seek(source.currentTime+10));
  play.addEventListener('click',()=>{if(source.paused)void source.play().catch(()=>{});else source.pause();});
  progress.addEventListener('input',()=>seek(progress.value));
  returnButton.addEventListener('click',()=>send('return'));
  closeButton.addEventListener('click',()=>send('close',latest));
  popup.__velocePipRequestClose=()=>send('close',latest);
  const events=['play','pause','timeupdate','durationchange','progress','loadedmetadata','ended'];events.forEach(e=>source.addEventListener(e,update));
  function cleanup(){if(disposed)return;disposed=true;clearTimeout(timer);clearInterval(watch);events.forEach(e=>source.removeEventListener(e,update));popup.removeEventListener('pointerup',release);popup.removeEventListener('pointercancel',release);stream.getTracks().forEach(t=>t.stop());video.srcObject=null;closed();}
  popup.addEventListener('pagehide',cleanup,{once:true});
  const watch=setInterval(update,500);
  renderVolume();update();void video.play().catch(()=>{});send('ready');
  return cleanup;
}
