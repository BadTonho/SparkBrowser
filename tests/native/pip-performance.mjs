import assert from 'node:assert/strict';
import {writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {withFloatingPlayer} from './pip-player.mjs';
import {openControlledPip} from './pip-continuity.mjs';

/** Collect presentation cadence in the floating document, not a background timer. */
export async function samplePipFrames(context,milliseconds=10000) {
  return withFloatingPlayer(context,()=>context.request('POST',`/session/${context.sessionId()}/execute/async`,{
    script:`const done=arguments[arguments.length-1],duration=arguments[0],image=document.querySelector('video'),source=window.opener.document.querySelector('video');
      const start=performance.now(),before=source.getVideoPlaybackQuality(),identity=source.currentSrc;
      let count=0,last=start,maxGap=0,callback,unstable=false;
      const onChange=event=>{if(source.loop&&source.currentTime<.25&&['waiting','seeking'].includes(event.type))return;unstable=true;};
      for(const event of ['emptied','loadedmetadata','waiting','seeking'])source.addEventListener(event,onChange);
      const frame=time=>{count++;maxGap=Math.max(maxGap,time-last);last=time;callback=image.requestVideoFrameCallback(frame);};
      callback=image.requestVideoFrameCallback(frame);
      setTimeout(()=>{
        image.cancelVideoFrameCallback(callback);for(const event of ['emptied','loadedmetadata','waiting','seeking'])source.removeEventListener(event,onChange);
        const end=performance.now(),after=source.getVideoPlaybackQuality(),seconds=(end-start)/1000;
        const track=image.srcObject?.getVideoTracks()[0];
        done({seconds,presentedFPS:count/seconds,maxGapMs:Math.max(maxGap,end-last),sourceFPS:(after.totalVideoFrames-before.totalVideoFrames)/seconds,sourceDrops:after.droppedVideoFrames-before.droppedVideoFrames,unstable:unstable||identity!==source.currentSrc||after.totalVideoFrames<before.totalVideoFrames,track:track?.readyState,captureFPS:track?.getSettings().frameRate});
      },duration);`,args:[milliseconds],
  }));
}

export async function checkPipPerformance(context) {
  const {invoke,script,withWebview,wait,directory}=context;
  const id=await openControlledPip(context,'media-pip-fps'),measurements=[];
  for(const fps of [30,60]) {
    await invoke('switch_tab',{tabId:id});
    await withWebview('/page/media-pip-fps',()=>script('void window.pipFixture.select(arguments[0]);',fps));
    await wait(2500);
    for(const mode of ['source-visible','source-image-hidden','source-tab-hidden']) {
      if(mode==='source-visible')await withWebview('/page/media-pip-fps',()=>script('document.querySelector("#pip-source").style.setProperty("visibility","visible","important");'));
      if(mode==='source-image-hidden')await withWebview('/page/media-pip-fps',()=>script('document.querySelector("#pip-source").style.removeProperty("visibility");'));
      let peer;
      if(mode==='source-tab-hidden')peer=(await invoke('create_tab',{url:context.fixture.origin+'/page/pip-fps-peer'})).tabs.find(t=>t.is_active).id;
      await wait(1000);
      const sample=await samplePipFrames(context);measurements.push({fps,mode,...sample});
      writeFileSync(resolve(directory,'artifacts','pip-fps.json'),JSON.stringify(measurements,null,2));
      console.log('PiP FPS',JSON.stringify({fps,mode,...sample}));
      assert.equal(sample.unstable,false,'controlled sample must not change source or buffer');
      assert.ok(sample.sourceFPS>=fps*.9,`fixture supplies ${fps} FPS: ${sample.sourceFPS}`);
      assert.ok(sample.presentedFPS>=fps*.9,`${mode}: PiP presents at least 90% of ${fps} FPS (${sample.presentedFPS})`);
      assert.ok(sample.maxGapMs<=250,`${mode}: presentation gap ${sample.maxGapMs} ms exceeds 250 ms`);
      if(peer){await invoke('switch_tab',{tabId:id});await invoke('close_tab',{tabId:peer});}
    }
  }
  await invoke('exit_video_pip');
}
