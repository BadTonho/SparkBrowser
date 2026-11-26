import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import { resolve, relative, isAbsolute, toNamespacedPath as namespaced } from 'node:path';
import { useAutomaticDownloads } from './download-preferences.mjs';

const movie = Buffer.from('artificial-movie-bytes'.repeat(4000));
const segments = [Buffer.from('first-segment'.repeat(3000)), Buffer.from('second-segment'.repeat(3000))];

/** A 32 × 18 gradient as an uncompressed bitmap. */
function poster() {
  const width = 32, height = 18, size = width * height * 3;
  const bytes = Buffer.alloc(54 + size);
  bytes.write('BM'); bytes.writeUInt32LE(54 + size, 2); bytes.writeUInt32LE(54, 10); bytes.writeUInt32LE(40, 14);
  bytes.writeInt32LE(width, 18); bytes.writeInt32LE(height, 22); bytes.writeUInt16LE(1, 26); bytes.writeUInt16LE(24, 28);
  for (let i = 0; i < size; i++) bytes[54 + i] = (i * 7) % 256;
  return bytes;
}

/** A page with one direct video and a script player that requests an HLS master playlist. */
async function site(clip) {
  const requests = [];
  const media = '#EXTM3U\n#EXTINF:1,\nseg0.ts\n#EXTINF:1,\nseg1.ts\n#EXT-X-ENDLIST\n';
  const routes = {
    '/watch': ['text/html', '<!doctype html><meta charset="utf-8"><title>Artificial lesson</title><video src="/clips/real.webm" muted autoplay loop style="width:320px;height:180px"></video><artificial-player></artificial-player><script>document.querySelector("artificial-player").attachShadow({mode:"open"}).innerHTML=\'<video src="/movie.mp4" poster="/poster.bmp" style="width:320px;height:180px"></video>\';performance.setResourceTimingBufferSize(1);fetch("/first").then(()=>fetch("/hls/master.m3u8")).then(()=>fetch("/locked.m3u8"))</script>'],
    '/first': ['text/plain', 'fills the resource list'],
    '/movie.mp4': ['video/mp4', movie],
    '/poster.bmp': ['image/bmp', poster()],
    // A real video without a poster: its preview can only be a frame drawn from the player.
    '/clips/real.webm': ['video/webm', clip],
    '/hls/master.m3u8': ['application/vnd.apple.mpegurl', '#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=900000,RESOLUTION=1280x720\nhigh.m3u8\n'],
    '/hls/high.m3u8': ['application/vnd.apple.mpegurl', media],
    '/hls/seg0.ts': ['video/mp2t', segments[0]],
    '/hls/seg1.ts': ['video/mp2t', segments[1]],
    '/locked.m3u8': ['application/vnd.apple.mpegurl', '#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI="key"\n#EXTINF:1,\nseg.ts\n#EXT-X-ENDLIST\n'],
  };
  const server = createServer((request, response) => {
    const path = new URL(request.url, 'http://localhost').pathname;
    requests.push({ path, referer: request.headers.referer ?? null });
    const route = routes[path];
    if (!route) { response.statusCode = 404; response.end(); return; }
    response.setHeader('Content-Type', route[0]);
    response.setHeader('Content-Length', Buffer.byteLength(route[1]));
    response.end(route[1]);
  });
  await new Promise(done => server.listen(0, '127.0.0.1', done));
  return { origin: `http://127.0.0.1:${server.address().port}`, requests, close: () => { server.closeAllConnections(); server.close(); } };
}

