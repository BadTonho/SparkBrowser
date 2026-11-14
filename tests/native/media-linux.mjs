import assert from 'node:assert/strict';
import { withFloatingPlayer } from './pip-player.mjs';

// WebKitGTK cannot capture media elements, so volume amplification goes through WebAudio and the
// floating player copies frames into a canvas. These checks cover that platform path.
export async function checkMediaLinux(context) {
  if (process.platform !== 'linux') return;
  const { invoke, until, fixture, script, wait, withWebview } = context;
  const volumeTab = (await invoke('create_tab', { url: fixture.origin + '/page/media-volume' })).tabs.find(t => t.is_active).id;
  await until(() => fixture.reports.has('media-volume'), 'audio fixture');
  await until(async () => (await invoke('get_tab_media')).tabs.find(t => t.tab_id === volumeTab)?.audible, 'audible tab');
  const media = async () => (await invoke('get_tab_media')).tabs.find(t => t.tab_id === volumeTab);
  let state = (await invoke('refresh_tab_media', { tabId: volumeTab })).tabs.find(t => t.tab_id === volumeTab);
  await wait(1500);
  const base = fixture.reports.get('media-volume').playerVolume;
  const set = async volume => {
    state = (await invoke('set_tab_volume', { tabId: volumeTab, volume, documentKey: state.document_key })).tabs.find(t => t.tab_id === volumeTab);
    assert.equal(state.applied_volume, volume, `applied ${volume}`);
    assert.equal(state.volume_status, 'ready', `status at ${volume}`);
  };
  await set(50);
  await wait(800);
  assert.ok(Math.abs(fixture.reports.get('media-volume').playerVolume - base) < 0.01, 'the page keeps seeing its own volume while attenuated');
  await set(200); await set(150); await set(0); await set(100);
  await invoke('set_tab_muted', { tabId: volumeTab, muted: true });
  await until(async () => (await media()).muted, 'tab muted');
  await invoke('set_tab_muted', { tabId: volumeTab, muted: false });
  await until(async () => !(await media()).muted, 'tab unmuted');

  const pipTab = (await invoke('create_tab', { url: fixture.origin + '/page/media-pip' })).tabs.find(t => t.is_active).id;
  await until(() => fixture.reports.has('media-pip'), 'PiP document');
  fixture.commands.set('media-pip', 'clip');
  const read = async () => (await invoke('get_tab_media')).tabs.find(t => t.tab_id === pipTab);
  const ready = await until(async () => { const s = await read(); return s.frames.flatMap(f => f.videos).some(v => v.eligible) ? s : null; }, 'eligible video');
  const frame = ready.frames.find(f => f.videos.some(v => v.eligible)), video = frame.videos.find(v => v.eligible);
  await invoke('enter_video_pip', { tabId: pipTab, context: frame.context, document: frame.document, video: video.id });
  assert.ok((await read()).frames.some(f => f.pip_mode === 'custom'), 'custom floating player');
  const picture = () => withFloatingPlayer(context, () => script('const c=document.querySelector("canvas");if(!c)return null;const d=c.getContext("2d").getImageData(0,0,c.width,c.height).data;let h=0;for(let i=0;i<d.length;i+=37)h=(h*31+d[i])|0;return h'));
  await until(async () => (await picture()) !== null, 'floating canvas');
  const before = await picture(); await wait(1000);
  assert.notEqual(await picture(), before, 'floating image advances');
  await invoke('create_tab', { url: fixture.origin + '/page/pip-other' });
  await wait(1500); const hidden = await picture(); await wait(2000);
  assert.notEqual(await picture(), hidden, 'floating image keeps advancing while its tab is in the background');
  await invoke('switch_tab', { tabId: pipTab });
  await withFloatingPlayer(context, () => script('document.querySelector("[data-testid=pip-close]").click();'));
  await until(async () => !(await read()).frames.some(f => f.picture_in_picture), 'floating player closed');
  assert.equal(await withWebview('/page/media-pip', () => script('return !document.querySelector("video").paused')), true, 'closing keeps the source playing');
}
