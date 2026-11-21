(() => {
  const $ = id => document.getElementById(id);
  const canvas = $('audio-background'), context = canvas?.getContext('2d');
  const defaults = { enabled:false, color:'#1db954', intensity:1, opacity:1, style:'bars' };
  const reduced = window.matchMedia?.('(prefers-reduced-motion: reduce)');
  let config = defaults, labels = {}, privateWindow = false, subscribed = false, raf = 0, expiry = 0;
  let target = Array(32).fill(0), displayed = Array(32).fill(0), last = 0, lastPaint = 0, status = 'idle';
  let styleStore = {}, styleId = 'bars';
  const styleFor = id => (id === 'bars' ? null : window.__sparkAudioStyles?.[id] ?? null);
  const text = key => labels[`newtab.widgets.${key}`] ?? key;
  const send = operation => window.dispatchEvent(new CustomEvent('spark-new-tab-action', { detail:{ action:'widgets', operation, data:{} } }));
  const change = detail => window.dispatchEvent(new CustomEvent('spark-audio-setting', { detail }));
  const active = () => config.enabled && !privateWindow && !document.hidden && Boolean(context);
  function clear() {
    if (raf) window.cancelAnimationFrame(raf); raf = 0;
    window.clearTimeout(expiry); expiry = 0; last = 0;
    target.fill(0); displayed.fill(0); styleStore = {};
    context?.clearRect(0,0,canvas.width,canvas.height); canvas.hidden = true;
  }
  // 100% is where the slider used to end (intensity 2); it now goes up to 200%.
  const percent = (value, unit = 1) => `${Math.round(Number(value) * unit * 100)}%`;
  function showValues() {
    const intensity = $('audio-background-intensity-value'), opacity = $('audio-background-opacity-value');
    if (intensity) intensity.textContent = percent(config.intensity, 0.5);
    if (opacity) opacity.textContent = percent(config.opacity ?? 1);
  }
  function applyOpacity() { canvas?.style?.setProperty?.('--audio-opacity', String(config.opacity ?? 1)); }
  function resize() {
    const ratio = Math.min(window.devicePixelRatio || 1, 2);
    const full = styleFor(config.style)?.layout === 'full';
    canvas.width = Math.round(window.innerWidth * ratio);
    canvas.height = Math.round(window.innerHeight * (full ? 1 : 0.25) * ratio);
    if (canvas.style) canvas.style.height = full ? '100vh' : '25vh';
  }
  function statusText() {
    const node = $('audio-background-status');
    const message = status === 'audio_unavailable' ? text('audioBackgroundUnavailable') : '';
    if (node.textContent !== message) node.textContent = message;
  }
  function paint(now) {
    raf = 0;
    if (!active()) { clear(); return; }
    const interval = reduced?.matches ? 100 : 33;
    if (now - lastPaint < interval) { raf = window.requestAnimationFrame(paint); return; }
    const elapsed = Math.min(now - (last || now - interval), 100); last = now; lastPaint = now;
    // Intensity makes the effect react more (as it always did) and, on top of that, grow bigger.
    const gain = Math.sqrt(config.intensity);
    let peak = 0;
    for (let i=0;i<32;i++) {
      const goal = Math.min(1,target[i] * config.intensity);
      if (reduced?.matches) displayed[i] = goal;
      else displayed[i] += (goal-displayed[i]) * (1-Math.exp(-elapsed/(goal>displayed[i]?70:100)));
      if (displayed[i]<0.004) displayed[i]=0;
      peak = Math.max(peak,displayed[i]);
    }
    context.clearRect(0,0,canvas.width,canvas.height);
    canvas.hidden = peak === 0;
    const style = styleFor(config.style);
    if (peak > 0 && style) {
      style.draw(context, canvas.width, canvas.height, displayed, { color:config.color, gain, t:now/1000, dt:elapsed/1000, store:styleStore });
      if (!reduced?.matches) raf = window.requestAnimationFrame(paint);
    } else if (peak > 0) {
      const gradient = context.createLinearGradient(0,canvas.height,0,0);
      gradient.addColorStop(0,config.color); gradient.addColorStop(1,`${config.color}20`);
      context.fillStyle = gradient;
      const slot = canvas.width/32, gap = Math.max(2,slot*0.15);
      for(let i=0;i<32;i++) {
        const height = Math.min(canvas.height,displayed[i]*gain*canvas.height*0.9);
        context.fillRect(i*slot+gap/2,canvas.height-height,slot-gap,height);
      }
      if (!reduced?.matches) raf = window.requestAnimationFrame(paint);
    }
  }
  function animate() {
    if (!raf && active() && (target.some(v=>v>0) || displayed.some(v=>v>0))) raf = window.requestAnimationFrame(paint);
  }
  function subscription() {
    const wanted = active();
    if (wanted !== subscribed) { subscribed = wanted; send(wanted?'audio_subscribe':'audio_unsubscribe'); }
    if (!wanted) clear();
  }
  $('audio-background-enabled').addEventListener('change',event=>change({enabled:event.target.checked}));
  $('audio-background-color').addEventListener('change',event=>change({color:event.target.value}));
  // The effect follows the slider while it moves; the value is saved when it is released.
  $('audio-background-intensity').addEventListener('input',event=>{ config = { ...config, intensity:Number(event.target.value) }; showValues(); animate(); });
  $('audio-background-intensity').addEventListener('change',event=>change({intensity:Number(event.target.value)}));
  $('audio-background-opacity').addEventListener('input',event=>{ config = { ...config, opacity:Number(event.target.value) }; showValues(); applyOpacity(); });
  $('audio-background-opacity').addEventListener('change',event=>change({opacity:Number(event.target.value)}));
  $('audio-background-style').addEventListener('change',event=>change({style:event.target.value}));
  window.__sparkAudioRender = (value, saving=false) => {
    config = { ...defaults, ...value };
    if (config.style !== styleId) { styleId = config.style; styleStore = {}; resize(); }
    $('audio-background-settings').hidden = privateWindow;
    $('audio-background-enabled').checked = config.enabled && !privateWindow;
    $('audio-background-options').hidden = !config.enabled || privateWindow;
    $('audio-background-color').value = config.color;
    $('audio-background-intensity').value = String(config.intensity);
    $('audio-background-opacity').value = String(config.opacity);
    applyOpacity(); showValues();
    $('audio-background-style').value = config.style;
    $('audio-background-enabled').disabled = saving || !context;
    $('audio-background-color').disabled = saving;
    $('audio-background-intensity').disabled = saving;
    $('audio-background-opacity').disabled = saving;
    $('audio-background-style').disabled = saving;
    subscription(); statusText(); animate();
  };
  window.addEventListener('spark-new-tab',event=>{
    labels=event.detail.labels; privateWindow=Boolean(event.detail.private);
    window.__sparkAudioRender(config);
  });
  window.addEventListener('spark-audio-frame',event=>{
    if (!subscribed || !active()) return;
    const frame=event.detail;
    if (!Array.isArray(frame.bands) || frame.bands.length!==32 || frame.bands.some(v=>!Number.isFinite(v)||v<0||v>1)) return;
    status=frame.status; statusText();
    target=frame.status==='ready'?[...frame.bands]:Array(32).fill(0);
    window.clearTimeout(expiry);
    expiry=window.setTimeout(()=>{target.fill(0);animate();},750);
    animate();
  });
  document.addEventListener('visibilitychange',subscription);
  window.addEventListener('pagehide',()=>{if(subscribed)send('audio_unsubscribe');subscribed=false;clear();});
  window.addEventListener('pageshow',subscription);
  window.addEventListener('resize',()=>{resize();animate();});
  reduced?.addEventListener('change',()=>{clear();animate();});
  resize();
})();
