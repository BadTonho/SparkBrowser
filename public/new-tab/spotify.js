(() => {
  const $ = id => document.getElementById(id);
  const card = window.__sparkMediaCard;
  const THUMBS = ['small', 'cover'], THUMB_KEY = 'spark_spotify_thumbnail';
  let labels = {}, current = null, privateWindow = false, draft = false, sample = null, timer = 0, shownTitle = null;
  const text = key => labels[`newtab.widgets.${key}`] ?? key;
  const send = operation => window.dispatchEvent(new CustomEvent('spark-new-tab-action', { detail: { action: 'widgets', operation, data: {} } }));
  const change = detail => window.dispatchEvent(new CustomEvent('spark-spotify-setting', { detail }));
  const navigate = input => window.dispatchEvent(new CustomEvent('spark-new-tab-action', { detail: { action: 'navigate', input } }));
  $('spotify-enabled').addEventListener('change', e => change({ spotify_enabled: e.target.checked }));
  $('spotify-thumb').addEventListener('change', event => {
    card.save(THUMB_KEY, THUMBS, event.target.value);
    if (current) render(current.config, current.state, current.saving);
  });
  $('spotify-client-id').addEventListener('input', () => { draft = true; });
  $('spotify-save-client').addEventListener('click', () => {
    const id = $('spotify-client-id').value.trim();
    if (id && !/^[a-f0-9]{32}$/i.test(id)) {
      $('spotify-config-status').textContent = text('spotifyInvalidClient'); return;
    }
    draft = false; change({ spotify_client_id: id });
  });
  for (const id of ['spotify-connect', 'spotify-widget-connect']) $(id).addEventListener('click', () => send('spotify_connect'));
  for (const [id, action] of [['spotify-cancel', 'spotify_cancel'], ['spotify-disconnect', 'spotify_disconnect'], ['spotify-open', 'spotify_open'], ['spotify-next', 'spotify_next'], ['spotify-previous', 'spotify_previous']]) {
    $(id).addEventListener('click', () => send(action));
  }
  $('spotify-dashboard').addEventListener('click', () => navigate('https://developer.spotify.com/dashboard'));
  $('spotify-toggle').addEventListener('click', () => send(current?.state?.data?.playing ? 'spotify_pause' : 'spotify_play'));
  const fitTitle = () => card.fitTitle($('spotify-marquee'), $('spotify-title'));
  // Playback is sampled every ten seconds; between samples the bar advances on its own while playing.
  function paintProgress() {
    if (!sample) return;
    const elapsed = Math.min(sample.duration, sample.progress + (sample.playing ? Date.now() - sample.at : 0));
    $('spotify-progress').value = elapsed;
    $('spotify-elapsed').textContent = card.clock(elapsed / 1000);
  }
  function schedule() {
    const wanted = Boolean(sample?.playing);
    if (wanted && !timer && typeof window.setInterval === 'function') timer = window.setInterval(paintProgress, 1000);
    if (!wanted && timer) { window.clearInterval(timer); timer = 0; }
  }
  function render(config, state = {}, saving = false) {
    current = { config, state, saving };
    const enabled = Boolean(config.spotify_enabled) && !privateWindow;
    const thumb = card.choice(THUMB_KEY, THUMBS);
    $('menu-item-spotify').hidden = privateWindow;
    $('spotify-widget').hidden = !enabled;
    $('spotify-widget').setAttribute('data-thumb', thumb);
    $('spotify-thumb').value = thumb;
    $('spotify-enabled').checked = enabled;
    $('spotify-options').hidden = !enabled;
    if (!draft) $('spotify-client-id').value = config.spotify_client_id ?? '';
    $('spotify-save-client').disabled = saving || state.connecting;
    const configured = Boolean(config.spotify_client_id);
    for (const id of ['spotify-connect', 'spotify-widget-connect']) {
      $(id).hidden = Boolean(state.connected) && state.error !== 'reconnect';
      $(id).disabled = Boolean(!configured || saving || state.connecting || state.busy);
    }
    $('spotify-cancel').hidden = !state.connecting;
    $('spotify-disconnect').hidden = !state.connected;
    const message = state.connecting ? text('spotifyConnecting')
      : state.error ? (labels[`newtab.widgets.error.spotify_${state.error}`] ?? labels[`newtab.widgets.error.${state.error}`] ?? text('error'))
      : !configured ? text('spotifyConfigure') : !state.connected ? text('spotifyConnectHint')
      : !state.data ? text('spotifyNoDevice') : '';
    $('spotify-status').textContent = message;
    $('spotify-config-status').textContent = message;
    const p = state.data;
    $('spotify-content').hidden = !p;
    $('spotify-controls').hidden = !p;
    const title = p?.title || text('spotifyUnknownContent');
    if (title !== shownTitle) {
      shownTitle = title; $('spotify-title').textContent = title;
      card.later(fitTitle);
    }
    $('spotify-marquee').setAttribute('title', [title, p?.artist, p?.device].filter(Boolean).join('\n'));
    $('spotify-artist').textContent = p?.artist ?? '';
    const open = $('spotify-open');
    open.title = text('spotifyOpen'); open.setAttribute('aria-label', text('spotifyOpen'));
    const cover = $('spotify-cover');
    const image = p?.image;
    const safe = typeof image === 'string' && /^data:image\/(jpeg|png|webp);base64,[A-Za-z0-9+/=]+$/.test(image);
    cover.hidden = !safe;
    $('spotify-widget').setAttribute('data-media', String(safe));
    if (safe && cover.src !== image) cover.src = image;
    else if (!safe) cover.removeAttribute('src');
    sample = p?.duration ? { progress: Math.min(p.progress || 0, p.duration), duration: p.duration, playing: Boolean(p.playing), at: Date.now() } : null;
    $('spotify-progress').hidden = !p?.duration; $('spotify-times').hidden = !p?.duration;
    $('spotify-progress').max = p?.duration || 1;
    $('spotify-progress').value = Math.min(p?.progress || 0, p?.duration || 0);
    $('spotify-progress').setAttribute('aria-label', text('spotifyProgress'));
    if (p?.duration) {
      $('spotify-elapsed').textContent = card.clock((p.progress || 0) / 1000);
      $('spotify-total').textContent = card.clock(p.duration / 1000);
    }
    schedule();
    const busy = saving || state.busy || state.connecting || Boolean(state.error);
    $('spotify-next').disabled = busy || !p?.can_next;
    $('spotify-previous').disabled = busy || !p?.can_previous;
    const toggle = $('spotify-toggle');
    toggle.disabled = busy || !(p?.playing ? p.can_pause : p?.can_play);
    toggle.textContent = p?.playing ? '⏸' : '▶';
    const label = text(p?.playing ? 'spotifyPause' : 'spotifyPlay');
    toggle.title = label; toggle.setAttribute('aria-label', label);
  }
  window.__sparkSpotifyRender = render;
  window.addEventListener('spark-new-tab', event => {
    labels = event.detail.labels; privateWindow = Boolean(event.detail.private);
    shownTitle = null;
    if (current) render(current.config, current.state, current.saving);
  });
  window.addEventListener('resize', () => { if (shownTitle) fitTitle(); });
  window.addEventListener('storage', event => { if (current && event.key === THUMB_KEY) render(current.config, current.state, current.saving); });
})();
