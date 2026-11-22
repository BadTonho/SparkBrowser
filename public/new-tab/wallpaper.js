(() => {
  const $ = id => (typeof document !== 'undefined' && typeof document.getElementById === 'function' ? document.getElementById(id) : null);
  const DB_NAME = 'spark_newtab_wallpaper';
  const DB_VERSION = 1;
  const STORE_NAME = 'wallpaper';
  const KEY = 'current';

  let labels = {};
  let currentMedia = null; // { mediaType: 'image' | 'video', url: string }
  let activeBlobUrl = null;
  let databasePromise = null;
  let recordPromise = null;
  let recordVersion = 0;
  let renderedRecord = undefined;
  const text = key => labels[`newtab.widgets.${key}`] ?? key;

  const getStorage = key => {
    try {
      return (typeof localStorage !== 'undefined' && localStorage.getItem(key)) || null;
    } catch (error) {
      logFailure(`Wallpaper preferences: read ${key}`, error);
      return null;
    }
  };

  const setStorage = (key, value) => {
    try {
      if (typeof localStorage !== 'undefined') localStorage.setItem(key, value);
    } catch (error) { logFailure(`Wallpaper preferences: write ${key}`, error); }
  };

  const logFailure = (context, error) => {
    window.__sparkLogIgnored?.(new Error(`${context}: ${error?.message ?? error}`, { cause: error }));
  };
  const framing = window.__sparkCreateWallpaperFraming(getStorage, setStorage, logFailure);

  function readDimming() {
    const dim = Number.parseInt(getStorage('spark_wallpaper_dim') ?? '40', 10);
    return Math.min(90, Math.max(0, Number.isFinite(dim) ? dim : 40));
  }

  function renderDimming(dim) {
    for (const id of ['wallpaper-overlay', 'wallpaper-preview-overlay']) {
      $(id)?.style.setProperty('--wallpaper-dim', String(dim / 100));
    }
    const dimVal = $('wallpaper-dim-val');
    if (dimVal) dimVal.textContent = `${dim}%`;
  }

  function cleanupUrls() {
    if (activeBlobUrl && typeof URL !== 'undefined' && typeof URL.revokeObjectURL === 'function') {
      try { URL.revokeObjectURL(activeBlobUrl); } catch (error) { window.__sparkLogIgnored?.(error); }
      activeBlobUrl = null;
    }
  }

  function openDB() {
    if (databasePromise) return databasePromise;
    databasePromise = new Promise((resolve, reject) => {
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
        req.onsuccess = () => {
          const db = req.result;
          db.onversionchange = () => { db.close?.(); databasePromise = null; };
          resolve(db);
        };
        req.onerror = () => reject(req.error);
      } catch (e) {
        reject(e);
      }
    }).catch(error => { databasePromise = null; throw error; });
    return databasePromise;
  }

  function invalidateRecord() { recordVersion++; recordPromise = null; }

  async function getStoredWallpaper() {
    if (recordPromise) return recordPromise;
    const pending = readStoredWallpaper().catch(() => { if (recordPromise === pending) recordPromise = null; return null; });
    recordPromise = pending;
    return pending;
  }

  async function readStoredWallpaper() {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readonly');
      const req = tx.objectStore(STORE_NAME).get(KEY);
      req.onsuccess = () => resolve(req.result || null);
      req.onerror = () => reject(req.error);
    });
  }

  async function saveStoredWallpaper(media) {
    try {
      const db = await openDB();
      return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readwrite');
        const req = tx.objectStore(STORE_NAME).put(media, KEY);
        req.onsuccess = () => { invalidateRecord(); resolve(); };
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
        req.onsuccess = () => { invalidateRecord(); resolve(); };
        req.onerror = () => reject(req.error);
      });
    } catch (error) { window.__sparkLogIgnored?.(error); }
  }

  function notifyUpdated() {
    setStorage('spark_wallpaper_updated', Date.now().toString());
  }

  function setStatus(msg) {
    const status = $('wallpaper-status');
    if (status) status.textContent = msg;
  }

  async function loadMediaRecord() {
    const version = recordVersion;
    const record = await getStoredWallpaper();
    if (version !== recordVersion) return loadMediaRecord();
    if (record === renderedRecord) return;
    framing.discardGesture();
    renderedRecord = record;
    cleanupUrls();

    if (!record) {
      currentMedia = null;
      return;
    }

    if (typeof record === 'string') {
      const isVideo = record.startsWith('data:video/');
      currentMedia = {
        mediaType: isVideo ? 'video' : 'image',
        url: record
      };
      return;
    }

    if (record.blob) {
      const url = typeof URL !== 'undefined' && typeof URL.createObjectURL === 'function'
        ? URL.createObjectURL(record.blob)
        : (record.dataUrl || '');
      activeBlobUrl = url;
      currentMedia = {
        mediaType: record.mediaType || (record.mime?.startsWith('video/') ? 'video' : 'image'),
        url
      };
      return;
    }

    if (record.dataUrl) {
      currentMedia = {
        mediaType: record.mediaType || 'image',
        url: record.dataUrl
      };
    } else {
      currentMedia = null;
    }
  }

  async function applyWallpaper() {
    await loadMediaRecord();

    const isEnabled = getStorage('spark_wallpaper_enabled') === 'true';
    const dim = readDimming();
    const container = $('wallpaper-container');
    const image = $('wallpaper-image');
    const video = $('wallpaper-video');
    const enabledToggle = $('wallpaper-enabled');
    const options = $('wallpaper-options');
    const dimInput = $('wallpaper-dim');
    const preview = $('wallpaper-preview');
    const videoPreview = $('wallpaper-video-preview');
    const editor = $('wallpaper-editor');
    const removeBtn = $('wallpaper-remove');

    if (enabledToggle) enabledToggle.checked = isEnabled;
    if (options) options.hidden = !isEnabled;
    if (dimInput) dimInput.value = dim.toString();
    renderDimming(dim);

    const active = isEnabled && Boolean(currentMedia?.url);

    if (container) {
      if (active) {
        container.hidden = false;
        if (document.body) document.body.classList.add('has-wallpaper');

        if (currentMedia.mediaType === 'video') {
          if (image) {
            image.hidden = true;
            image.removeAttribute('src');
          }
          if (video) {
            video.hidden = false;
            if (video.getAttribute('src') !== currentMedia.url) {
              video.src = currentMedia.url;
              try { video.play?.(); } catch (error) { window.__sparkLogIgnored?.(error); }
            }
          }
        } else {
          if (video) {
            video.hidden = true;
            try { video.pause?.(); } catch (error) { window.__sparkLogIgnored?.(error); }
            video.removeAttribute('src');
          }
          if (image) {
            image.hidden = false;
            if (image.getAttribute('src') !== currentMedia.url) image.src = currentMedia.url;
          }
        }
      } else {
        container.hidden = true;
        if (document.body) document.body.classList.remove('has-wallpaper');
        if (image) {
          image.hidden = true;
          image.removeAttribute('src');
        }
        if (video) {
          video.hidden = true;
          try { video.pause?.(); } catch (error) { window.__sparkLogIgnored?.(error); }
          video.removeAttribute('src');
        }
      }
    }

    if (editor && removeBtn) {
      if (currentMedia?.url) {
        editor.hidden = false;
        removeBtn.hidden = false;
        if (currentMedia.mediaType === 'video') {
          if (preview) {
            preview.hidden = true;
            preview.removeAttribute('src');
          }
          if (videoPreview) {
            videoPreview.hidden = false;
            if (videoPreview.getAttribute('src') !== currentMedia.url) {
              videoPreview.src = currentMedia.url;
              try {
                videoPreview.play()?.catch(error => logFailure('Wallpaper preview: play video', error));
              } catch (error) {
                logFailure('Wallpaper preview: start video playback', error);
              }
            }
          }
        } else {
          if (videoPreview) {
            videoPreview.hidden = true;
            try { videoPreview.pause?.(); } catch (error) { window.__sparkLogIgnored?.(error); }
            videoPreview.removeAttribute('src');
          }
          if (preview) {
            preview.hidden = false;
            if (preview.getAttribute('src') !== currentMedia.url) preview.src = currentMedia.url;
          }
        }
      } else {
        editor.hidden = true;
        removeBtn.hidden = true;
        if (preview) {
          preview.hidden = true;
          preview.removeAttribute('src');
        }
        if (videoPreview) {
          videoPreview.hidden = true;
          try { videoPreview.pause?.(); } catch (error) { window.__sparkLogIgnored?.(error); }
          videoPreview.removeAttribute('src');
        }
      }
    }
    framing.render();
  }

  function initFramingHelp() {
    const help = $('wallpaper-help');
    const tooltip = $('wallpaper-help-tooltip');
    const reopen = () => help.removeAttribute('data-dismissed');
    help.addEventListener('pointerenter', reopen);
    help.addEventListener('focusin', reopen);
    $('wallpaper-help-button').addEventListener('click', reopen);
    $('settings-panel').addEventListener('keydown', event => {
      if (event.key !== 'Escape' || getComputedStyle(tooltip).display === 'none') return;
      event.preventDefault();
      help.setAttribute('data-dismissed', '');
    });
  }

  function initControls() {
    framing.init();
    initFramingHelp();
    for (const type of ['image', 'video']) {
      const preview = $(type === 'image' ? 'wallpaper-preview' : 'wallpaper-video-preview');
      preview?.addEventListener('error', () => {
        if (!preview.getAttribute('src')) return;
        logFailure(`Wallpaper preview: load ${type}`, preview.error ?? new Error('Could not decode media'));
        setStatus(text('wallpaperError'));
      });
    }
    const enabledToggle = $('wallpaper-enabled');
    const dimInput = $('wallpaper-dim');
    const fileInput = $('wallpaper-file');
    const chooseBtn = $('wallpaper-choose');
    const removeBtn = $('wallpaper-remove');

    if (enabledToggle) {
      enabledToggle.addEventListener('change', async event => {
        $('menu-item-wallpaper')?.click();
        const checked = event.target.checked;
        setStorage('spark_wallpaper_enabled', checked ? 'true' : 'false');
        await applyWallpaper();
        if (checked && !currentMedia?.url) {
          setStatus(text('chooseWallpaper'));
        } else {
          setStatus('');
        }
      });
    }

    if (dimInput) {
      dimInput.addEventListener('input', event => renderDimming(Number(event.target.value)));
      dimInput.addEventListener('change', event => {
        setStorage('spark_wallpaper_dim', event.target.value);
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

        const isVideo = file.type.startsWith('video/') || /\.(mp4|webm|ogg)$/i.test(file.name);
        const maxVideoSize = 50 * 1024 * 1024; // 50MB

        if (isVideo) {
          if (file.size > maxVideoSize) {
            setStatus(text('wallpaperVideoTooLarge'));
            return;
          }

          setStatus(text('wallpaperLoading'));
          saveStoredWallpaper({
            mediaType: 'video',
            mime: file.type || 'video/mp4',
            blob: file
          })
          .then(async () => {
            framing.reset();
            setStorage('spark_wallpaper_enabled', 'true');
            notifyUpdated();
            await applyWallpaper();
            setStatus(text('wallpaperSaved'));
          })
          .catch(() => setStatus(text('wallpaperError')));
          return;
        }

        if (file.size > 20 * 1024 * 1024) {
          setStatus(text('wallpaperImageTooLarge'));
          return;
        }

        setStatus(text('wallpaperLoading'));
        const loadImage = (source, release) => {
          if (typeof Image === 'undefined') {
            saveStoredWallpaper({
              mediaType: 'image',
              mime: file.type || 'image/png',
              blob: file
            })
              .then(async () => {
                framing.reset();
                setStorage('spark_wallpaper_enabled', 'true');
                notifyUpdated();
                await applyWallpaper();
                setStatus(text('wallpaperSaved'));
              })
              .catch(() => setStatus(text('wallpaperError')))
              .finally(release);
            return;
          }

          const img = new Image();
          img.onload = async () => {
            let w = img.naturalWidth || img.width;
            let h = img.naturalHeight || img.height;
            const maxDim = 2560;
            let finalBlob = file;

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
                  if (typeof canvas.toBlob === 'function') {
                    finalBlob = await new Promise(res => canvas.toBlob(res, 'image/jpeg', 0.9)) || file;
                  }
                }
              } catch { finalBlob = file; }
            }

            try {
              await saveStoredWallpaper({
                mediaType: 'image',
                mime: finalBlob.type || file.type || 'image/jpeg',
                blob: finalBlob
              });
              framing.reset();
              setStorage('spark_wallpaper_enabled', 'true');
              notifyUpdated();
              await applyWallpaper();
              setStatus(text('wallpaperSaved'));
            } catch {
              setStatus(text('wallpaperError'));
            } finally {
              release();
            }
          };
          img.onerror = () => { release(); setStatus(text('wallpaperError')); };
          img.src = source;
        };
        if (typeof URL !== 'undefined' && typeof URL.createObjectURL === 'function') {
          const source = URL.createObjectURL(file);
          loadImage(source, () => URL.revokeObjectURL(source));
        } else {
          const reader = new FileReader();
          reader.onload = () => loadImage(reader.result, () => {});
          reader.onerror = () => setStatus(text('wallpaperError'));
          reader.readAsDataURL(file);
        }
      });
    }

    if (removeBtn) {
      removeBtn.addEventListener('click', async () => {
        cleanupUrls();
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
      if (framing.handlesStorage(event.key)) {
        framing.render();
        return;
      }
      if (
        event.key === 'spark_wallpaper_updated' ||
        event.key === 'spark_wallpaper_enabled' ||
        event.key === 'spark_wallpaper_dim'
      ) {
        if (event.key === 'spark_wallpaper_updated') invalidateRecord();
        applyWallpaper();
      }
    });

    window.addEventListener('spark-new-tab', event => {
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
