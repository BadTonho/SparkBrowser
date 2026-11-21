import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import {iconFixture} from './new-tab-fixture.mjs';

/** Controlled loopback pages report only artificial test values; no user browsing data is read. */
export async function createFixture() {
  const loads = new Map();
  const icons=iconFixture();
  const reports = new Map();
  const gpcRequests = [];
  const shieldRequests = [];
  const trackingRequests = [];
  const feed = { invalid: false, requests: 0, body: null };
  const commands = new Map();
  const download = { version: 1, mode: 'normal', requests: [], images: new Map() };
  const payload = version => Buffer.from(Array.from({ length: 2 * 1024 * 1024 }, (_, i) => (i + version) % 251));
  const server = createServer(async (request, response) => {
    if (request.url === '/account-cookie') {
      response.writeHead(204, { 'Set-Cookie': 'fixture_account=alpha; Max-Age=3600; Path=/', 'Cache-Control': 'no-store' }).end(); return;
    }
    if (request.url === '/fixture-feed.xml') {
      feed.requests++;
      response.writeHead(200, { 'Content-Type': 'application/rss+xml', 'Cache-Control': 'no-store' }).end(feed.invalid ? '<rss><channel>' : feed.body ?? '<rss><channel><item><title><![CDATA[Fixture article]]></title><link>/page/feed-article</link><description>&lt;script&gt;not executed&lt;/script&gt;</description></item></channel></rss>'); return;
    }
    if (request.url === '/page/tracking-redirect') {
      response.writeHead(302, { Location: '/page/link-target?utm_source=redirect&keep=redirect', 'Cache-Control': 'no-store' }).end(); return;
    }
    if (request.url === '/page/signed-redirect') {
      response.writeHead(302, { Location: '/page/link-target?utm_source=signed&state=opaque', 'Cache-Control': 'no-store' }).end(); return;
    }
    if (request.url === '/page/post-redirect') {
      response.writeHead(307, { Location: '/page/post-target?utm_source=post-redirect', 'Cache-Control': 'no-store' }).end(); return;
    }
    if (request.url === '/page/link-source') {
      response.writeHead(200, { 'Content-Type': 'text/html', 'Cache-Control': 'no-store' }).end('<!doctype html><title>Tracking fixture</title><a id="tracking-link" href="/page/link-target?utm_source=fixture&amp;keep=1">Link</a><form method="post" action="/page/post-target?utm_source=fixture"><button id="post-button">Send</button></form><form method="post" action="/page/post-redirect"><button id="post-redirect-button">Redirect POST</button></form>'); return;
    }
    if (request.url?.startsWith('/page/link-target') || request.url?.startsWith('/page/post-target')) {
      trackingRequests.push({ url: request.url, method: request.method });
      response.writeHead(200, { 'Content-Type': 'text/html', 'Cache-Control': 'no-store' }).end('<!doctype html><title>Destination</title>'); return;
    }
    if (request.url === '/page/filter-test') {
      response.writeHead(200, { 'Content-Type': 'text/html', 'Cache-Control': 'no-store' }).end('<!doctype html><title>Filter fixture</title><script src="/ads-native.js"></script><script src="/ordinary-script.js"></script>'); return;
    }
    if (request.url === '/ads-native.js' || request.url === '/ordinary-script.js') {
      shieldRequests.push(request.url);
      response.writeHead(200, { 'Content-Type': 'text/javascript', 'Cache-Control': 'no-store' }).end('window.fixtureLoaded = true'); return;
    }
    if (request.url?.startsWith('/page/gpc') || request.url?.startsWith('/gpc-resource/')) gpcRequests.push({ url: request.url, header: request.headers['sec-gpc'] });
    if (request.url === '/page/gpc-contexts') {
      response.writeHead(200, { 'Content-Type': 'text/html', 'Cache-Control': 'no-store' }).end('<!doctype html><title>GPC contexts</title><iframe src="/gpc-resource/frame"></iframe><script>fetch("/gpc-resource/main-fetch"); new Worker("/gpc-resource/worker.js");</script>'); return;
    }
    if (request.url === '/gpc-resource/frame') {
      response.writeHead(200, { 'Content-Type': 'text/html', 'Cache-Control': 'no-store' }).end('<!doctype html><script>fetch("/gpc-resource/frame-fetch")</script>'); return;
    }
    if (request.url === '/gpc-resource/worker.js') {
      response.writeHead(200, { 'Content-Type': 'text/javascript', 'Cache-Control': 'no-store' }).end('fetch("/gpc-resource/worker-fetch")'); return;
    }
    if (request.url?.startsWith('/gpc-resource/')) { response.writeHead(200, { 'Cache-Control': 'no-store' }).end('ok'); return; }
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
    if (url.pathname.startsWith('/download/visual/')) {
      const fixture = download.images.get(decodeURIComponent(url.pathname.slice('/download/visual/'.length)));
      if (!fixture) { response.writeHead(404).end(); return; }
      response.writeHead(200, { 'Content-Type': fixture.mime, 'Content-Length': fixture.bytes.length, 'Content-Disposition': 'attachment' });
      response.end(fixture.bytes); return;
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
    if (url.pathname === '/clip.webm') {
      // Artificial local video for engines that cannot play MediaRecorder output.
      const clip = readFileSync(new URL('./fixtures/clip.webm', import.meta.url));
      const range = /^bytes=(\d+)-(\d*)$/.exec(request.headers.range ?? '');
      response.setHeader('Content-Type', 'video/webm'); response.setHeader('Accept-Ranges', 'bytes');
      if (range) {
        const start = Number(range[1]), end = range[2] ? Number(range[2]) : clip.length - 1;
        response.writeHead(206, { 'Content-Range': `bytes ${start}-${end}/${clip.length}`, 'Content-Length': end - start + 1 });
        response.end(clip.subarray(start, end + 1));
      } else { response.setHeader('Content-Length', clip.length); response.end(clip); }
      return;
    }
    if (url.pathname === '/pip-video.js') {
      response.setHeader('Content-Type','text/javascript; charset=utf-8');
      response.end(readFileSync(new URL('./fixtures/pip-video.js',import.meta.url),'utf8'));return;
    }
    if (url.pathname === '/capture-probe') {
      response.setHeader('Content-Type', 'text/html; charset=utf-8');
      response.end(`<!doctype html><meta charset="utf-8"><title>Full page capture probe</title>
        <style>
          *{box-sizing:border-box}body{margin:0;font:20px system-ui;background:#f4f4f4;color:#17202a}
          header{position:fixed;z-index:2;inset:0 0 auto;height:58px;padding:16px;background:#173f5f;color:white}
          main{padding-top:58px}.section{height:900px;padding:32px;border-bottom:4px solid #222}
          .section:nth-child(even){background:#b9e5d2}.section:nth-child(odd){background:#f6d186}
          .sticky{position:sticky;top:58px;padding:10px;background:#b33;color:white}
          img{display:block;width:120px;height:80px;margin:18px 0;background:#fff}
          iframe{display:block;width:90%;height:360px;border:5px solid #324}
          footer{position:fixed;z-index:2;inset:auto 0 0;padding:14px;background:#173f5f;color:white}
          #bottom-marker{height:180px;background:#28a745;color:#fff;padding:28px}
        </style>
        <header id="fixed-header">Fixed header</header><main id="capture-main">
        <section class="section"><div class="sticky">Sticky marker</div><h1>Capture fixture: top</h1><p>Scrollable document test</p><img id="lazy-image" loading="lazy" src="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='120' height='80'%3E%3Crect width='120' height='80' fill='%230078d4'/%3E%3C/svg%3E"></section>
        <section class="section"><h2>Middle section A</h2></section><section class="section"><h2>Middle section B</h2><iframe title="Long iframe" srcdoc="&lt;style&gt;body{margin:0;background:%23d8c4f1}div{height:700px;padding:20px;font:18px sans-serif}&lt;/style&gt;&lt;div&gt;iframe top&lt;/div&gt;&lt;div&gt;iframe bottom&lt;/div&gt;"></iframe></section>
        <section class="section"><h2>Near bottom</h2><p id="dynamic-marker">Dynamic text: initial</p><div style="height:500px"></div></section>
        <div id="bottom-marker">BOTTOM MARKER — expected near the end of the captured image</div></main><footer id="fixed-footer">Fixed footer</footer>
        <script>window.captureProbeReady=true;setTimeout(()=>scrollTo(0,720),100);setInterval(()=>{const n=document.querySelector('#dynamic-marker');if(n)n.textContent='Dynamic text: '+Date.now()},500);</script>`);
      return;
    }
    if (url.pathname === '/minimum-font-probe') {
      response.setHeader('Content-Type', 'text/html; charset=utf-8');
      response.end(`<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Minimum font probe</title>
        <style>
          *{box-sizing:border-box}body{margin:0;padding:24px;font:16px Arial,sans-serif;color:#15202b;background:#fff}
          .sample{margin:12px 0;padding:12px;border:1px solid #678}
          #px8{font-size:8px}#px12{font-size:12px}#px16{font-size:16px}#px24{font-size:24px}
          #relative{font-size:.5em}a,button,input{font-size:10px}iframe{width:100%;height:100px;border:1px solid #333}
          @media(max-width:500px){body{background:#e9f4ff}}
        </style>
        <h1>Minimum font probe</h1><div id="px8" class="sample">8px text sample</div><div id="px12" class="sample">12px text sample</div>
        <div id="px16" class="sample">16px text sample</div><div id="px24" class="sample">24px text sample</div>
        <div id="relative" class="sample">Relative .5em text sample</div>
        <div class="sample"><a href="#">10px link sample</a> <button>10px button sample</button> <input value="10px input sample" aria-label="10px input"></div>
        <iframe title="10px iframe sample" srcdoc="&lt;style&gt;body{font:10px Arial}&lt;/style&gt;&lt;p&gt;10px text inside iframe&lt;/p&gt;"></iframe>
        <script>window.minimumFontProbeReady=true;</script>`);
      return;
    }
    if (url.pathname === '/dark-mode-image.svg') {
      response.setHeader('Content-Type', 'image/svg+xml; charset=utf-8');
      response.end('<svg xmlns="http://www.w3.org/2000/svg" width="240" height="120"><rect width="120" height="120" fill="#ffd400"/><rect x="120" width="120" height="120" fill="#e40046"/><circle cx="120" cy="60" r="34" fill="#00a6a6"/></svg>');
      return;
    }
    if (url.pathname === '/autofill-probe') {
      const months = Array.from({ length: 12 }, (_, i) => `<option value="${String(i + 1).padStart(2, '0')}">${i + 1}</option>`).join('');
      const years = [2029, 2030, 2031, 2032].map(y => `<option value="${y}">${y}</option>`).join('');
      response.setHeader('Content-Type', 'text/html; charset=utf-8');
      response.end(`<!doctype html><html><head><meta charset="utf-8"><title>Autofill probe</title></head><body>
        <form id="login" onsubmit="return false"><input id="user" name="username" type="text"><input id="pass" name="password" type="password"></form>
        <form id="pay" onsubmit="return false">
          <input id="ccname" autocomplete="cc-name"><input id="ccnum" autocomplete="cc-number">
          <select id="ccmonth" autocomplete="cc-exp-month">${months}</select>
          <select id="ccyear" autocomplete="cc-exp-year">${years}</select>
        </form>
        <form id="contact" onsubmit="return false"><input id="cname" autocomplete="name"><input id="cmail" autocomplete="email"><input id="cphone" autocomplete="tel"><input id="ccity" autocomplete="address-level2"></form>
        <input id="search" name="q"></body></html>`);
      return;
    }
    if (url.pathname === '/dark-mode-probe') {
      const kind = ['supported', 'light-only', 'already-dark'].includes(url.searchParams.get('kind'))
        ? url.searchParams.get('kind') : 'supported';
      const transformed = url.searchParams.get('transform') === '1';
      response.setHeader('Content-Type', 'text/html; charset=utf-8');
      response.end(`<!doctype html><html class="${transformed ? 'probe-transform' : ''}" data-kind="${kind}"><head>
        <meta charset="utf-8">${kind === 'supported' ? '<meta name="color-scheme" content="light dark">' : ''}<title>Dark mode probe: ${kind}</title>
        <style>
          *{box-sizing:border-box}html,body{margin:0;min-height:100%;font:18px system-ui,sans-serif}
          body{background:#fff;color:#171717;transition:background-color .15s,color .15s}
          header{position:sticky;top:0;padding:16px;background:#d9e8ff;color:#10243a;border-bottom:3px solid #3478c8}
          main{max-width:760px;margin:0 auto;padding:24px}.card{padding:18px;margin:16px 0;background:#f2f2f2;color:#222;border:2px solid #555;border-radius:8px}
          .swatches{display:flex;gap:8px}.swatches i{width:72px;height:48px;border:1px solid #222;background:#ffe11a}.swatches i:nth-child(2){background:#d51b65}.swatches i:nth-child(3){background:#087f8c}
          img,video{display:block;width:240px;height:120px;object-fit:contain;background:#fafafa;border:1px solid #333}
          footer{height:100px;padding:20px;background:#ddd;color:#111}
          html[data-kind="supported"]{color-scheme:light dark}
          @media(prefers-color-scheme:dark){html[data-kind="supported"] body{background:#171b22;color:#f2f4f8}html[data-kind="supported"] header{background:#22344c;color:#f5f8fc}html[data-kind="supported"] .card{background:#292f38;color:#f2f4f8;border-color:#aab5c5}}
          html[data-kind="already-dark"] body{background:#111820;color:#f2f4f8}html[data-kind="already-dark"] header{background:#1d2936;color:#f5f8fc}html[data-kind="already-dark"] .card{background:#252e38;color:#f2f4f8;border-color:#8995a5}
          html.probe-transform{filter:invert(1) hue-rotate(180deg)}html.probe-transform img,html.probe-transform video{filter:invert(1) hue-rotate(180deg)}
        </style></head><body><header><strong>Forced dark mode probe · ${kind}</strong></header>
        <main><h1>Page color test</h1><p id="scheme-state">Waiting for color scheme</p>
        <section class="card"><h2>Authored surface</h2><p>Background and text authored by the page.</p><div class="swatches"><i></i><i></i><i></i></div></section>
        <section class="card"><h2>Image</h2><img id="probe-image" src="/dark-mode-image.svg" alt="Three color swatches"></section>
        <section class="card"><h2>Live video</h2><canvas id="video-source" width="240" height="120" hidden></canvas><video id="probe-video" muted autoplay playsinline></video></section>
        <section class="card"><h2>Dynamic content</h2><p id="dynamic-content">Initial dynamic content</p><input aria-label="Probe input" value="Input contrast sample"></section>
        <footer>Bottom of the test page</footer></main>
        <script>
          const media=matchMedia('(prefers-color-scheme: dark)');
          function report(){document.querySelector('#scheme-state').textContent='prefers-color-scheme: '+(media.matches?'dark':'light');}
          report();media.addEventListener('change',report);
          let tick=0;setInterval(()=>{tick++;document.querySelector('#dynamic-content').textContent='Dynamic update '+tick+' · '+Date.now()},700);
          const canvas=document.querySelector('#video-source'),ctx=canvas.getContext('2d'),video=document.querySelector('#probe-video');
          ctx.fillStyle='#ffcc00';ctx.fillRect(0,0,120,120);ctx.fillStyle='#e40046';ctx.fillRect(120,0,120,120);ctx.fillStyle='#00a6a6';ctx.fillRect(0,60,240,60);
          video.srcObject=canvas.captureStream(5);video.play().catch(()=>{});setInterval(()=>{ctx.fillStyle='#00a6a6';ctx.fillRect((Date.now()/40)%240,0,5,120)},100);window.darkModeProbeReady=true;
        </script></body></html>`);
      return;
    }
    if (url.pathname === '/caret-browsing') {
      response.setHeader('Content-Type', 'text/html; charset=utf-8');
      response.end(`<!doctype html><meta charset="utf-8"><title>Caret browsing fixture</title>
        <style>body{font:20px sans-serif;margin:32px}p{max-width:700px;line-height:1.6}iframe{display:block;width:700px;height:180px;border:1px solid #888}input{font:inherit}</style>
        <h1>Caret browsing fixture</h1><p id="text">Keyboard navigation can move a text caret through this paragraph and select its words.</p>
        <p><a id="link" href="#link-target">A link that can be selected without opening it</a></p>
        <p><label>Editable field <input id="field" value="Editable input remains usable"></label></p>
        <iframe id="frame" title="Caret browsing frame" srcdoc="&lt;!doctype html&gt;&lt;meta charset='utf-8'&gt;&lt;p id='frame-text' style='font:20px sans-serif'&gt;Text inside the embedded frame is available for keyboard navigation.&lt;/p&gt;"></iframe>
        <p id="link-target">End of fixture.</p>`);
      return;
    }
    if (url.pathname.startsWith('/page/')) {
      loads.set(id, (loads.get(id) ?? 0) + 1);
      response.setHeader('Content-Type', 'text/html; charset=utf-8');
      response.end(`<!doctype html><title>Native fixture ${id}</title>${id.startsWith('new-tab')?`<link rel="icon" href="/new-tab-icon.png?${id==='new-tab-late'?'delay':id==='new-tab-bad'?'bad':'normal'}">`:''}
        <label>Fixture form <input id="draft"></label><div style="height:3000px"></div>
        <script>
          const id = ${JSON.stringify(id)};
          const pageLoad = ${loads.get(id)};
          let measuredGain=null,measuredAnalyser=null;
          if(id.startsWith('media')) {const create=AudioContext.prototype.createGain;AudioContext.prototype.createGain=function(){const gain=create.call(this);measuredGain=gain;measuredAnalyser=this.createAnalyser();measuredAnalyser.fftSize=1024;gain.connect(measuredAnalyser);return gain;};}
          if (id.startsWith('media')) { const audio = document.createElement('audio'); audio.id = 'fixture-audio'; audio.src = '/tone.wav'; audio.loop = true; audio.autoplay = true; document.body.append(audio); audio.play().catch(() => {}); }
          const field = document.getElementById('draft');
          field.value = 'artificial-test-draft';
          if (id === 'a') field.dispatchEvent(new Event('input', { bubbles: true }));
          scrollTo(0, 600);
          let busy = false; let permission = null;
          async function accountStorage(write = false) {
            const db = await new Promise((resolve, reject) => {
              const request = indexedDB.open('fixture-account', 1);
              request.onupgradeneeded = () => request.result.createObjectStore('records');
              request.onerror = () => reject(request.error);
              request.onsuccess = () => resolve(request.result);
            });
            try {
              if (write) {
                localStorage.setItem('fixture-account', 'alpha');
                await fetch('/account-cookie');
                await new Promise((resolve, reject) => {
                  const tx = db.transaction('records', 'readwrite');
                  tx.objectStore('records').put('alpha', 'value');
                  tx.oncomplete = resolve; tx.onerror = () => reject(tx.error);
                });
              }
              const indexed = await new Promise((resolve, reject) => {
                const request = db.transaction('records').objectStore('records').get('value');
                request.onsuccess = () => resolve(request.result ?? null);
                request.onerror = () => reject(request.error);
              });
              return { local: localStorage.getItem('fixture-account'), cookie: document.cookie, indexed };
            } finally { db.close(); }
          }
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
              const account = id.startsWith('account-') ? await accountStorage(command === 'store-account-alpha') : undefined;
              if (command === 'reset-widths') { widthTracking++; widths = [innerWidth]; }
              if (command === 'native-player-volume') {const el=document.getElementById('fixture-audio');Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype,'volume').set.call(el,0.8);}
              if (command === 'restore-player-volume') document.getElementById('fixture-audio').volume=1;
              if (command === 'native-controls') {const el=document.getElementById('fixture-audio');el.controls=true;el.src='/tone.wav';await el.play();}
              if (command === 'oscillator') {document.querySelectorAll('audio,video').forEach(el=>el.pause());const c=new AudioContext();const o=c.createOscillator();const g=c.createGain();g.gain.value=0.001;o.connect(g).connect(c.destination);o.start();await c.resume();}
              if (command === 'videos') {for(let n=0;n<2;n++)await makeVideo(n);}
              if (command === 'clip') {const video=document.createElement('video');video.title='Artificial clip';video.loop=true;video.src='/clip.webm';document.body.prepend(video);await video.play();}
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
              await fetch('/report/' + id, { method: 'POST', body: JSON.stringify({ pageLoad, account, videoCount:document.querySelectorAll('video').length,pip:Boolean(document.pictureInPictureElement),videoTime:document.pictureInPictureElement?.currentTime,audioRms:rms,audioGain:measuredGain?.gain.value,nativeVolume:audio?Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype,'volume').get.call(audio):null,nativeMuted:audio?Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype,'muted').get.call(audio):null,playerVolume:audio?.volume,value: field.value, width: innerWidth, widths, widthTracking, pixelRatio: devicePixelRatio, focused: document.hasFocus(), scroll: scrollY, scrollAnchor: document.querySelector('div[style*="3000px"]').getBoundingClientRect().top, pageClicks, url: location.pathname + location.search, dirty: id === 'a', permission, visibility: document.visibilityState, hiddenCount, reportedAt: Date.now() }) });
            } finally { busy = false; }
          }
          setInterval(tick, 200); tick();
        </script>${['media-pip-continuity','media-pip-fps'].includes(id)?'<script src="/pip-video.js"></script>':''}`);
      return;
    }
    response.writeHead(404).end();
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  return { origin: `http://127.0.0.1:${server.address().port}`, loads, reports, commands, download, payload,icons, gpcRequests, shieldRequests, trackingRequests, feed,
    close: () => new Promise(resolve => { server.closeAllConnections(); server.close(resolve); }) };
}