export async function checkVideoDownload({ fixture: shared, invoke, until, directory, script, click, focusWindow, screenshot, withWebview }) {
  await useAutomaticDownloads({ script, invoke });
  await focusWindow();
  const clip = Buffer.from(await (await fetch(shared.origin + '/clip.webm')).arrayBuffer());
  const fixture = await site(clip);
  try {
    await invoke('navigate_to', { input: fixture.origin + '/watch' });
    await until(() => fixture.requests.some(r => r.path === '/locked.m3u8') && fixture.requests.some(r => r.path === '/hls/master.m3u8'), 'page player requested its playlists');

    // The harness turns every toolbar button on.
    await until(() => script('return !!document.querySelector("[data-testid=open-video-download]")'), 'video download button');
    await focusWindow();

    const inside = action => withWebview('/index.html?surface=video-download', action);
    const order = await script('return [...document.querySelectorAll(".titlebar-actions [data-action]")].map(b => b.dataset.action)');
    assert.equal(order.at(-1), 'settings', JSON.stringify(order));
    const rows = 'return [...document.querySelectorAll("[data-testid=video-source]")].map(row=>({text:row.textContent,disabled:row.querySelector("button").disabled}))';
    await click('[data-testid=open-video-download]');
    await inside(() => until(() => script('return document.querySelectorAll("[data-testid=video-source]").length===3 && !!document.querySelector("[data-testid=video-unavailable]")'), 'native popover lists the videos'));
    assert.equal(await script('return document.querySelector(".browser-preview-fallback") === null'), true, 'the popover keeps the page visible');
    const button = await script('const r=document.querySelector("[data-testid=open-video-download]").getBoundingClientRect();return {right:r.right,bottom:r.bottom};');
    const box = await inside(() => script('const r=document.querySelector(".video-download-surface").getBoundingClientRect();return {right:r.right,top:r.top,width:r.width};'));
    assert.equal(box.width, 360);
    assert.ok(Math.abs(box.top - button.bottom - 8) < 2, `the popover opens below its button: ${JSON.stringify({ box, button })}`);
    assert.ok(Math.abs(box.right - button.right) < 2, `the popover lines up with its button: ${JSON.stringify({ box, button })}`);
    const listed = await inside(() => script(rows));
    assert.equal(await inside(() => script('return document.querySelectorAll("[data-testid=video-card] .video-download-thumb img[src^=\\"data:image/png\\"]").length')), 2, 'one preview from a poster and one drawn from the playing video');
    const direct = listed.findIndex(row => row.text.includes('movie.mp4'));
    const stream = listed.findIndex(row => row.text.includes('high.m3u8'));
    assert.ok(direct >= 0 && stream >= 0, JSON.stringify(listed));
    assert.ok(listed[stream].text.includes('720p'));
    assert.ok(listed.every(row => !row.disabled && !row.text.includes('locked.m3u8')), 'an option that cannot be saved is not listed');
    assert.equal(await inside(() => script('return document.querySelectorAll("[data-testid=video-unavailable]").length')), 1, 'its card says why');
    await inside(() => screenshot('video-download'));

    const save = index => inside(() => script('document.querySelectorAll("[data-testid=video-source]")[arguments[0]].querySelector("button").click()', index));
    await save(direct);
    await save(stream);
    const tasks = await until(async () => {
      const all = await invoke('get_download_tasks');
      return all.length === 2 && all.every(task => task.status === 'Completed') ? all : null;
    }, 'both videos downloaded');
    const read = task => {
      const child = relative(namespaced(resolve(directory, 'data', 'downloads')), namespaced(resolve(task.save_path)));
      assert.ok(child && !child.startsWith('..') && !isAbsolute(child));
      return readFileSync(task.save_path);
    };
    const file = tasks.find(task => task.source === 'url');
    const hls = tasks.find(task => task.source === 'hls');
    assert.equal(file.file_name, 'Artificial lesson.mp4');
    assert.equal(hls.file_name, 'Artificial lesson.ts');
    assert.deepEqual(read(file), movie);
    assert.deepEqual(read(hls), Buffer.concat(segments));
    assert.ok(fixture.requests.some(r => r.path === '/hls/seg1.ts' && r.referer === fixture.origin + '/watch'), 'segments carry the page as referer');

    await inside(() => script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true,cancelable:true}))'));
    await until(() => script('return !document.querySelector("[data-testid=video-download-dialog]")'), 'popover closes');
  } finally { fixture.close(); }
}
