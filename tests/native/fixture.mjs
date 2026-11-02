import { createServer } from 'node:http';

/** Controlled loopback pages report only artificial test values; no user browsing data is read. */
export async function createFixture() {
  const loads = new Map();
  const reports = new Map();
  const commands = new Map();
  const server = createServer(async (request, response) => {
    const url = new URL(request.url, 'http://127.0.0.1');
    const id = url.pathname.split('/')[2] ?? 'a';
    if (!/^[abpi]$/.test(id)) { response.writeHead(404).end(); return; }
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
  return { origin: `http://127.0.0.1:${server.address().port}`, loads, reports, commands,
    close: () => new Promise(resolve => { server.closeAllConnections(); server.close(resolve); }) };
}
