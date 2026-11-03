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
    const { labels, preferences, searches, error, autofocus } = event.detail;
    document.documentElement.lang = preferences.language;
    document.title = labels['newtab.title'];
    document.querySelector('.content').setAttribute('aria-label', labels['newtab.search']);
    input.placeholder = labels['newtab.placeholder'];
    input.setAttribute('aria-label', labels['newtab.placeholder']);
    const submit = document.querySelector('#submit');
    submit.title = labels['newtab.search']; submit.setAttribute('aria-label', labels['newtab.search']);
    recent.setAttribute('aria-label', labels['newtab.recent']);
    document.querySelector('#empty').textContent = labels['newtab.empty'];
    document.querySelector('#empty').hidden = error || Boolean(searches?.length);
    document.querySelector('#error-text').textContent = labels['newtab.error'];
    document.querySelector('#error').hidden = !error;
    document.querySelector('#retry').textContent = labels['newtab.retry'];
    // Retain existing controls when content is unchanged: updates must preserve keyboard focus.
    const key = JSON.stringify(searches ?? []);
    if (recent.dataset.key !== key) {
      const oldFocus = recent.contains(document.activeElement) ? document.activeElement?.dataset.query : null;
      recent.replaceChildren(); recent.dataset.key = key;
      for (const item of searches ?? []) {
        const button = document.createElement('button');
        button.type = 'button'; button.className = 'recent-search'; button.dataset.query = item.query;
        const engine = item.engine === 'duckduckgo' ? 'DuckDuckGo' : 'Google';
        button.title = `${item.query} · ${engine}`;
        button.setAttribute('aria-label', button.title);
        const query = document.createElement('span'); query.className = 'query'; query.textContent = item.query;
        const provider = document.createElement('span'); provider.className = 'engine'; provider.textContent = engine;
        button.append(query, provider);
        button.addEventListener('click', () => send({ action:'navigate', input:item.query, engine:item.engine }));
        recent.append(button);
      }
      if (oldFocus) ([...recent.children].find(node => node.dataset.query === oldFocus) ?? input).focus();
    }
    if (!focused && autofocus) { focused = true; input.focus(); }
  });
})();
