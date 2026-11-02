import { createServer } from 'node:http';

/** Controlled loopback pages report only artificial test values; no user browsing data is read. */
export async function createFixture() {
  const loads = new Map();
  const reports = new Map();
  const commands = new Map();
  const download = { version: 1, mode: 'normal', requests: [] };
  const payload = version => Buffer.from(Array.from({ length: 2 * 1024 * 1024 }, (_, i) => (i + version) % 251));
  const server = createServer(async (request, response) => {
    const url = new URL(request.url, 'http://127.0.0.1');
    if (url.pathname === '/download/file') {
      const body = payload(download.version); const etag = `"fixture-${download.version}"`;
      download.requests.push({ range: request.headers.range, ifRange: request.headers['if-range'] });
      let start = 0;
      if (request.headers.range && download.mode !== 'ignore-range' && (!request.headers['if-range'] || request.headers['if-range'] === etag)) start = Number(request.headers.range.match(/^bytes=(\d+)-$/)?.[1] ?? 0);
      if (download.mode === '416' || start >= body.length) { response.writeHead(416, { 'Content-Range': `bytes */${body.length}` }).end(); return; }
      const status = start ? 206 : 200;
      response.writeHead(status, { 'Content-Type': 'application/octet-stream', 'Content-Disposition': 'attachment; filename="fixture.bin"', 'Accept-Ranges': 'bytes', 'ETag': download.mode === 'weak' ? `W/${etag}` : etag, 'Content-Length': body.length - start, ...(start ? { 'Content-Range': `bytes ${download.mode === 'bad-range' ? start + 1 : start}-${body.length - 1}/${body.length}` } : {}) });
      let offset = start;
      const timer = setInterval(() => { const end = Math.min(offset + 32768, body.length); response.write(body.subarray(offset, end)); offset = end; if (offset === body.length) { clearInterval(timer); response.end(); } }, 30);
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
    if (url.pathname.startsWith('/page/')) {
      loads.set(id, (loads.get(id) ?? 0) + 1);
      response.setHeader('Content-Type', 'text/html; charset=utf-8');
      response.end(`<!doctype html><title>Native fixture ${id}</title>
        <label>Fixture form <input id="draft"></label><div style="height:3000px"></div>
        <script>
          const id = ${JSON.stringify(id)};
          const field = document.getElementById('draft');
          field.value = 'artificial-test-draft';
          if (id === 'a') field.dispatchEvent(new Event('input', { bubbles: true }));
          scrollTo(0, 600);
          let busy = false; let permission = null;
          let hiddenCount = 0;
          document.addEventListener('visibilitychange', () => {
            if (document.hidden) hiddenCount++;
          });
          async function mediaPermission() {
            try { const stream = await navigator.mediaDevices.getUserMedia({ audio: true }); stream.getTracks().forEach(track => track.stop()); permission = 'granted'; }
            catch { permission = 'denied'; }
          }
          if (id === 'i') mediaPermission();
          async function tick() {
            if (busy) return; busy = true;
            try {
              const command = await fetch('/control/' + id).then(response => response.json());
              if (command === 'spa') { history.pushState({}, '', '/page/' + id + '/spa'); document.title = 'Native SPA ' + id; }
              if (command === 'microphone') { permission = null; void mediaPermission(); }
              if (command === 'iframe') { const frame = document.createElement('iframe'); frame.allow = 'microphone'; frame.src = location.origin.replace('127.0.0.1', 'localhost') + '/page/i'; document.body.prepend(frame); }
              if (command === 'remove-iframe') document.querySelectorAll('iframe').forEach(frame => frame.remove());
              if (command === 'download') { const a = document.createElement('a'); a.href = '/download/file?native=1'; a.download = 'fixture.bin'; a.click(); }
              await fetch('/report/' + id, { method: 'POST', body: JSON.stringify({ value: field.value, scroll: scrollY, url: location.pathname + location.search, dirty: id === 'a', permission, visibility: document.visibilityState, hiddenCount, reportedAt: Date.now() }) });
            } finally { busy = false; }
          }
          setInterval(tick, 200); tick();
        </script>`);
      return;
    }
    response.writeHead(404).end();
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  return { origin: `http://127.0.0.1:${server.address().port}`, loads, reports, commands, download, payload,
    close: () => new Promise(resolve => { server.closeAllConnections(); server.close(resolve); }) };
}
