// WebKitGTK cannot display a MediaStream, so the source frames are copied straight into the popup.
// The source stays visible: WebKit pauses invisible muted-autoplay videos, which would freeze the copy.
(popup, source, image, initial, failed) => {
  const document = source.ownerDocument;
  const context = image.getContext('2d');
  let disposed = false, drawn = -1;
  function current() {
    try { return source.isConnected && source.ownerDocument === document && document.defaultView.document === document; }
    catch { return false; }
  }
  function stop() {
    if (disposed) return;
    disposed = true;
    popup.clearInterval(timer);
  }
  function draw() {
    if (disposed) return;
    if (!current()) { stop(); return; }
    const width = source.videoWidth, height = source.videoHeight;
    if (!width || !height || source.readyState < 2) return;
    const frames = source.getVideoPlaybackQuality?.().totalVideoFrames ?? -1;
    if (frames === drawn && frames !== -1 && image.width === width) return;
    drawn = frames;
    if (image.width !== width || image.height !== height) { image.width = width; image.height = height; }
    try { context.drawImage(source, 0, 0, width, height); }
    catch { stop(); if (current()) failed(); return; }
  }
  const timer = popup.setInterval(draw, 33);
  draw();
  return stop;
}
