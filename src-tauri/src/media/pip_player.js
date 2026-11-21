// Executed in the source frame; only the silent video track enters the popup.
(popup, source, stream, config, send, closed) => {
  const doc = popup.document;
  const setTimeout = popup.setTimeout.bind(popup), clearTimeout = popup.clearTimeout.bind(popup);
  const setInterval = popup.setInterval.bind(popup), clearInterval = popup.clearInterval.bind(popup);
  const labels = {play:'Play',pause:'Pause',back:'Back 10 seconds',forward:'Forward 10 seconds',progress:'Playback position',volume:'Volume',return:'Return to tab',close:'Close',minimize:'Minimize',...config.presentation?.labels};
  const icon = __PIP_ICONS__;
  const createCapture = config.canvas ? __PIP_CANVAS__ : __PIP_CAPTURE__;
  const sheet = new popup.CSSStyleSheet();
  sheet.replaceSync(__PIP_PLAYER_STYLE__);
  doc.adoptedStyleSheets = [sheet];
  doc.documentElement.lang = config.presentation?.locale || 'en';
  doc.title = config.title || 'Floating video';
  function el(tag, parent, properties = {}) { const node=doc.createElement(tag);Object.assign(node,properties);parent.append(node);return node; }
  function setIcon(button, name) { if(button.getAttribute('data-icon')===name)return;button.replaceChildren(icon(doc,name));button.setAttribute('data-icon',name); }
  function button(parent, name, label, id) { const node=el('button',parent,{type:'button',title:label});node.setAttribute('aria-label',label);node.setAttribute('data-testid',id);setIcon(node,name);return node; }
  const video=config.canvas?el('canvas',doc.body):el('video',doc.body,{muted:true,autoplay:true,playsInline:true});
  video.setAttribute('data-testid','pip-image');
  const chrome=el('div',doc.body,{className:'chrome'});chrome.setAttribute('data-testid','pip-chrome');chrome.setAttribute('data-visible','true');
  const header=el('div',chrome,{className:'header'});
  const dragArea=el('div',header,{className:'drag-area'});dragArea.setAttribute('data-testid','pip-drag');dragArea.append(icon(doc,'site'));
  el('span',dragArea,{className:'origin',textContent:config.hostname || '',title:config.hostname || ''}).setAttribute('data-testid','pip-origin');
  const windowButtons=el('div',header,{className:'window-buttons'});
  const minimize=button(windowButtons,'minimize',labels.minimize,'pip-minimize');
  const returnButton=button(windowButtons,'return',labels.return,'pip-return');
  const closeButton=button(windowButtons,'close',labels.close,'pip-close');
  const transport=el('div',chrome,{className:'transport'});
  const back=button(transport,'back',labels.back,'pip-back');
  const play=button(transport,'play',labels.play,'pip-play');play.className='play';
  const forward=button(transport,'forward',labels.forward,'pip-forward');
  const footer=el('div',chrome,{className:'footer'});
  const progress=el('input',footer,{className:'progress',type:'range',min:'0',max:'1',step:'0.1',value:'0',disabled:true});
  progress.setAttribute('aria-label',labels.progress);progress.setAttribute('data-testid','pip-progress');
  const row=el('div',footer,{className:'bottom-row'});
  const time=el('span',row,{className:'time'});
  const area=el('div',row,{className:'volume-area'});
  const speaker=button(area,'speaker',labels.volume,'pip-speaker');
  speaker.setAttribute('aria-expanded','false');speaker.setAttribute('aria-controls','pip-volume-panel');
  const panel=el('div',area,{id:'pip-volume-panel',className:'volume-panel',hidden:true});
  panel.setAttribute('role','group');panel.setAttribute('aria-label',labels.volume);
  const percentage=el('span',panel,{className:'percentage'});percentage.setAttribute('aria-hidden','true');percentage.setAttribute('data-testid','pip-volume-percentage');
  const volume=el('input',panel,{type:'range',min:'0',max:'200',step:'1',value:String(config.volume ?? 100)});
  volume.setAttribute('data-testid','pip-volume');volume.setAttribute('aria-label',labels.volume);

  let disposed=false,dragging=false,seeking=false,windowDragging=false,editing=false,hover=false,suppressFocus=false;
  let volumeTimer,idleTimer,focusTimer,renderTimer,capture,latest=Number(volume.value),sent=0,acknowledged=0,lastPaused=source.paused;
  const removers=[];
  function listen(target,type,callback) {target.addEventListener(type,callback);removers.push(()=>target.removeEventListener(type,callback));}
  const pinned=()=>source.paused||dragging||seeking||windowDragging||!panel.hidden||chrome.contains(doc.activeElement);
  function wake() {
    if(disposed)return;
    clearTimeout(idleTimer);
    if(chrome.getAttribute('data-visible')!=='true'){chrome.setAttribute('data-visible','true');update();}
    if(!pinned())idleTimer=setTimeout(()=>{if(!pinned())chrome.setAttribute('data-visible','false');},2000);
  }
  function showVolume() {clearTimeout(volumeTimer);panel.hidden=false;row.className='bottom-row volume-open';speaker.setAttribute('aria-expanded','true');wake();}
  function hideVolume() {clearTimeout(volumeTimer);panel.hidden=true;row.className='bottom-row';speaker.setAttribute('aria-expanded','false');wake();}
  function scheduleVolumeHide() {clearTimeout(volumeTimer);volumeTimer=setTimeout(()=>{if(!dragging&&!hover&&!area.contains(doc.activeElement))hideVolume();},300);}
  function renderVolume() {percentage.textContent=`${volume.value}%`;volume.setAttribute('aria-valuetext',percentage.textContent);volume.style.setProperty('--volume-progress',`${Number(volume.value)/2}%`);}
  function change(final=false) {if(volume.disabled)return;latest=Number(volume.value);editing=true;renderVolume();sent=send(final?'flush':'volume',latest);wake();}
  function action(node, callback) {listen(node,'click',event=>{callback(event);if(event.detail>0&&node!==speaker)node.blur();wake();});}
  listen(doc,'pointermove',wake);listen(doc,'pointerdown',wake);listen(doc,'focusin',wake);
  listen(doc,'focusout',()=>{clearTimeout(focusTimer);focusTimer=setTimeout(wake,0);});
  listen(popup,'focus',wake);
  listen(area,'pointerenter',()=>{hover=true;showVolume();});
  listen(area,'pointerleave',()=>{hover=false;scheduleVolumeHide();});
  listen(area,'focusin',()=>{if(!suppressFocus)showVolume();});
  listen(area,'focusout',scheduleVolumeHide);
  action(speaker,showVolume);
  listen(volume,'pointerdown',()=>{dragging=true;showVolume();});
  listen(volume,'input',()=>change());listen(volume,'change',()=>change(true));
  listen(progress,'pointerdown',()=>{seeking=true;wake();});
  const release=()=>{if(dragging){dragging=false;change(true);scheduleVolumeHide();}seeking=false;windowDragging=false;wake();};
  listen(popup,'pointerup',release);listen(popup,'pointercancel',release);
  listen(dragArea,'pointerdown',event=>{if(event.button===0){event.preventDefault();windowDragging=true;wake();send('drag');}});
  popup.__sparkPipDragFinished=()=>{windowDragging=false;wake();};
  listen(doc,'keydown',event=>{wake();if(event.key==='Escape'&&!panel.hidden){event.preventDefault();event.stopPropagation();hideVolume();suppressFocus=true;speaker.focus();suppressFocus=false;}});
  popup.__sparkPipState=state=>{
    if(disposed||state.session!==config.session)return;
    acknowledged=Math.max(acknowledged,state.sequence??0);volume.disabled=Boolean(state.disabled);
    if(!dragging&&(!editing||acknowledged>=sent)){volume.value=String(state.volume);latest=Number(volume.value);editing=false;renderVolume();}
    setIcon(speaker,state.muted||Number(volume.value)===0?'muted':'speaker');
  };
  const format=t=>Number.isFinite(t)?`${Math.floor(t/60)}:${String(Math.floor(t%60)).padStart(2,'0')}`:'0:00';
  function update() {
    if(disposed)return;
    if(!source.isConnected){send('close',latest);return;}
    if(lastPaused!==source.paused){lastPaused=source.paused;wake();}
    if(chrome.getAttribute('data-visible')==='false')return;
    setIcon(play,source.paused?'play':'pause');play.title=source.paused?labels.play:labels.pause;play.setAttribute('aria-label',play.title);
    const ranges=source.seekable;
    const start=ranges.length?ranges.start(0):0,end=ranges.length?ranges.end(ranges.length-1):Number.isFinite(source.duration)?source.duration:0;
    progress.disabled=end<=start;progress.min=String(start);progress.max=String(Math.max(start,end));
    if(!seeking)progress.value=String(source.currentTime);
    progress.style.setProperty('--progress',`${end>start?Math.max(0,Math.min(100,(source.currentTime-start)/(end-start)*100)):0}%`);
    time.textContent=`${format(source.currentTime)} / ${format(source.duration)}`;
  }
  function scheduleUpdate() {if(!disposed&&renderTimer===undefined)renderTimer=setTimeout(()=>{renderTimer=undefined;update();},100);}
  function seek(value) {const n=Number(value);if(Number.isFinite(n)&&!progress.disabled)source.currentTime=Math.max(Number(progress.min),Math.min(Number(progress.max),n));wake();}
  action(back,()=>seek(source.currentTime-10));action(forward,()=>seek(source.currentTime+10));
  action(play,()=>{if(source.paused)void source.play().catch(()=>{});else source.pause();});
  listen(progress,'input',()=>seek(progress.value));
  action(minimize,()=>send('minimize'));action(returnButton,()=>send('return',latest));action(closeButton,()=>send('close',latest));
  popup.__sparkPipRequestClose=()=>send('close',latest);
  for(const event of ['play','pause','ended'])listen(source,event,update);
  for(const event of ['timeupdate','durationchange','progress','loadedmetadata'])listen(source,event,scheduleUpdate);
  function cleanup() {if(disposed)return;disposed=true;clearTimeout(volumeTimer);clearTimeout(idleTimer);clearTimeout(focusTimer);clearTimeout(renderTimer);clearInterval(watch);removers.forEach(stop=>stop());capture?.();closed();}
  listen(popup,'pagehide',cleanup);
  const watch=setInterval(scheduleUpdate,500);
  renderVolume();update();wake();capture=createCapture(popup,source,video,stream,()=>send('capture-failed',latest));send('ready');
  return cleanup;
}
