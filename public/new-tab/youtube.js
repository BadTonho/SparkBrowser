// YouTube "now playing" card, its settings and the Spotify | YouTube tabs of the media category.
(() => {
  const $ = id => document.getElementById(id);
  const EMBED = 'https://www.youtube-nocookie.com';
  const card = window.__sparkMediaCard;
  const OPTIONS = { spark_youtube_thumbnail: ['small', 'cover'], spark_youtube_preview: ['image', 'video'] };
  let labels = {}, current = null, privateWindow = false, sample = null, timer = 0, shownTitle = null;
  let frameVideo = null, embedTime = null, framePlaying = null, lastSeek = 0;
  const text = key => labels[`newtab.widgets.${key}`] ?? key;
  const send = operation => window.dispatchEvent(new CustomEvent('spark-new-tab-action', { detail: { action: 'widgets', operation, data: {} } }));
  const stored = key => card.choice(key, OPTIONS[key]);
  const tabs = ['spotify', 'youtube'];
  function selectTab(name) {
    for (const tab of tabs) {
      const selected = tab === name;
      $(`media-tab-${tab}`).setAttribute('aria-selected', String(selected));
      $(`media-tab-${tab}`).tabIndex = selected ? 0 : -1;
      $(`${tab}-panel`).hidden = !selected;
    }
  }
  for (const tab of tabs) {
    $(`media-tab-${tab}`).addEventListener('click', () => selectTab(tab));
    $(`media-tab-${tab}`).addEventListener('keydown', event => {
      if (event.key !== 'ArrowUp' && event.key !== 'ArrowDown') return;
      const next = tabs[(tabs.indexOf(tab) + (event.key === 'ArrowDown' ? 1 : tabs.length - 1)) % tabs.length];
      selectTab(next); $(`media-tab-${next}`).focus();
    });
  }
  $('youtube-enabled').addEventListener('change', event => window.dispatchEvent(new CustomEvent('spark-youtube-setting', { detail: { youtube_enabled: event.target.checked } })));
  $('youtube-sidebar').addEventListener('change', event => window.dispatchEvent(new CustomEvent('spark-youtube-setting', { detail: { youtube_sidebar: event.target.checked } })));
  for (const [id, key] of [['youtube-thumb', 'spark_youtube_thumbnail'], ['youtube-preview', 'spark_youtube_preview']]) {
    $(id).addEventListener('change', event => {
      card.save(key, OPTIONS[key], event.target.value);
      if (current) render(current.config, current.video, current.saving);
    });
  }
  for (const [id, operation] of [['youtube-previous', 'youtube_previous'], ['youtube-toggle', 'youtube_toggle'], ['youtube-next', 'youtube_next'], ['youtube-open', 'youtube_open']]) {
    $(id).addEventListener('click', () => send(operation));
  }
  const position = () => sample ? Math.min(sample.duration, sample.position + (sample.playing ? (Date.now() - sample.at) / 1000 : 0)) : null;
  const fitTitle = () => card.fitTitle($('youtube-marquee'), $('youtube-title'));
  // The player is scaled up past its box (like object-fit: cover, then ZOOM further) so the title bar at
  // the top and the logo at the bottom that YouTube draws over every embed fall outside the visible area.
  const ZOOM = 1.4;
  function fitFrame() {
    const media = $('youtube-media'), frame = $('youtube-frame');
    const width = media.clientWidth, height = media.clientHeight;
    if (!frame.style || !width || !height) { frame.style?.removeProperty('width'); frame.style?.removeProperty('height'); return; }
    const wide = width / height > 16 / 9;
    frame.style.setProperty('width', `${Math.ceil((wide ? width : height * 16 / 9) * ZOOM)}px`);
    frame.style.setProperty('height', `${Math.ceil((wide ? width * 9 / 16 : height) * ZOOM)}px`);
  }
  // The player is only shown while it is really playing; loading, paused, ended and refused videos
  // (which YouTube covers with a big play button or an error message) show the thumbnail instead.
  function setReady(ready) { $('youtube-frame').setAttribute('data-ready', String(ready)); }
  function paintProgress() {
    if (!sample) return;
    const elapsed = position();
    $('youtube-progress').max = sample.duration; $('youtube-progress').value = elapsed;
    $('youtube-elapsed').textContent = card.clock(elapsed);
    $('youtube-total').textContent = card.clock(sample.duration);
  }
  function schedule() {
    const wanted = Boolean(sample?.playing);
    if (wanted && !timer && typeof window.setInterval === 'function') timer = window.setInterval(paintProgress, 1000);
    if (!wanted && timer) { window.clearInterval(timer); timer = 0; }
  }
  // The embedded player is muted and follows the tab: same video, same play state, same position.
  const command = (func, args = []) => $('youtube-frame').contentWindow?.postMessage(JSON.stringify({ event: 'command', func, args }), EMBED);
  function syncFrame(video, wanted) {
    const frame = $('youtube-frame');
    const usable = wanted && video && /^[A-Za-z0-9_-]{11}$/.test(video.video_id) && !document.hidden;
    if (!usable) {
      if (frameVideo) { frame.removeAttribute('src'); frameVideo = null; embedTime = null; framePlaying = null; }
      setReady(false);
      frame.hidden = true;
      return false;
    }
    if (frameVideo !== video.video_id) {
      frameVideo = video.video_id; embedTime = null; framePlaying = Boolean(video.playing);
      setReady(false);
      const start = Math.max(0, Math.floor(position() ?? 0));
      const origin = encodeURIComponent(window.location?.origin ?? '');
      frame.src = `${EMBED}/embed/${video.video_id}?autoplay=${video.playing ? 1 : 0}&mute=1&controls=0&disablekb=1&fs=0&iv_load_policy=3&modestbranding=1&playsinline=1&rel=0&enablejsapi=1&start=${start}&origin=${origin}`;
    } else if (framePlaying !== Boolean(video.playing)) {
      framePlaying = Boolean(video.playing);
      command(framePlaying ? 'playVideo' : 'pauseVideo');
    }
    frame.hidden = false;
    return true;
  }
  function correctDrift() {
    const expected = position();
    if (expected === null || embedTime === null || Math.abs(embedTime - expected) <= 4 || Date.now() - lastSeek < 4000) return;
    lastSeek = Date.now(); command('seekTo', [expected, true]);
  }
  $('youtube-frame').addEventListener('load', () => {
    $('youtube-frame').contentWindow?.postMessage(JSON.stringify({ event: 'listening', id: 1, channel: 'widget' }), EMBED);
  });
  window.addEventListener('message', event => {
    const frame = $('youtube-frame');
    if (event.origin !== EMBED || !frame.contentWindow || event.source !== frame.contentWindow) return;
    let data; try { data = JSON.parse(event.data); } catch { return; }
    if (data?.event !== 'infoDelivery' || !data.info || typeof data.info !== 'object') return;
    // playerState: -1 unstarted, 0 ended, 1 playing, 2 paused, 3 buffering, 5 cued (buffering keeps the last view).
    if (data.info.playerState === 1) setReady(true);
    else if ([-1, 0, 2, 5].includes(data.info.playerState)) setReady(false);
    if (Number.isFinite(data.info.currentTime)) { embedTime = data.info.currentTime; correctDrift(); }
  });
  function render(config, video = null, saving = false) {
    current = { config, video, saving };
    const enabled = Boolean(config.youtube_enabled) && !privateWindow;
    const thumb = stored('spark_youtube_thumbnail'), preview = stored('spark_youtube_preview');
    const widget = $('youtube-widget');
    widget.hidden = !enabled;
    widget.setAttribute('data-thumb', thumb); widget.setAttribute('data-preview', preview);
    $('youtube-enabled').checked = enabled;
    $('youtube-enabled').disabled = saving;
    $('youtube-sidebar').checked = config.youtube_sidebar !== false;
    $('youtube-sidebar').disabled = saving;
    $('youtube-thumb').value = thumb; $('youtube-preview').value = preview;
    $('youtube-options').hidden = !enabled;
    $('youtube-content').hidden = !video;
    $('youtube-controls').hidden = !video;
    $('youtube-status').textContent = video ? '' : text('youtubeNothingPlaying');
    const music = video?.source === 'music';
    $('youtube-heading').textContent = music ? 'YouTube Music' : 'YouTube';
    $('youtube-logo-video').hidden = music; $('youtube-logo-music').hidden = !music;
    const open = $('youtube-open');
    open.disabled = saving || !video;
    open.title = video ? text('youtubeOpen') : ''; open.setAttribute('aria-label', video ? `${music ? 'YouTube Music' : 'YouTube'}: ${text('youtubeOpen')}` : (music ? 'YouTube Music' : 'YouTube'));
    const title = video?.title || text('youtubeUnknownContent');
    if (title !== shownTitle) {
      shownTitle = title; $('youtube-title').textContent = title;
      $('youtube-marquee').setAttribute('title', title);
      card.later(fitTitle);
    }
    const duration = video?.duration;
    const known = Number.isFinite(duration) && duration > 0 && Number.isFinite(video?.position);
    sample = known ? { position: video.position, duration, playing: Boolean(video.playing), at: Date.now() } : null;
    $('youtube-progress').hidden = !known; $('youtube-times').hidden = !known;
    $('youtube-progress').setAttribute('aria-label', text('youtubeProgress'));
    if (known) paintProgress();
    schedule();
    const playing = syncFrame(video, preview === 'video' && enabled);
    $('youtube-frame').setAttribute('title', text('youtubePreviewVideo'));
    const cover = $('youtube-cover'), image = video?.image;
    const safe = typeof image === 'string' && /^data:image\/jpeg;base64,[A-Za-z0-9+/=]+$/.test(image);
    // The still image is also the placeholder while the player loads or if embedding is refused.
    cover.hidden = !safe;
    if (safe && cover.src !== image) cover.src = image;
    else if (!safe) cover.removeAttribute('src');
    widget.setAttribute('data-video', String(playing)); widget.setAttribute('data-media', String(safe || playing));
    card.later(fitFrame);
    const toggle = $('youtube-toggle');
    toggle.textContent = video?.playing ? '⏸' : '▶';
    const label = text(video?.playing ? 'youtubePause' : 'youtubePlay');
    toggle.title = label; toggle.setAttribute('aria-label', label);
    for (const id of ['youtube-previous', 'youtube-toggle', 'youtube-next']) $(id).disabled = saving;
  }
  window.__sparkYoutubeRender = render;
  window.addEventListener('spark-new-tab', event => {
    labels = event.detail.labels; privateWindow = Boolean(event.detail.private);
    shownTitle = null;
    if (current) render(current.config, current.video, current.saving);
  });
  window.addEventListener('resize', () => { if (shownTitle) { fitTitle(); fitFrame(); } });
  window.addEventListener('storage', event => { if (current && event.key in OPTIONS) render(current.config, current.video, current.saving); });
  // A hidden new tab must not keep decoding a second copy of the video.
  document.addEventListener?.('visibilitychange', () => { if (current) render(current.config, current.video, current.saving); });
})();
