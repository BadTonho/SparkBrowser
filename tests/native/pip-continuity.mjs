import assert from 'node:assert/strict';
import {withFloatingPlayer} from './pip-player.mjs';

export async function openControlledPip(context,name='media-pip-continuity') {
  const {invoke,fixture,withWebview,script,until}=context;
  const id=(await invoke('create_tab',{url:fixture.origin+'/page/'+name})).tabs.find(t=>t.is_active).id;
  await withWebview('/page/'+name,()=>until(()=>script('return Boolean(window.pipFixture)&&document.querySelector("#pip-source").readyState>=2;'),'controlled source ready',20000));
  const report=(await invoke('refresh_tab_media',{tabId:id})).tabs.find(t=>t.tab_id===id);
  const frame=report.frames.find(f=>f.videos.some(v=>v.eligible)),video=frame.videos.find(v=>v.eligible);
  await invoke('enter_video_pip',{tabId:id,context:frame.context,document:frame.document,video:video.id});
  await withFloatingPlayer(context,()=>until(()=>script('return document.querySelector("video").readyState>=2;'),'first mirrored frame'));
  return id;
}

export async function checkPipContinuity(context) {
  const {invoke,script,until,wait,withWebview}=context;
  const id=await openControlledPip(context),path='/page/media-pip-continuity';
  const source=operation=>withWebview(path,operation);
  await source(()=>until(()=>script('return getComputedStyle(document.querySelector("#pip-source")).visibility==="hidden";'),'only mirrored image is visible'));
  const session=(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id).frames.find(f=>f.pip_session).pip_session;
  const captures=await source(()=>script('return window.pipFixture.captures;'));
  await withFloatingPlayer(context,()=>script('document.querySelector("[data-testid=pip-minimize]").click();'));
  await wait(7000);await context.floatingWindow('Restore');
  assert.equal(await source(()=>script('return window.pipFixture.captures;')),captures,'minimizing does not consume recapture attempts');
  // A replacement resource on the same element represents an ad/content switch.
  await source(()=>script('void window.pipFixture.select(60);'));
  await withFloatingPlayer(context,async()=>{
    await until(()=>script('return document.querySelector("video").srcObject.getVideoTracks()[0]?.getSettings().frameRate>50;'),'new source track reaches the existing PiP',12000);
    const before=await script('return document.querySelector("video").getVideoPlaybackQuality().totalVideoFrames');await wait(600);
    assert.ok(await script('return document.querySelector("video").getVideoPlaybackQuality().totalVideoFrames>arguments[0]',before));
    assert.equal(await script('return document.querySelector("video").srcObject.getAudioTracks().length'),0);
  });
  assert.equal((await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id).frames.find(f=>f.pip_session).pip_session,session,'source change preserves the window session');
  await source(()=>script('window.pipFixture.endCapture();'));
  await source(()=>until(()=>script('return window.pipFixture.captures>=2;'),'bounded recapture after ended track',7000));
  await withFloatingPlayer(context,async()=>{
    await until(()=>script('return document.querySelector("video").readyState>=2&&document.querySelector("video").srcObject.getVideoTracks()[0]?.readyState==="live";'),'recaptured stream is live');
    await script('document.querySelector("[data-testid=pip-speaker]").click();const s=document.querySelector("[data-testid=pip-volume]");s.value="43";s.dispatchEvent(new Event("input",{bubbles:true}));');
  });
  await source(()=>script('window.pipFixture.failCapture();'));
  await until(async()=>!(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id)?.frames.some(f=>f.pip_session),'failed capture closes its session',12000);
  const state=(await invoke('get_tab_media')).tabs.find(t=>t.tab_id===id);
  assert.equal(state.applied_volume,43,'failure confirms the latest volume');
  await source(()=>{
    return script('const v=document.querySelector("#pip-source");if(v.paused||getComputedStyle(v).visibility==="hidden")throw Error("original playback was not restored");');
  });
  await until(()=>script('return Boolean(document.querySelector(".media-notice"));'),'capture failure uses media notice');
  await invoke('navigate_to',{input:context.fixture.origin+'/page/pip-after-continuity'});
  await assert.rejects(()=>invoke('set_tab_volume',{tabId:id,volume:22,documentKey:state.document_key,pipSession:session,pipSequence:999}),/media.stale|media.unavailable/);
}
