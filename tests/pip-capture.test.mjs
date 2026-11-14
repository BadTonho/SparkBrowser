import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';
import {test} from 'node:test';
import assert from 'node:assert/strict';

const code=readFileSync(new URL('../src-tauri/src/media/player/capture.js',import.meta.url),'utf8');
function fixture() {
  let now=0,next=0,failed=0,captures=0;
  const timers=new Map(),intervals=new Map(),frames=new Map();
  class Events {
    listeners=new Map();
    addEventListener(type,callback){if(!this.listeners.has(type))this.listeners.set(type,new Set());this.listeners.get(type).add(callback);}
    removeEventListener(type,callback){this.listeners.get(type)?.delete(callback);if(!this.listeners.get(type)?.size)this.listeners.delete(type);}
    fire(type,event={}){for(const listener of [...this.listeners.get(type)??[]])listener(event);}
  }
  class Track extends Events {
    constructor(kind='video'){super();this.kind=kind;this.readyState='live';this.muted=false;this.stops=0;}
    stop(){this.stops++;this.readyState='ended';}
    end(){this.readyState='ended';this.fire('ended');}
  }
  class Stream extends Events {
    constructor(tracks=[]){super();this.tracks=[...tracks];}
    getTracks(){return [...this.tracks];}
    getVideoTracks(){return this.tracks.filter(t=>t.kind==='video');}
    addTrack(track){this.tracks.push(track);this.fire('addtrack',{track});}
    removeTrack(track){this.tracks=this.tracks.filter(t=>t!==track);this.fire('removetrack',{track});}
  }
  const popup={MediaStream:Stream,performance:{now:()=>now},setTimeout(callback,delay){timers.set(++next,{callback,at:now+delay});return next;},clearTimeout:id=>timers.delete(id),setInterval(callback,delay){intervals.set(++next,{callback,delay,at:now+delay});return next;},clearInterval:id=>intervals.delete(id)};
  popup.document=Object.assign(new Events(),{hidden:false});
  popup.crypto={randomUUID:()=> 'session'};
  const document={adoptedStyleSheets:[],defaultView:{CSSStyleSheet:class{replaceSync(value){this.text=value;}}}};document.defaultView.document=document;
  const classes=new Set(['site-player']);
  const originalVideo=new Track(),audio=new Track('audio'),initial=new Stream([originalVideo,audio]);
  const source=Object.assign(new Events(),{ownerDocument:document,isConnected:true,readyState:4,currentTime:1,paused:false,ended:false,seeking:false,classList:{add:v=>classes.add(v),remove:v=>classes.delete(v)},captureStream(){captures++;return new Stream([new Track(),new Track('audio')]);}});
  const image={play:()=>Promise.resolve(),requestVideoFrameCallback(callback){frames.set(++next,callback);return next;},cancelVideoFrameCallback:id=>frames.delete(id)};
  const create=runInNewContext(`(${code})`,{});
  const stop=create(popup,source,image,initial,()=>failed++);
  function advance(ms,{moving=true}={}){
    const end=now+ms;
    while(true){
      const due=[...timers].map(([id,v])=>({id,...v,interval:false})).concat([...intervals].map(([id,v])=>({id,...v,interval:true}))).filter(v=>v.at<=end).sort((a,b)=>a.at-b.at)[0];
      if(!due)break;
      if(moving)source.currentTime+=(due.at-now)/1000;
      now=due.at;
      if(due.interval)intervals.get(due.id).at+=due.delay;else timers.delete(due.id);
      due.callback();
    }
    if(moving)source.currentTime+=(end-now)/1000;
    now=end;
  }
  const frame=()=>{const callbacks=[...frames.values()];frames.clear();callbacks.forEach(callback=>callback(now));};
  return {popup,source,image,initial,originalVideo,audio,document,classes,Track,Stream,frame,advance,stop,frames,timers,intervals,stats:()=>({failed,captures})};
}

