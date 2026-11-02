// Artificial local media only. Recording prepares finite fixtures, not user media.
(() => {
  const video=document.createElement('video');video.id='pip-source';video.loop=true;video.width=320;document.body.prepend(video);
  const nativeCapture=video.captureStream.bind(video),captures=[];
  let rejectCapture=false;
  video.captureStream=(...args)=>{if(rejectCapture)throw new DOMException('Fixture capture failed','NotSupportedError');const stream=nativeCapture(...args);captures.push(stream);return stream;};
  const urls=new Map();
  async function prepare(fps) {
    const canvas=document.createElement('canvas');canvas.width=320;canvas.height=180;
    const context=canvas.getContext('2d');let frame=0;
    const draw=()=>{context.fillStyle=fps===30?'#235574':'#634371';context.fillRect(0,0,320,180);context.fillStyle='#fff';context.fillRect((frame++*5)%300,70,20,30);context.fillText(`${fps} FPS / ${frame}`,10,25);};
    draw();const stream=canvas.captureStream(fps),recorder=new MediaRecorder(stream,{mimeType:'video/webm;codecs=vp8'}),chunks=[];
    const interval=setInterval(draw,4);
    const complete=new Promise(resolve=>{recorder.ondataavailable=event=>{if(event.data.size)chunks.push(event.data);};recorder.onstop=resolve;});
    recorder.start();await new Promise(resolve=>setTimeout(resolve,2200));recorder.stop();await complete;
    clearInterval(interval);stream.getTracks().forEach(track=>track.stop());
    urls.set(fps,URL.createObjectURL(new Blob(chunks,{type:recorder.mimeType})));
  }
  async function select(fps) {video.src=urls.get(fps);await video.play();}
  const ready=(async()=>{await prepare(30);await prepare(60);await select(30);return true;})();
  window.pipFixture={ready,select,get captures(){return captures.length;},endCapture(){captures.at(-1)?.getVideoTracks().forEach(track=>track.stop());},failCapture(){rejectCapture=true;this.endCapture();}};
})();
