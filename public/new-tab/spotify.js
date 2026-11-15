(() => {
  const $ = id => document.getElementById(id);
  let labels = {}, current = null, privateWindow = false, draft = false;
  const text = key => labels[`newtab.widgets.${key}`] ?? key;
  const send = operation => window.dispatchEvent(new CustomEvent('spark-new-tab-action', { detail: { action: 'widgets', operation, data: {} } }));
  const change = detail => window.dispatchEvent(new CustomEvent('spark-spotify-setting', { detail }));
  const navigate = input => window.dispatchEvent(new CustomEvent('spark-new-tab-action', { detail: { action: 'navigate', input } }));
  $('spotify-enabled').addEventListener('change', e => change({ spotify_enabled: e.target.checked }));
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
  function render(config, state = {}, saving = false) {
    current = { config, state, saving };
    const enabled = Boolean(config.spotify_enabled) && !privateWindow;
    $('menu-item-spotify').hidden = privateWindow;
    $('spotify-widget').hidden = !enabled;
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
    $('spotify-title').textContent = p?.title || text('spotifyUnknownContent');
    $('spotify-artist').textContent = p?.artist ?? '';
    $('spotify-device').textContent = p?.device ?? '';
    const cover = $('spotify-cover');
    const image = p?.image;
    const safe = typeof image === 'string' && /^data:image\/(jpeg|png|webp);base64,[A-Za-z0-9+/=]+$/.test(image);
    cover.hidden = !safe;
    if (safe && cover.src !== image) cover.src = image;
    else if (!safe) cover.removeAttribute('src');
    $('spotify-progress').hidden = !p?.duration;
    $('spotify-progress').max = p?.duration || 1;
    $('spotify-progress').value = Math.min(p?.progress || 0, p?.duration || 0);
    $('spotify-progress').setAttribute('aria-label', text('spotifyProgress'));
    const busy = saving || state.busy || state.connecting || Boolean(state.error);
    $('spotify-next').disabled = busy || !p?.can_next;
    $('spotify-previous').disabled = busy || !p?.can_previous;
    const toggle = $('spotify-toggle');
    toggle.disabled = busy || !(p?.playing ? p.can_pause : p?.can_play);
    toggle.textContent = p?.playing ? '⏸' : '▶';
    const title = text(p?.playing ? 'spotifyPause' : 'spotifyPlay');
    toggle.title = title; toggle.setAttribute('aria-label', title);
  }
  window.__sparkSpotifyRender = render;
  window.addEventListener('spark-new-tab', event => {
    labels = event.detail.labels; privateWindow = Boolean(event.detail.private);
    if (current) render(current.config, current.state, current.saving);
  });
})();
