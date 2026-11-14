(() => {
  const $ = id => document.getElementById(id);
  const FONTS = ['system', 'monospace', 'serif', 'rounded', 'display'];
  const POSITIONS = ['bottom', 'top', 'card'];

  let labels = {};
  let language = 'en';
  let timerId = null;

  const text = key => labels[`newtab.widgets.${key}`] ?? key;

  const getStorage = (key, fallback) => {
    try {
      const val = localStorage.getItem(key);
      return val !== null ? val : fallback;
    } catch {
      return fallback;
    }
  };

  const setStorage = (key, val) => {
    try {
      localStorage.setItem(key, val);
    } catch {}
  };

  function getConfig() {
    const rawEnabled = getStorage('spark_clock_enabled', 'true');
    const enabled = rawEnabled === 'true' || rawEnabled === true;
    const font = FONTS.includes(getStorage('spark_clock_font', 'system')) ? getStorage('spark_clock_font', 'system') : 'system';
    const format = getStorage('spark_clock_format', '24h') === '12h' ? '12h' : '24h';
    const rawPos = getStorage('spark_clock_position', 'bottom');
    const position = POSITIONS.includes(rawPos) ? rawPos : 'bottom';
    const seconds = getStorage('spark_clock_seconds', 'false') === 'true';
    const showDate = getStorage('spark_clock_date', 'true') === 'true';
    const showGreeting = getStorage('spark_clock_greeting', 'false') === 'true';

    return { enabled, font, format, position, seconds, showDate, showGreeting };
  }

  function getGreeting(hour) {
    if (hour < 5) return text('greetingNight');
    if (hour < 12) return text('greetingMorning');
    if (hour < 18) return text('greetingAfternoon');
    return text('greetingEvening');
  }

  function formatTime(now, format, showSeconds) {
    const hours = now.getHours();
    const minutes = String(now.getMinutes()).padStart(2, '0');
    const seconds = String(now.getSeconds()).padStart(2, '0');

    if (format === '12h') {
      const h12 = hours % 12 || 12;
      const ampm = hours >= 12 ? 'PM' : 'AM';
      const secPart = showSeconds ? `:${seconds}` : '';
      return `${h12}:${minutes}${secPart} ${ampm}`;
    }

    const h24 = String(hours).padStart(2, '0');
    const secPart = showSeconds ? `:${seconds}` : '';
    return `${h24}:${minutes}${secPart}`;
  }

  function formatDate(now, lang) {
    try {
      const formatter = new Intl.DateTimeFormat(lang, {
        weekday: 'long',
        day: 'numeric',
        month: 'long'
      });
      return formatter.format(now);
    } catch {
      return now.toLocaleDateString(lang);
    }
  }

  function applyFont(font) {
    const widgets = document.querySelectorAll('.clock-widget');
    for (const el of widgets) {
      for (const f of FONTS) {
        el.classList.remove(`clock-font-${f}`);
      }
      el.classList.add(`clock-font-${font}`);
    }
  }

  function updateClockDisplay() {
    const config = getConfig();
    const now = new Date();

    const timeStr = formatTime(now, config.format, config.seconds);
    const dateStr = config.showDate ? formatDate(now, language) : '';
    const greetingStr = config.showGreeting ? getGreeting(now.getHours()) : '';

    const bottomWidget = $('clock-bottom-widget');
    const topWidget = $('clock-top-widget');
    const cardWidget = $('clock-card-widget');
    const widgetsGrid = $('widgets');

    applyFont(config.font);

    // Hide all first
    if (bottomWidget) bottomWidget.hidden = true;
    if (topWidget) topWidget.hidden = true;
    if (cardWidget) cardWidget.hidden = true;

    if (!config.enabled) {
      return;
    }

    let activePrefix = null;
    let activeWidget = null;

    if (config.position === 'top') {
      activePrefix = 'clock-top';
      activeWidget = topWidget;
    } else if (config.position === 'card') {
      activePrefix = 'clock-card';
      activeWidget = cardWidget;
      if (widgetsGrid && widgetsGrid.hidden) {
        widgetsGrid.hidden = false;
      }
    } else {
      activePrefix = 'clock-bottom';
      activeWidget = bottomWidget;
    }

    if (activeWidget) {
      activeWidget.hidden = false;
      const timeEl = $(`${activePrefix}-time`);
      const dateEl = $(`${activePrefix}-date`);
      const greetingEl = $(`${activePrefix}-greeting`);

      if (timeEl) timeEl.textContent = timeStr;
      if (dateEl) {
        dateEl.textContent = dateStr;
        dateEl.hidden = !config.showDate;
      }
      if (greetingEl) {
        greetingEl.textContent = greetingStr;
        greetingEl.hidden = !config.showGreeting;
      }
    }
  }

  function scheduleNextTick() {
    if (timerId) {
      clearTimeout(timerId);
      timerId = null;
    }

    updateClockDisplay();

    const config = getConfig();
    if (!config.enabled) return;

    const now = new Date();
    if (config.seconds) {
      const delay = 1000 - now.getMilliseconds();
      timerId = setTimeout(() => {
        scheduleNextTick();
      }, Math.max(20, delay));
      if (typeof timerId === 'object' && typeof timerId?.unref === 'function') timerId.unref();
    } else {
      const delay = (60 - now.getSeconds()) * 1000 - now.getMilliseconds();
      timerId = setTimeout(() => {
        scheduleNextTick();
      }, Math.max(100, delay));
      if (typeof timerId === 'object' && typeof timerId?.unref === 'function') timerId.unref();
    }
  }

  function syncControls() {
    const config = getConfig();

    const enabledInput = $('clock-enabled');
    const optionsWrap = $('clock-options');
    const fontInput = $('clock-font');
    const formatInput = $('clock-format');
    const positionInput = $('clock-position');
    const secondsInput = $('clock-seconds');
    const dateInput = $('clock-date');
    const greetingInput = $('clock-greeting');

    if (enabledInput) enabledInput.checked = config.enabled;
    if (optionsWrap) optionsWrap.hidden = !config.enabled;
    if (fontInput) fontInput.value = config.font;
    if (formatInput) formatInput.value = config.format;
    if (positionInput) positionInput.value = config.position;
    if (secondsInput) secondsInput.checked = config.seconds;
    if (dateInput) dateInput.checked = config.showDate;
    if (greetingInput) greetingInput.checked = config.showGreeting;
  }

  function notifyUpdated() {
    try {
      window.dispatchEvent(new CustomEvent('spark_clock_updated', { detail: getConfig() }));
    } catch {}
  }

  function bindEvents() {
    $('clock-enabled')?.addEventListener('change', event => {
      setStorage('spark_clock_enabled', event.target.checked ? 'true' : 'false');
      syncControls();
      scheduleNextTick();
      notifyUpdated();
    });

    $('clock-font')?.addEventListener('change', event => {
      setStorage('spark_clock_font', event.target.value);
      applyFont(event.target.value);
      notifyUpdated();
    });

    $('clock-format')?.addEventListener('change', event => {
      setStorage('spark_clock_format', event.target.value);
      scheduleNextTick();
      notifyUpdated();
    });

    $('clock-position')?.addEventListener('change', event => {
      setStorage('spark_clock_position', event.target.value);
      scheduleNextTick();
      notifyUpdated();
    });

    $('clock-seconds')?.addEventListener('change', event => {
      setStorage('spark_clock_seconds', event.target.checked ? 'true' : 'false');
      scheduleNextTick();
      notifyUpdated();
    });

    $('clock-date')?.addEventListener('change', event => {
      setStorage('spark_clock_date', event.target.checked ? 'true' : 'false');
      scheduleNextTick();
      notifyUpdated();
    });

    $('clock-greeting')?.addEventListener('change', event => {
      setStorage('spark_clock_greeting', event.target.checked ? 'true' : 'false');
      scheduleNextTick();
      notifyUpdated();
    });

    window.addEventListener('storage', event => {
      if (event.key && event.key.startsWith('spark_clock_')) {
        syncControls();
        scheduleNextTick();
      }
    });

    window.addEventListener('spark_clock_updated', () => {
      syncControls();
      scheduleNextTick();
    });

    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'visible') {
        scheduleNextTick();
      }
    });
  }

  window.addEventListener('spark-new-tab', event => {
    if (event.detail?.labels) {
      labels = event.detail.labels;
    }
    if (event.detail?.preferences?.language) {
      language = event.detail.preferences.language;
    }
    syncControls();
    scheduleNextTick();
  });

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      bindEvents();
      syncControls();
      scheduleNextTick();
    });
  } else {
    bindEvents();
    syncControls();
    scheduleNextTick();
  }
})();
