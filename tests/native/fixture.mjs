import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import {iconFixture} from './new-tab-fixture.mjs';

/** Controlled loopback pages report only artificial test values; no user browsing data is read. */
export async function createFixture() {
  const loads = new Map();
  const icons=iconFixture();
  const reports = new Map();
  const commands = new Map();
  const download = { version: 1, mode: 'normal', requests: [] };
  const payload = version => Buffer.from(Array.from({ length: 2 * 1024 * 1024 }, (_, i) => (i + version) % 251));
  const server = createServer(async (request, response) => {
    const url = new URL(request.url, 'http://127.0.0.1');
    if(icons.handle(url,response))return;
    if (url.pathname === '/tone.wav') {
      const rate = 8000, count = rate * 2, wave = Buffer.alloc(44 + count * 2);
      wave.write('RIFF'); wave.writeUInt32LE(wave.length - 8, 4); wave.write('WAVEfmt ', 8); wave.writeUInt32LE(16, 16);
      wave.writeUInt16LE(1, 20); wave.writeUInt16LE(1, 22); wave.writeUInt32LE(rate, 24); wave.writeUInt32LE(rate * 2, 28);
      wave.writeUInt16LE(2, 32); wave.writeUInt16LE(16, 34); wave.write('data', 36); wave.writeUInt32LE(count * 2, 40);
      for (let i = 0; i < count; i++) wave.writeInt16LE(Math.round(Math.sin(i * 2 * Math.PI * 440 / rate) * 300), 44 + i * 2);
      response.writeHead(200, { 'Content-Type': 'audio/wav', 'Access-Control-Allow-Origin': '*' }).end(wave); return;
    }
    if (url.pathname === '/download/file') {
      const body = payload(download.version); const etag = `"fixture-${download.version}"`;
      download.requests.push({ range: request.headers.range, ifRange: request.headers['if-range'] });
      let start = 0;
      if (request.headers.range && download.mode !== 'ignore-range' && (!request.headers['if-range'] || request.headers['if-range'] === etag)) start = Number(request.headers.range.match(/^bytes=(\d+)-$/)?.[1] ?? 0);
      if (download.mode === '416' || start >= body.length) { response.writeHead(416, { 'Content-Range': `bytes */${body.length}` }).end(); return; }
      const status = start ? 206 : 200;
      response.writeHead(status, { 'Content-Type': 'application/octet-stream', 'Content-Disposition': 'attachment; filename="fixture.bin"', 'Accept-Ranges': 'bytes', 'ETag': download.mode === 'weak' ? `W/${etag}` : etag, ...(url.searchParams.has('unknown') ? {} : { 'Content-Length': body.length - start }), ...(start ? { 'Content-Range': `bytes ${download.mode === 'bad-range' ? start + 1 : start}-${body.length - 1}/${body.length}` } : {}) });
      let offset = start;
      const timer = setInterval(() => { const end = Math.min(offset + 32768, body.length); response.write(body.subarray(offset, end)); offset = end; if (offset === body.length) { clearInterval(timer); response.end(); } }, url.searchParams.has('slow') ? 150 : 30);
      response.on('close', () => clearInterval(timer)); return;
    }
    const id = url.pathname.split('/')[2] ?? 'a';
    if (!/^[a-z][a-z0-9-]{0,63}$/.test(id)) { response.writeHead(404).end(); return; }
    if (url.pathname.startsWith('/report/')) {
      let body = '';
      for await (const chunk of request) {
        body += chunk;
        if (body.length > 4096) { response.writeHead(413).end(); return; }
      }
      try { reports.set(id, JSON.parse(body)); } catch { response.writeHead(400).end(); return; }
      response.writeHead(204).end(); return;
    }
    if (url.pathname.startsWith('/control/')) {
      const command = commands.get(id) ?? null;
      commands.delete(id);
      response.setHeader('Content-Type', 'application/json');
      response.end(JSON.stringify(command)); return;
    }
    if (url.pathname === '/pip-video.js') {
      response.setHeader('Content-Type','text/javascript; charset=utf-8');
      response.end(readFileSync(new URL('./fixtures/pip-video.js',import.meta.url),'utf8'));return;
    }
    if (url.pathname.startsWith('/page/')) {
      loads.set(id, (loads.get(id) ?? 0) + 1);
      response.setHeader('Content-Type', 'text/html; charset=utf-8');
      response.end(`<!doctype html><title>Native fixture ${id}</title>${id.startsWith('new-tab')?`<link rel="icon" href="/new-tab-icon.png?${id==='new-tab-late'?'delay':id==='new-tab-bad'?'bad':'normal'}">`:''}
        <label>Fixture form <input id="draft"></label><div style="height:3000px"></div>
        <script>
          const id = ${JSON.stringify(id)};
          let measuredGain=null,measuredAnalyser=null;
          if(id.startsWith('media')) {const create=AudioContext.prototype.createGain;AudioContext.prototype.createGain=function(){const gain=create.call(this);measuredGain=gain;measuredAnalyser=this.createAnalyser();measuredAnalyser.fftSize=1024;gain.connect(measuredAnalyser);return gain;};}
          if (id.startsWith('media')) { const audio = document.createElement('audio'); audio.id = 'fixture-audio'; audio.src = '/tone.wav'; audio.loop = true; audio.autoplay = true; document.body.append(audio); audio.play().catch(() => {}); }
          const field = document.getElementById('draft');
          field.value = 'artificial-test-draft';
          if (id === 'a') field.dispatchEvent(new Event('input', { bubbles: true }));
          scrollTo(0, 600);
          let busy = false; let permission = null;
          let hiddenCount = 0, pageClicks = 0;
          let widthTracking = 0, widths = [innerWidth];
          window.addEventListener('resize', () => { widths.push(innerWidth); if(widths.length>32)widths.shift(); });
          document.addEventListener('click', () => pageClicks++);
          document.addEventListener('visibilitychange', () => {
            if (document.hidden) hiddenCount++;
          });
          async function mediaPermission() {
            try { const stream = await navigator.mediaDevices.getUserMedia({ audio: true }); stream.getTracks().forEach(track => track.stop()); permission = 'granted'; }
            catch { permission = 'denied'; }
          }
          if (id === 'i') mediaPermission();
          async function makeVideo(n) {
            const canvas=document.createElement('canvas');canvas.width=320+n*40;canvas.height=180;
            const ctx=canvas.getContext('2d');
            const draw=()=>{ctx.fillStyle=n?'#334466':'#664433';ctx.fillRect(0,0,canvas.width,canvas.height);ctx.fillStyle='white';ctx.fillText('Controlled video '+n+' '+Date.now(),10,30);};
            draw();const timer=setInterval(draw,50);
            const sound=new AudioContext(),oscillator=sound.createOscillator(),gain=sound.createGain(),output=sound.createMediaStreamDestination();
            gain.gain.value=0.001;oscillator.connect(gain).connect(output);oscillator.start();await sound.resume();
            const stream=new MediaStream([...canvas.captureStream(20).getVideoTracks(),...output.stream.getAudioTracks()]);
            const recorder=new MediaRecorder(stream,{mimeType:'video/webm;codecs=vp8,opus'}),chunks=[];
            const recorded=new Promise(resolve=>{recorder.ondataavailable=e=>{if(e.data.size)chunks.push(e.data);};recorder.onstop=resolve;});
            recorder.start();await new Promise(resolve=>setTimeout(resolve,500));recorder.stop();await recorded;
            clearInterval(timer);stream.getTracks().forEach(t=>t.stop());oscillator.stop();await sound.close();
            const video=document.createElement('video');video.title='Controlled video '+n;video.loop=true;
            video.src=URL.createObjectURL(new Blob(chunks,{type:recorder.mimeType}));document.body.prepend(video);await video.play();
          }
          async function tick() {
            if (busy) return; busy = true;
            try {
              const command = await fetch('/control/' + id).then(response => response.json());
              if (command === 'reset-widths') { widthTracking++; widths = [innerWidth]; }
              if (command === 'native-player-volume') {const el=document.getElementById('fixture-audio');Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype,'volume').set.call(el,0.8);}
              if (command === 'restore-player-volume') document.getElementById('fixture-audio').volume=1;
              if (command === 'native-controls') {const el=document.getElementById('fixture-audio');el.controls=true;el.src='/tone.wav';await el.play();}
              if (command === 'oscillator') {document.querySelectorAll('audio,video').forEach(el=>el.pause());const c=new AudioContext();const o=c.createOscillator();const g=c.createGain();g.gain.value=0.001;o.connect(g).connect(c.destination);o.start();await c.resume();}
              if (command === 'videos') {for(let n=0;n<2;n++)await makeVideo(n);}
              if (command === 'disable-pip') document.querySelectorAll('video').forEach(v=>v.disablePictureInPicture=true);
              if (command === 'exit-pip') {if(document.pictureInPictureElement)await document.exitPictureInPicture();}
              if (command === 'media-frame') {const f=document.createElement('iframe');f.src=location.origin.replace('127.0.0.1','localhost')+'/page/media-frame';document.body.prepend(f);}
              if (command === 'cross-audio') {const el=document.getElementById('fixture-audio');el.src=location.origin.replace('127.0.0.1','localhost')+'/tone.wav';el.play();}
              if (command === 'stop-media') document.querySelectorAll('audio,video').forEach(el => el.pause());
              if (command === 'dirty') { field.value += ' edited'; field.dispatchEvent(new Event('input', { bubbles: true })); }
              if (command === 'spa') { history.pushState({}, '', '/page/' + id + '/spa'); document.title = 'Native SPA ' + id; }
              if (command === 'microphone') { permission = null; void mediaPermission(); }
              if (command === 'iframe') { const frame = document.createElement('iframe'); frame.allow = 'microphone'; frame.src = location.origin.replace('127.0.0.1', 'localhost') + '/page/i'; document.body.prepend(frame); }
              if (command === 'remove-iframe') document.querySelectorAll('iframe').forEach(frame => frame.remove());
              if (command === 'download') { const a = document.createElement('a'); a.href = '/download/file?native=1'; a.download = 'fixture.bin'; a.click(); }
              const samples=new Float32Array(1024);measuredAnalyser?.getFloatTimeDomainData(samples);const rms=Math.sqrt(samples.reduce((sum,n)=>sum+n*n,0)/samples.length);const audio=document.getElementById('fixture-audio');
              await fetch('/report/' + id, { method: 'POST', body: JSON.stringify({ videoCount:document.querySelectorAll('video').length,pip:Boolean(document.pictureInPictureElement),videoTime:document.pictureInPictureElement?.currentTime,audioRms:rms,audioGain:measuredGain?.gain.value,nativeVolume:audio?Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype,'volume').get.call(audio):null,nativeMuted:audio?Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype,'muted').get.call(audio):null,playerVolume:audio?.volume,value: field.value, width: innerWidth, widths, widthTracking, pixelRatio: devicePixelRatio, focused: document.hasFocus(), scroll: scrollY, scrollAnchor: document.querySelector('div[style*="3000px"]').getBoundingClientRect().top, pageClicks, url: location.pathname + location.search, dirty: id === 'a', permission, visibility: document.visibilityState, hiddenCount, reportedAt: Date.now() }) });
            } finally { busy = false; }
          }
          setInterval(tick, 200); tick();
        </script>${['media-pip-continuity','media-pip-fps'].includes(id)?'<script src="/pip-video.js"></script>':''}`);
      return;
    }
    response.writeHead(404).end();
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  return { origin: `http://127.0.0.1:${server.address().port}`, loads, reports, commands, download, payload,icons,
    close: () => new Promise(resolve => { server.closeAllConnections(); server.close(resolve); }) };
}
