(() => {
  const input = document.querySelector('#search');
  const recent = document.querySelector('#recent');
  let focused = false;
  const send = detail => window.dispatchEvent(new CustomEvent('veloce-new-tab-action', { detail }));
  document.querySelector('#search-form').addEventListener('submit', event => {
    event.preventDefault();
    if (input.value.trim()) send({ action:'navigate', input:input.value.trim() });
  });
  document.querySelector('#retry').addEventListener('click', () => send({ action:'read' }));
  window.addEventListener('veloce-new-tab', event => {
    const { labels, preferences, sites, error, autofocus } = event.detail;
    document.documentElement.lang = preferences.language;
    document.title = labels['newtab.title'];
    document.querySelector('.content').setAttribute('aria-label', labels['newtab.search']);
    input.placeholder = labels['newtab.placeholder'];
    input.setAttribute('aria-label', labels['newtab.placeholder']);
    const submit = document.querySelector('#submit');
    submit.title = labels['newtab.search']; submit.setAttribute('aria-label', labels['newtab.search']);
    recent.setAttribute('aria-label', labels['newtab.recent']);
    document.querySelector('#empty').textContent = labels['newtab.empty'];
    document.querySelector('#empty').hidden = error || Boolean(sites?.length);
    document.querySelector('#error-text').textContent = labels['newtab.error'];
    document.querySelector('#error').hidden = !error;
    document.querySelector('#retry').textContent = labels['newtab.retry'];
    // Retain existing controls when content is unchanged: updates must preserve keyboard focus.
    const key = JSON.stringify(sites ?? []);
    if (recent.dataset.key !== key) {
      const oldFocus = recent.contains(document.activeElement) ? document.activeElement?.dataset.siteKey : null;
      recent.replaceChildren(); recent.dataset.key = key;
      for (const item of sites ?? []) {
        const button = document.createElement('button');
        button.type = 'button'; button.className = 'recent-site'; button.dataset.siteKey = item.site_key;
        button.title = `${item.domain} · ${item.title}`;
        button.setAttribute('aria-label', button.title);
        const icon = document.createElement('img'); icon.className = 'site-icon'; icon.alt = ''; icon.width = 24; icon.height = 24;
        const fallback = '/new-tab/site-fallback.svg';
        icon.src = typeof item.icon === 'string' && /^data:image\/png;base64,[A-Za-z0-9+/=]+$/.test(item.icon) ? item.icon : fallback;
        icon.addEventListener('error', () => { if (icon.getAttribute('src') !== fallback) icon.src = fallback; });
        const domain = document.createElement('span'); domain.className = 'site-domain'; domain.textContent = item.domain;
        const title = document.createElement('span'); title.className = 'site-title'; title.textContent = item.title;
        button.append(icon, domain, title);
        button.addEventListener('click', () => send({ action:'visit', input:item.url, site_key:item.site_key }));
        recent.append(button);
      }
      if (oldFocus) ([...recent.children].find(node => node.dataset.siteKey === oldFocus) ?? input).focus();
    }
    if (!focused && autofocus) { focused = true; input.focus(); }
  });
})();
