(() => {
  const $ = id => document.getElementById(id);
  const panel = $('settings-panel'), gear = $('widget-settings');
  let labels = {}, language = 'en', snapshot = null, desired = null, saving = false, saveRevision = -1;
  let cities = [], cityPending = false, cityQuery = '', weatherKey = '', gmailKey = '';
  const text = key => labels[`newtab.widgets.${key}`] ?? key;
  const send = (operation, data = {}) => window.dispatchEvent(new CustomEvent('veloce-new-tab-action', { detail: { action: 'widgets', operation, data } }));
  const settings = () => desired ?? snapshot?.settings;
  const errorText = code => labels[`newtab.widgets.error.${code}`] ?? text('error');
  const cityName = city => city ? [city.name, city.region, city.country].filter(Boolean).join(', ') : text('noCity');
  const temperature = value => `${Math.round(settings().fahrenheit ? value * 9 / 5 + 32 : value)}°${settings().fahrenheit ? 'F' : 'C'}`;
  const date = (value, options) => new Intl.DateTimeFormat(language, options).format(value);
  function condition(code) {
    if (code === 0) return 'clear';
    if (code <= 3) return 'cloudy';
    if (code <= 48) return 'fog';
    if (code >= 95) return 'storm';
    if ([71,73,75,77,85,86].includes(code)) return 'snow';
    return 'rain';
  }
  function icon(code) {
    const image = document.createElement('img'); image.className = 'weather-symbol';
    const name = condition(code); image.src = `/new-tab/weather/${name}.svg`; image.alt = text(name);
    return image;
  }
  function close() { if (panel.open) panel.close(); gear.setAttribute('aria-expanded', 'false'); gear.focus(); }
  gear.addEventListener('click', () => {
    if (panel.open) { close(); return; }
    send('panel'); panel.showModal(); gear.setAttribute('aria-expanded', 'true'); $('settings-close').focus();
  });
  $('settings-close').addEventListener('click', close);
  panel.addEventListener('cancel', event => { event.preventDefault(); close(); });
  panel.addEventListener('click', event => {
    if (event.target !== panel) return;
    const box = panel.getBoundingClientRect();
    if (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom) close();
  });
  panel.addEventListener('close', () => gear.setAttribute('aria-expanded', 'false'));
  function flush() {
    if (saving || !desired || !snapshot) return;
    saving = true; saveRevision = snapshot.revision;
    send('save', { revision: saveRevision, settings: { ...desired } });
  }
  function change(patch) {
    if (!snapshot) return;
    desired = { ...settings(), ...patch }; $('settings-status').textContent = '';
    render(); flush();
  }
  $('weather-enabled').addEventListener('change', event => change({ weather_enabled: event.target.checked }));
  $('gmail-enabled').addEventListener('change', event => change({ gmail_enabled: event.target.checked }));
  $('temperature-unit').addEventListener('change', event => change({ fahrenheit: event.target.value === 'fahrenheit' }));
  $('location-mode').addEventListener('change', event => change({ automatic_location: event.target.value === 'automatic' }));
  function searchCities() {
    const query = $('city-query').value.trim(); if (query.length < 2) return;
    cityPending = true; cityQuery = query; cities = []; renderCities();
    send('cities', { query });
  }
  $('city-search').addEventListener('click', searchCities);
  $('city-query').addEventListener('keydown', event => { if (event.key === 'Enter') { event.preventDefault(); searchCities(); } });
  $('city-query').addEventListener('input', () => { cities = []; cityPending = false; cityQuery = ''; renderCities(); });
  function renderCities() {
    $('city-results').replaceChildren();
    $('city-status').textContent = cityPending ? text('loading') : cityQuery && !cities.length ? text('noCities') : '';
    for (const city of cities) {
      const button = document.createElement('button'); button.type = 'button'; button.textContent = cityName(city);
      button.addEventListener('click', () => { change({ city, automatic_location: false }); cities = []; cityQuery = ''; renderCities(); $('city-query').focus(); });
      $('city-results').append(button);
    }
  }
  for (const id of ['location-request', 'weather-locate']) $(id).addEventListener('click', () => send('locate'));
  $('weather-configure').addEventListener('click', () => gear.click());
  $('weather-refresh').addEventListener('click', () => send('refresh_weather'));
  $('gmail-refresh').addEventListener('click', () => send('refresh_gmail'));
  for (const id of ['gmail-connect', 'gmail-widget-connect']) $(id).addEventListener('click', () => send('connect'));
  $('gmail-disconnect').addEventListener('click', () => send('disconnect'));
  $('gmail-cancel').addEventListener('click', () => send('cancel_connect'));
  $('gmail-open').addEventListener('click', () => send('open_gmail'));
  $('weather-attribution').addEventListener('click', () => window.dispatchEvent(new CustomEvent('veloce-new-tab-action', { detail: { action: 'navigate', input: 'https://open-meteo.com/' } })));

  function renderWeather() {
    const config = settings(), weather = snapshot.weather;
    $('weather-widget').hidden = !config.weather_enabled;
    $('weather-place').textContent = config.automatic_location ? text('currentLocation') : cityName(config.city);
    $('weather-locate').hidden = !config.automatic_location;
    $('weather-locate').disabled = snapshot.location_busy || saving;
    $('weather-configure').hidden = config.automatic_location || Boolean(config.city);
    $('weather-refresh').disabled = weather.busy || (!config.city && !config.automatic_location);
    const data = weather.data;
    $('weather-status').textContent = snapshot.location_busy ? text('locating') : weather.error ? errorText(weather.error) : weather.busy ? text('loading') : data ? `${text('updated')} ${date(new Date(data.updated * 1000), { hour: '2-digit', minute: '2-digit' })}` : config.automatic_location ? text('locationNeeded') : text('chooseCity');
    const key = JSON.stringify([data, config.fahrenheit, language]);
    if (key === weatherKey) return; weatherKey = key;
    $('weather-content').replaceChildren(); if (!data) return;
    const current = document.createElement('div'); current.className = 'weather-temperature';
    const value = document.createElement('span'); value.textContent = temperature(data.temperature); current.append(icon(data.code), value);
    const feels = document.createElement('p'); feels.textContent = `${text('feelsLike')} ${temperature(data.feels_like)}`;
    const days = document.createElement('div'); days.className = 'weather-days';
    for (const item of data.days) {
      const day = document.createElement('div'); day.className = 'weather-day';
      const name = document.createElement('span'); name.textContent = date(new Date(`${item.date}T12:00:00`), { weekday: 'short' });
      const range = document.createElement('span'); range.textContent = `${temperature(item.min)} / ${temperature(item.max)}`;
      day.append(name, icon(item.code), range); days.append(day);
    }
    $('weather-content').append(current, feels, days);
  }
  function renderGmail() {
    const mail = snapshot.gmail, data = mail.data;
    $('gmail-widget').hidden = !settings().gmail_enabled;
    $('gmail-account').textContent = mail.email ?? ''; $('gmail-account').title = mail.email ?? '';
    $('gmail-unread').textContent = data ? text('unread').replace('{count}', data.unread) : '';
    const status = mail.connecting ? text('connecting') : mail.error ? errorText(mail.error) : !mail.configured ? text('notConfigured') : !mail.email ? text('notConnected') : mail.busy ? text('loading') : data?.messages.length === 0 ? text('inboxEmpty') : data ? `${text('updated')} ${date(new Date(data.updated * 1000), {hour:'2-digit',minute:'2-digit'})}` : '';
    $('gmail-status').textContent = status; $('gmail-config-status').textContent = status;
    for (const id of ['gmail-connect', 'gmail-widget-connect']) { $(id).hidden = Boolean(mail.email) && mail.error !== 'reconnect'; $(id).disabled = !mail.configured || mail.connecting || saving; }
    $('gmail-cancel').hidden = !mail.connecting;
    $('gmail-disconnect').hidden = !mail.email;
    $('gmail-open').hidden = !mail.email;
    $('gmail-refresh').disabled = !mail.email || mail.busy || mail.connecting;
    const key = JSON.stringify([data, language]); if (key === gmailKey) return; gmailKey = key;
    $('gmail-messages').replaceChildren();
    for (const item of data?.messages ?? []) {
      const row = document.createElement('li'); row.className = `gmail-message${item.unread ? ' unread' : ''}`;
      const sender = document.createElement('span'); sender.className = 'mail-sender'; sender.textContent = item.sender || text('unknownSender'); sender.title = sender.textContent;
      const subject = document.createElement('span'); subject.textContent = item.subject || text('noSubject'); subject.title = subject.textContent;
      const time = document.createElement('time'); time.dateTime = new Date(item.date * 1000).toISOString(); time.textContent = date(new Date(item.date * 1000), { month: 'short', day: 'numeric', hour:'2-digit', minute:'2-digit' });
      row.setAttribute('aria-label', `${item.unread ? text('unreadMessage') + '. ' : ''}${sender.textContent}. ${subject.textContent}. ${time.textContent}`);
      row.append(sender, subject, time); $('gmail-messages').append(row);
    }
  }
  function render() {
    if (!snapshot) return;
    const config = settings();
    gear.disabled = false;
    $('widgets').hidden = !config.weather_enabled && !config.gmail_enabled;
    $('weather-enabled').checked = config.weather_enabled; $('gmail-enabled').checked = config.gmail_enabled;
    $('weather-options').hidden = !config.weather_enabled; $('gmail-options').hidden = !config.gmail_enabled;
    $('location-mode').value = config.automatic_location ? 'automatic' : 'manual';
    $('manual-location').hidden = config.automatic_location; $('location-request').hidden = !config.automatic_location;
    $('location-request').disabled = snapshot.location_busy || saving;
    $('temperature-unit').value = config.fahrenheit ? 'fahrenheit' : 'celsius';
    $('selected-city').textContent = cityName(config.city);
    renderWeather(); renderGmail();
  }
  window.addEventListener('veloce-new-tab', event => {
    labels = event.detail.labels; language = event.detail.preferences.language;
    for (const node of document.querySelectorAll('[data-label]')) {
      const value = text(node.dataset.label);
      if (node.classList.contains('icon-button')) { node.title = value; node.setAttribute('aria-label', value); }
      else node.textContent = value;
    }
    gear.title = text('settings'); gear.setAttribute('aria-label', text('settings'));
    $('city-query').placeholder = text('cityPlaceholder'); $('city-query').setAttribute('aria-label', text('cityPlaceholder'));
    $('weather-heading').textContent = text('weather'); render();
  });
  window.addEventListener('veloce-new-tab-widgets', event => {
    snapshot = event.detail;
    if (snapshot.action_error || snapshot.settings_error) {
      $('settings-status').textContent = errorText(snapshot.action_error ?? snapshot.settings_error);
      if (saving && (snapshot.operation === 'save' || snapshot.settings_error)) { saving = false; desired = null; }
      if (cityPending && snapshot.operation === 'cities') { cityPending = false; $('city-status').textContent = errorText(snapshot.action_error); }
    } else if (saving && snapshot.revision > saveRevision) {
      saving = false;
      if (JSON.stringify(desired) === JSON.stringify(snapshot.settings)) desired = null;
      flush();
    }
    if (snapshot.cities && snapshot.city_query?.trim() === $('city-query').value.trim()) {
      cities = snapshot.cities; cityQuery = snapshot.city_query; cityPending = false; renderCities();
    }
    render();
  });
})();
