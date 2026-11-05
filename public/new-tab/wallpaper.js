(() => {
  const $ = id => (typeof document !== 'undefined' && typeof document.getElementById === 'function' ? document.getElementById(id) : null);
  const DB_NAME = 'veloce_newtab_wallpaper';
  const DB_VERSION = 1;
  const STORE_NAME = 'wallpaper';
  const KEY = 'current';

  let labels = {};
  let currentDataUrl = null;
  const text = key => labels[`newtab.widgets.${key}`] ?? key;

  const getStorage = key => {
    try {
      return (typeof localStorage !== 'undefined' && localStorage.getItem(key)) || null;
    } catch {
      return null;
    }
  };

  const setStorage = (key, value) => {
    try {
      if (typeof localStorage !== 'undefined') localStorage.setItem(key, value);
    } catch {}
  };

  const removeStorage = key => {
    try {
      if (typeof localStorage !== 'undefined') localStorage.removeItem(key);
    } catch {}
  };

  function openDB() {
    return new Promise((resolve, reject) => {
      if (typeof indexedDB === 'undefined') {
        reject(new Error('IndexedDB unavailable'));
        return;
      }
      try {
        const req = indexedDB.open(DB_NAME, DB_VERSION);
        req.onupgradeneeded = () => {
          const db = req.result;
          if (!db.objectStoreNames.contains(STORE_NAME)) {
            db.createObjectStore(STORE_NAME);
          }
        };
        req.onsuccess = () => resolve(req.result);
        req.onerror = () => reject(req.error);
      } catch (e) {
        reject(e);
      }
    });
  }

  async function getStoredWallpaper() {
    try {
      const db = await openDB();
      return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readonly');
        const req = tx.objectStore(STORE_NAME).get(KEY);
        req.onsuccess = () => resolve(req.result || null);
        req.onerror = () => reject(req.error);
      });
    } catch {
      return null;
    }
  }

  async function saveStoredWallpaper(dataUrl) {
    try {
      const db = await openDB();
      return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readwrite');
        const req = tx.objectStore(STORE_NAME).put(dataUrl, KEY);
        req.onsuccess = () => resolve();
        req.onerror = () => reject(req.error);
      });
    } catch (e) {
      console.error('Failed to save wallpaper', e);
      throw e;
    }
  }

  async function deleteStoredWallpaper() {
    try {
      const db = await openDB();
      return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readwrite');
        const req = tx.objectStore(STORE_NAME).delete(KEY);
        req.onsuccess = () => resolve();
        req.onerror = () => reject(req.error);
      });
    } catch {}
  }

  function notifyUpdated() {
    setStorage('veloce_wallpaper_updated', Date.now().toString());
  }

  function setStatus(msg) {
    const status = $('wallpaper-status');
    if (status) status.textContent = msg;
  }

  async function applyWallpaper() {
    const isEnabled = getStorage('veloce_wallpaper_enabled') === 'true';
    const dim = Math.min(90, Math.max(0, parseInt(getStorage('veloce_wallpaper_dim') ?? '40', 10) || 40));

    currentDataUrl = await getStoredWallpaper();

    const container = $('wallpaper-container');
    const image = $('wallpaper-image');
    const overlay = $('wallpaper-overlay');
    const enabledToggle = $('wallpaper-enabled');
    const options = $('wallpaper-options');
    const dimInput = $('wallpaper-dim');
    const dimVal = $('wallpaper-dim-val');
    const preview = $('wallpaper-preview');
    const previewWrap = $('wallpaper-preview-wrap');
    const removeBtn = $('wallpaper-remove');

    if (enabledToggle) enabledToggle.checked = isEnabled;
    if (options) options.hidden = !isEnabled;
    if (dimInput) dimInput.value = dim.toString();
    if (dimVal) dimVal.textContent = `${dim}%`;

    const active = isEnabled && Boolean(currentDataUrl);

    if (container && image) {
      if (active) {
        container.hidden = false;
        image.src = currentDataUrl;
        if (overlay) overlay.style.setProperty('--wallpaper-dim', (dim / 100).toString());
        if (document.body) document.body.classList.add('has-wallpaper');
      } else {
        container.hidden = true;
        image.removeAttribute('src');
        if (document.body) document.body.classList.remove('has-wallpaper');
      }
    }

    if (preview && previewWrap && removeBtn) {
      if (currentDataUrl) {
        previewWrap.hidden = false;
        preview.src = currentDataUrl;
        removeBtn.hidden = false;
      } else {
        previewWrap.hidden = true;
        preview.removeAttribute('src');
        removeBtn.hidden = true;
      }
    }
  }

  function initControls() {
    const enabledToggle = $('wallpaper-enabled');
    const dimInput = $('wallpaper-dim');
    const dimVal = $('wallpaper-dim-val');
    const fileInput = $('wallpaper-file');
    const chooseBtn = $('wallpaper-choose');
    const removeBtn = $('wallpaper-remove');

    if (enabledToggle) {
      enabledToggle.addEventListener('change', async event => {
        const checked = event.target.checked;
        setStorage('veloce_wallpaper_enabled', checked ? 'true' : 'false');
        notifyUpdated();
        await applyWallpaper();
        if (checked && !currentDataUrl) {
          setStatus(text('chooseWallpaper'));
        } else {
          setStatus('');
        }
      });
    }

    if (dimInput) {
      dimInput.addEventListener('input', event => {
        const val = event.target.value;
        if (dimVal) dimVal.textContent = `${val}%`;
        const overlay = $('wallpaper-overlay');
        if (overlay) overlay.style.setProperty('--wallpaper-dim', (parseInt(val, 10) / 100).toString());
      });
      dimInput.addEventListener('change', event => {
        const val = event.target.value;
        setStorage('veloce_wallpaper_dim', val);
        notifyUpdated();
      });
    }

    if (chooseBtn && fileInput) {
      chooseBtn.addEventListener('click', () => {
        fileInput.value = '';
        fileInput.click();
      });

      fileInput.addEventListener('change', () => {
        const file = fileInput.files?.[0];
        if (!file) return;

        setStatus(text('wallpaperLoading'));

        const reader = new FileReader();
        reader.onload = () => {
          const rawData = reader.result;
          if (typeof Image === 'undefined') {
            saveStoredWallpaper(rawData)
              .then(() => {
                setStorage('veloce_wallpaper_enabled', 'true');
                notifyUpdated();
                applyWallpaper();
                setStatus(text('wallpaperSaved'));
              })
              .catch(() => setStatus(text('wallpaperError')));
            return;
          }

          const img = new Image();
          img.onload = async () => {
            let w = img.naturalWidth || img.width;
            let h = img.naturalHeight || img.height;
            const maxDim = 2560;
            let finalData = rawData;

            if (typeof document !== 'undefined' && (w > maxDim || h > maxDim)) {
              try {
                if (w > h) {
                  h = Math.round((h * maxDim) / w);
                  w = maxDim;
                } else {
                  w = Math.round((w * maxDim) / h);
                  h = maxDim;
                }
                const canvas = document.createElement('canvas');
                canvas.width = w;
                canvas.height = h;
                const ctx = canvas.getContext('2d');
                if (ctx) {
                  ctx.drawImage(img, 0, 0, w, h);
                  finalData = canvas.toDataURL('image/jpeg', 0.9);
                }
              } catch {
                finalData = rawData;
              }
            }

            try {
              await saveStoredWallpaper(finalData);
              setStorage('veloce_wallpaper_enabled', 'true');
              notifyUpdated();
              await applyWallpaper();
              setStatus(text('wallpaperSaved'));
            } catch {
              setStatus(text('wallpaperError'));
            }
          };
          img.onerror = () => setStatus(text('wallpaperError'));
          img.src = rawData;
        };
        reader.onerror = () => setStatus(text('wallpaperError'));
        reader.readAsDataURL(file);
      });
    }

    if (removeBtn) {
      removeBtn.addEventListener('click', async () => {
        await deleteStoredWallpaper();
        notifyUpdated();
        await applyWallpaper();
        setStatus('');
      });
    }
  }

  // Cross-tab synchronization via storage events
  if (typeof window !== 'undefined' && typeof window.addEventListener === 'function') {
    window.addEventListener('storage', event => {
      if (
        event.key === 'veloce_wallpaper_updated' ||
        event.key === 'veloce_wallpaper_enabled' ||
        event.key === 'veloce_wallpaper_dim'
      ) {
        applyWallpaper();
      }
    });

    window.addEventListener('veloce-new-tab', event => {
      labels = event.detail.labels || {};
      for (const node of document.querySelectorAll?.('[data-label]') ?? []) {
        const val = text(node.dataset.label);
        if (node.classList?.contains('icon-button')) {
          node.title = val;
          node.setAttribute('aria-label', val);
        } else {
          node.textContent = val;
        }
      }
      applyWallpaper();
    });

    if (typeof document !== 'undefined') {
      if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
          initControls();
          applyWallpaper();
        });
      } else {
        initControls();
        applyWallpaper();
      }
    }
  }
})();
