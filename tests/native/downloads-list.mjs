import assert from 'node:assert/strict';
import { unlinkSync } from 'node:fs';
import { resolve, relative, isAbsolute, toNamespacedPath } from 'node:path';
import { useAutomaticDownloads } from './download-preferences.mjs';

export async function checkDownloadsList(c) {
  await useAutomaticDownloads(c);
  const { script, invoke, click, until, fixture } = c;
  const inside = action => c.withWebview('/index.html?surface=downloads', action);
  const shown = () => inside(() => until(() => script('return !!document.querySelector("[data-testid=downloads-dialog]")'), 'compact downloads surface'));
  const close = async () => { await click('[data-testid=close-downloads]'); await until(() => script('return !document.querySelector(".downloads-controller")'), 'downloads closed'); };
  const open = async () => { await click('[data-action=downloads]'); await shown(); };
  const expand = async () => { await click('[data-testid=download-add-toggle]'); await inside(() => until(() => script('return document.activeElement?.matches("[data-testid=download-url]")'), 'expanded link focus')); };
  const edit = value => inside(() => script('const e=document.querySelector("[data-testid=download-url]");e.value=arguments[0];e.dispatchEvent(new Event("input",{bubbles:true}));', value));
  const state = id => invoke('get_download_tasks').then(rows => rows.find(row => row.id === id));
  await invoke('create_tab', { url: fixture.origin + '/page/downloads-list' });
  await until(() => fixture.reports.has('downloads-list'), 'download page'); await c.focusWindow();
  const images = await script('const c=document.createElement("canvas");c.width=320;c.height=200;const ctx=c.getContext("2d");ctx.fillStyle="#4477bb";ctx.fillRect(0,0,320,200);ctx.fillStyle="#ffbb44";ctx.fillRect(60,30,120,120);return {png:c.toDataURL("image/png").split(",")[1],jpeg:c.toDataURL("image/jpeg").split(",")[1]}');
  const bmp = Buffer.alloc(58); bmp.write('BM'); bmp.writeUInt32LE(58,2); bmp.writeUInt32LE(54,10); bmp.writeUInt32LE(40,14); bmp.writeInt32LE(1,18); bmp.writeInt32LE(1,22); bmp.writeUInt16LE(1,26); bmp.writeUInt16LE(24,28); bmp[54]=180; bmp[55]=100; bmp[56]=40;
  const name = 'a-very-long-picture-name-that-must-stay-on-one-line-'.repeat(3) + '.png';
  const files = [
    [name, 'image/png', Buffer.from(images.png,'base64')], ['picture.jpg','image/jpeg',Buffer.from(images.jpeg,'base64')],
    ['picture.gif','image/gif',Buffer.from('R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7','base64')], ['picture.bmp','image/bmp',bmp],
    ['corrupt.png','image/png',Buffer.from('not an image')], ['document.pdf','application/pdf',Buffer.from('artificial PDF placeholder')],
  ];
  for(const [fileName,mime,bytes] of files) fixture.download.images.set(fileName,{mime,bytes});
  await open();
  assert.equal(await inside(() => script('return !!document.querySelector("[data-testid=download-url]")')),false);
  await expand(); await edit('https://fixture.example/draft');
  await click('[data-testid=download-add-toggle]'); await expand();
  assert.equal(await inside(() => script('return document.querySelector("[data-testid=download-url]").value')),'https://fixture.example/draft');
  await close(); await open();
  assert.equal(await inside(() => script('return !!document.querySelector("[data-testid=download-url]")')),false);
  await expand(); assert.equal(await inside(() => script('return document.querySelector("[data-testid=download-url]").value')),'https://fixture.example/draft');
  await edit('not a valid link'); await click('[data-testid=download-start]');
  await inside(() => until(() => script('return !!document.querySelector("[role=alert]") && !!document.querySelector("[data-testid=download-url]")'), 'invalid link keeps form visible'));
  await edit(fixture.origin + '/download/visual/' + name); await click('[data-testid=download-start]');
  const first = await until(async () => (await invoke('get_download_tasks')).find(row => row.url.includes('/download/visual/')), 'image download');
  await until(async () => (await state(first.id)).status === 'Completed', 'image completed');
  await inside(() => until(() => script('return !document.querySelector("[data-testid=download-url]") && document.activeElement?.matches("[data-testid=download-add-toggle]")'), 'successful link collapses form'));
  const tasks = [first];
  for(const [fileName] of files.slice(1)) {
    const task = await invoke('enqueue_download',{url:fixture.origin+'/download/visual/'+fileName});
    await until(async () => (await state(task.id)).status === 'Completed','visual fixture completed'); tasks.push(task);
  }
  for(const task of tasks.slice(0,4)) {
    const visual = await invoke('get_download_visual',{source:{kind:'task',id:task.id}});
    assert.ok(visual.thumbnail_png?.startsWith('data:image/png;base64,')); assert.ok(visual.size_bytes > 0);
  }
  await inside(() => until(() => script('return [...document.querySelectorAll(".download-file-preview img")].filter(e=>e.complete&&e.naturalWidth>0).length===4'), 'four real native thumbnails'));
  const layout = await inside(() => script('const e=[...document.querySelectorAll("[data-task-id]")].find(e=>e.dataset.taskId===arguments[0]),n=e.querySelector(".download-name"),p=e.querySelector(".download-file-preview").getBoundingClientRect();return {height:e.getBoundingClientRect().height,nowrap:getComputedStyle(n).whiteSpace,ellipsis:getComputedStyle(n).textOverflow,truncated:n.scrollWidth>n.clientWidth,preview:[p.width,p.height],name:n.textContent,text:e.textContent}', first.id));
  assert.ok(layout.height <= 66); assert.equal(layout.nowrap,'nowrap'); assert.equal(layout.ellipsis,'ellipsis'); assert.equal(layout.truncated,true); assert.deepEqual(layout.preview,[40,40]);
  assert.ok(!layout.text.includes('http://') && !layout.text.includes('SHA-256'));
  await inside(() => c.screenshot('downloads-compact-list'));
  for(const task of tasks.slice(4)) assert.equal((await invoke('get_download_visual',{source:{kind:'task',id:task.id}})).thumbnail_png,null);
  const firstPath = (await state(first.id)).save_path;
  const history = await invoke('get_downloads'); const old = history.find(row => row.save_path === firstPath);
  assert.ok((await invoke('get_download_visual',{source:{kind:'history',id:old.id}})).thumbnail_png);
  for(const source of [{kind:'task',id:'../outside.png'},{kind:'history',id:-1},{kind:'task',id:first.id,path:'C:/Windows/file.png'},{kind:'history',id:99999999}]) await assert.rejects(invoke('get_download_visual',{source}));
  await inside(() => assert.rejects(invoke('get_download_visual',{source:{kind:'task',id:first.id}}),/restricted|denied|not allowed/i));
  const saved = (await state(first.id)).save_path, root = resolve(c.directory,'data','downloads');
  const child = relative(toNamespacedPath(root),toNamespacedPath(resolve(saved))); assert.ok(child && !child.startsWith('..') && !isAbsolute(child));
  unlinkSync(saved); assert.equal((await invoke('get_download_visual',{source:{kind:'task',id:first.id}})).thumbnail_png,null);
  await close(); await open();
  await inside(() => until(() => script('return ![...document.querySelectorAll("[data-task-id]")].find(e=>e.dataset.taskId===arguments[0])?.querySelector(".download-file-preview img")',first.id),'removed file falls back after reopening'));
  // The existing blank-input behavior still downloads the active page.
  await expand(); await edit(''); await click('[data-testid=download-start]');
  await until(async () => (await invoke('get_download_tasks')).some(row => row.url === fixture.origin + '/page/downloads-list'),'empty input uses active page');
  await close();
}
