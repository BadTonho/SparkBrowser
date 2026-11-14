// Own the capture stream for the lifetime of one source element/document.
(popup, source, image, initial, failed) => {
  const document = source.ownerDocument;
  const mirror = new popup.MediaStream();
  const now = () => popup.performance.now();
  // The page-visible styling marker is unrelated to the native bridge token.
  const className = `spark-pip-source-${popup.crypto.randomUUID?.() ?? Math.random().toString(36).slice(2)}`;
  const sheet = new document.defaultView.CSSStyleSheet();
  sheet.replaceSync(`.${className} { visibility: hidden !important; }`);
  let disposed = false, hidden = false, capture, frameCallback, recoveryTimer, watch;
  let lastFrame = now(), lastAttempt = -Infinity, healthySince = 0, attempts = 0;
  let previousTime = source.currentTime, previousDecoded = source.getVideoPlaybackQuality?.().totalVideoFrames ?? 0, advancingUntil = 0;
  const listeners = [], captureListeners = [], trackListeners = new Map();
  function listen(target, type, callback, removers = listeners) {
    target.addEventListener(type, callback);
    removers.push(() => target.removeEventListener(type, callback));
  }
  function current() {
    try { return source.isConnected && source.ownerDocument === document && document.defaultView.document === document; }
    catch { return false; }
  }
  function revealSource() {
    if (!hidden) return;
    source.classList.remove(className);
    document.adoptedStyleSheets = document.adoptedStyleSheets.filter(value => value !== sheet);
    hidden = false;
  }
  function hideSource() {
    if (hidden || !current()) return;
    document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
    source.classList.add(className);
    hidden = true;
  }
  function releaseCapture() {
    captureListeners.splice(0).forEach(stop => stop());
    for (const removers of trackListeners.values()) removers.forEach(stop => stop());
    trackListeners.clear();
    capture?.getTracks().forEach(track => track.stop());
    capture = undefined;
  }
  function reconcile() {
    if (disposed) return;
    const tracks = capture.getTracks();
    for (const [track, removers] of trackListeners) if (!tracks.includes(track)) {
      removers.forEach(stop => stop()); trackListeners.delete(track);
    }
    for (const track of tracks) {
      if (track.kind !== 'video') { track.stop(); continue; }
      if (!trackListeners.has(track)) {
        const removers = []; trackListeners.set(track, removers);
        for (const event of ['ended', 'mute', 'unmute']) listen(track, event, reconcile, removers);
      }
    }
    const live = tracks.filter(track => track.kind === 'video' && track.readyState === 'live');
    const selected = live.find(track => !track.muted) ?? live[0];
    const previous = mirror.getVideoTracks()[0];
    if (previous !== selected) {
      mirror.getVideoTracks().forEach(track => mirror.removeTrack(track));
      if (selected) mirror.addTrack(selected);
      healthySince = 0;
      if (selected) void image.play().catch(() => {});
    }
  }
  function bind(stream) {
    releaseCapture(); capture = stream;
    listen(capture, 'addtrack', reconcile, captureListeners);
    listen(capture, 'removetrack', reconcile, captureListeners);
    reconcile();
  }
  function stop() {
    if (disposed) return;
    disposed = true;
    popup.clearTimeout(recoveryTimer); popup.clearInterval(watch);
    image.cancelVideoFrameCallback(frameCallback);
    listeners.splice(0).forEach(remove => remove());
    releaseCapture();
    mirror.getTracks().forEach(track => mirror.removeTrack(track));
    image.srcObject = null; revealSource();
  }
  function fail() { stop(); if (current()) failed(); }
  function recover() {
    recoveryTimer = undefined;
    if (disposed || !current()) return;
    if (popup.document.hidden || source.paused || source.ended || source.seeking || source.readyState < 3) return;
    if (now() - lastFrame < 1500 || now() - lastAttempt < 2000) return;
    if (source.mediaKeys || source.disablePictureInPicture || attempts >= 2) { fail(); return; }
    attempts++; lastAttempt = now(); healthySince = 0;
    try {
      const stream = source.captureStream();
      if (!stream.getVideoTracks().some(track => track.readyState === 'live')) {
        stream.getTracks().forEach(track => track.stop());
        return;
      }
      bind(stream);
    } catch { /* The watchdog retries only within this bounded recovery episode. */ }
  }
  function scheduleRecovery() {
    if (!disposed && recoveryTimer === undefined) recoveryTimer = popup.setTimeout(recover, 100);
  }
  function frame() {
    if (disposed || !current()) return;
    const time = now();
    if (time - lastFrame > 500) healthySince = 0;
    healthySince ||= time;
    if (time - healthySince >= 2000) attempts = 0;
    lastFrame = time; hideSource();
    frameCallback = image.requestVideoFrameCallback(frame);
  }
  image.srcObject = mirror;
  bind(initial);
  frameCallback = image.requestVideoFrameCallback(frame);
  // Source events can arrive in bursts when MediaSource/advertisements change.
  for (const event of ['loadedmetadata', 'loadeddata', 'playing', 'emptied']) listen(source, event, scheduleRecovery);
  listen(source, 'seeking', () => { lastFrame = now(); healthySince = 0; });
  listen(popup.document, 'visibilitychange', () => { lastFrame = now(); healthySince = 0; });
  watch = popup.setInterval(() => {
    if (!current()) { stop(); return; }
    const time = now(), position = source.currentTime, decoded = source.getVideoPlaybackQuality?.().totalVideoFrames ?? 0;
    if (position > previousTime + .01 || decoded > previousDecoded) advancingUntil = time + 750;
    previousTime = position; previousDecoded = decoded;
    if (!popup.document.hidden && time < advancingUntil && time - lastFrame >= 1500) scheduleRecovery();
  }, 500);
  return stop;
}