test('source image is hidden only after a mirrored frame and restored without pausing or changing site styles',()=>{
  const f=fixture();assert.equal(f.classes.has('spark-pip-source-session'),false);
  assert.equal(f.audio.readyState,'ended');assert.equal(f.image.srcObject.getTracks().length,1);
  f.frame();assert.equal(f.classes.has('spark-pip-source-session'),true);assert.equal(f.document.adoptedStyleSheets.length,1);
  const siteSheet={site:true};f.document.adoptedStyleSheets.push(siteSheet);
  f.stop();f.stop();assert.deepEqual([...f.classes],['site-player']);assert.equal(f.document.adoptedStyleSheets.length,1);assert.equal(f.document.adoptedStyleSheets[0],siteSheet);
  assert.equal(f.originalVideo.stops,1);assert.equal(f.source.paused,false);assert.equal(f.image.srcObject,null);
  assert.equal(f.frames.size,0);assert.equal(f.timers.size,0);assert.equal(f.intervals.size,0);assert.equal(f.source.listeners.size,0);
});

test('new tracks replace ended tracks in the same silent mirror without opening a new capture',()=>{
  const f=fixture(),mirror=f.image.srcObject;f.frame();f.originalVideo.end();
  assert.equal(mirror.getVideoTracks().length,0);
  const replacement=new f.Track(),audio=new f.Track('audio');f.initial.addTrack(replacement);f.initial.addTrack(audio);
  assert.equal(f.image.srcObject,mirror);assert.equal(mirror.getVideoTracks()[0],replacement);assert.equal(mirror.getTracks().length,1);
  assert.equal(audio.readyState,'ended');assert.deepEqual(f.stats(),{captures:0,failed:0});f.stop();
  assert.equal(replacement.listeners.size,0);assert.equal(f.initial.listeners.size,0);
});

test('a stalled copy recaptures the same element; source bursts coalesce and recovery is bounded',()=>{
  const f=fixture();f.frame();
  f.advance(1600);assert.equal(f.stats().captures,1);
  for(let n=0;n<10;n++)f.source.fire('loadedmetadata');
  f.advance(300);assert.equal(f.stats().captures,1);
  f.advance(5000);assert.deepEqual(f.stats(),{captures:2,failed:1});
  assert.equal(f.image.srcObject,null);assert.equal(f.classes.has('spark-pip-source-session'),false);
  assert.equal(f.frames.size,0);assert.equal(f.intervals.size,0);
});

test('pause, buffering and seeking never consume recovery attempts',()=>{
  const f=fixture();f.frame();f.source.paused=true;f.advance(4000);
  f.source.paused=false;f.source.readyState=2;f.advance(4000);
  f.source.readyState=4;f.source.seeking=true;f.advance(4000);
  assert.deepEqual(f.stats(),{captures:0,failed:0});
  f.source.seeking=false;f.source.fire('seeking');f.advance(1000);assert.equal(f.stats().captures,0);f.stop();
});

test('a minimized floating document does not trigger recovery and restoring gives frames time to resume',()=>{
  const f=fixture();f.frame();f.popup.document.hidden=true;f.popup.document.fire('visibilitychange');f.advance(10000);
  assert.deepEqual(f.stats(),{captures:0,failed:0});
  f.popup.document.hidden=false;f.popup.document.fire('visibilitychange');f.advance(1000);f.frame();
  assert.deepEqual(f.stats(),{captures:0,failed:0});f.stop();assert.equal(f.popup.document.listeners.size,0);
});

test('late callbacks from a removed source or an old document cannot recapture or hide the new page',()=>{
  for(const invalidate of [f=>{f.source.isConnected=false;},f=>{f.document.defaultView.document={};}]) {
    const f=fixture();const callbacks=[...f.frames.values()];invalidate(f);callbacks.forEach(callback=>callback());f.advance(6000);
    assert.deepEqual(f.stats(),{captures:0,failed:0});assert.equal(f.classes.has('spark-pip-source-session'),false);assert.equal(f.image.srcObject,null);
  }
});

test('a recovered stream keeps running and resets its retry budget only after sustained frames',()=>{
  const f=fixture();f.frame();f.advance(1600);assert.equal(f.stats().captures,1);
  for(let i=0;i<80;i++){f.advance(40);f.frame();}
  assert.deepEqual(f.stats(),{captures:1,failed:0});
  f.advance(2000);assert.equal(f.stats().captures,2);assert.equal(f.stats().failed,0);f.stop();
});
