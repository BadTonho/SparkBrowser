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
    } catch {
      return null;
    }
  };

  const setStorage = (key, value) => {
    try {
      if (typeof localStorage !== 'undefined') localStorage.setItem(key, value);
    } catch (error) { window.__sparkLogIgnored?.(error); }
  };

  const removeStorage = key => {
    try {
      if (typeof localStorage !== 'undefined') localStorage.removeItem(key);
    } catch (error) { window.__sparkLogIgnored?.(error); }
  };

  const clampPct = value => Math.min(100, Math.max(0, Number.isFinite(value) ? value : 50));

  function readPosition() {
    const [x, y] = (getStorage('spark_wallpaper_pos') ?? '50,50').split(',').map(Number);
    return { x: clampPct(x), y: clampPct(y) };
  }

  function readZoom() {
    const value = Number(getStorage('spark_wallpaper_zoom') ?? 100);
    return Math.min(300, Math.max(25, Number.isFinite(value) ? value : 100));
  }

  const MEDIA = [
    ['wallpaper-image', 'wallpaper-container'], ['wallpaper-video', 'wallpaper-container'],
    ['wallpaper-preview', 'wallpaper-preview-wrap'], ['wallpaper-video-preview', 'wallpaper-preview-wrap']
  ];
  let zoomLive = null;

  const naturalSize = node => ({ w: node.naturalWidth || node.videoWidth || 0, h: node.naturalHeight || node.videoHeight || 0 });

  // Zoom 100% is the picture filling the whole area; lower shows more of it (and the margins around),
  // higher enlarges it. Where the picture overflows the area it can be panned; where it fits, it is centred.
  function mediaGeometry(node, box, pos, zoom) {
    const { w: nw, h: nh } = naturalSize(node);
    const W = box.clientWidth, H = box.clientHeight;
    if (!nw || !nh || !W || !H) return null;
    const scale = Math.max(W / nw, H / nh) * (zoom / 100);
    const w = nw * scale, h = nh * scale;
    // Only an axis where the picture is bigger than the area can be panned; otherwise it stays centred.
    const place = (room, focus) => (room >= 0 ? room / 2 : room * focus / 100);
    return { w, h, W, H, left: place(W - w, pos.x), top: place(H - h, pos.y) };
  }

  function fitPreviewBox() {
    const wrap = $('wallpaper-preview-wrap');
    if (!wrap || !window.innerWidth || !window.innerHeight) return;
    // The preview has the shape of the screen, so it shows the same framing as the new tab.
    const ratio = window.innerWidth / window.innerHeight, avail = wrap.parentElement?.clientWidth || 0;
    let width = 180 * ratio;
    if (avail && width > avail) width = avail;
    const size = [`${Math.round(width)}px`, `${Math.round(width / ratio)}px`];
    if (wrap.style.width !== size[0]) wrap.style.width = size[0];
    if (wrap.style.height !== size[1]) wrap.style.height = size[1];
  }

  function applyPosition(pos = readPosition(), zoom = zoomLive ?? readZoom()) {
    fitPreviewBox();
    for (const [id, boxId] of MEDIA) {
      const node = $(id), box = $(boxId);
      if (!node || !box) continue;
      const g = mediaGeometry(node, box, pos, zoom);
      const style = node.style;
      if (!g) {
        // Not measurable yet (hidden or still loading): the stylesheet's "cover" applies meanwhile.
        for (const key of ['left', 'top', 'width', 'height', 'objectFit', 'inset']) style[key] = '';
        continue;
      }
      style.inset = 'auto'; style.objectFit = 'fill';
      style.width = `${g.w}px`; style.height = `${g.h}px`;
      style.left = `${g.left}px`; style.top = `${g.top}px`;
    }
  }

  function resetPosition() {
    setStorage('spark_wallpaper_pos', '50,50');
    setStorage('spark_wallpaper_zoom', '100');
  }

  function initPositionDrag() {
    const wrap = $('wallpaper-preview-wrap');
    if (!wrap) return;
    for (const [id] of MEDIA) {
      const node = $(id);
      node?.addEventListener?.('load', () => applyPosition());
      node?.addEventListener?.('loadedmetadata', () => applyPosition());
    }
    window.addEventListener('resize', () => applyPosition());
    if (typeof ResizeObserver !== 'undefined') new ResizeObserver(() => applyPosition()).observe(wrap);
    let drag = null;
    wrap.addEventListener('pointerdown', event => {
      if (event.button !== 0) return;
      const media = currentMedia?.mediaType === 'video' ? $('wallpaper-video-preview') : $('wallpaper-preview');
      if (!media || media.hidden) return;
      const origin = readPosition();
      const g = mediaGeometry(media, wrap, origin, zoomLive ?? readZoom());
      if (!g) return;
      drag = { id: event.pointerId, startX: event.clientX, startY: event.clientY, origin, slackX: g.W - g.w, slackY: g.H - g.h, pos: origin };
      try { wrap.setPointerCapture(event.pointerId); } catch (error) { window.__sparkLogIgnored?.(error); }
      wrap.classList.add('dragging');
      event.preventDefault();
    });
    wrap.addEventListener('pointermove', event => {
      if (!drag || event.pointerId !== drag.id) return;
      // The picture follows the pointer: left = slack * x, so x moves by dx / slack (slack < 0 when it overflows).
      const x = drag.slackX < -0.5 ? drag.origin.x + ((event.clientX - drag.startX) / drag.slackX) * 100 : drag.origin.x;
      const y = drag.slackY < -0.5 ? drag.origin.y + ((event.clientY - drag.startY) / drag.slackY) * 100 : drag.origin.y;
      drag.pos = { x: clampPct(x), y: clampPct(y) };
      applyPosition(drag.pos);
    });
    const finish = event => {
      if (!drag || event.pointerId !== drag.id) return;
      const { pos } = drag;
      drag = null;
      wrap.classList.remove('dragging');
      setStorage('spark_wallpaper_pos', `${Math.round(pos.x * 10) / 10},${Math.round(pos.y * 10) / 10}`);
      notifyUpdated();
    };
    wrap.addEventListener('pointerup', finish);
    wrap.addEventListener('pointercancel', finish);
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
    const isEnabled = getStorage('spark_wallpaper_enabled') === 'true';
    const dim = Math.min(90, Math.max(0, parseInt(getStorage('spark_wallpaper_dim') ?? '40', 10) || 40));

    await loadMediaRecord();

    const container = $('wallpaper-container');
    const image = $('wallpaper-image');
    const video = $('wallpaper-video');
    const overlay = $('wallpaper-overlay');
    const enabledToggle = $('wallpaper-enabled');
    const options = $('wallpaper-options');
    const dimInput = $('wallpaper-dim');
    const dimVal = $('wallpaper-dim-val');
    const preview = $('wallpaper-preview');
    const videoPreview = $('wallpaper-video-preview');
    const previewWrap = $('wallpaper-preview-wrap');
    const removeBtn = $('wallpaper-remove');

    if (enabledToggle) enabledToggle.checked = isEnabled;
    if (options) options.hidden = !isEnabled;
    if (dimInput) dimInput.value = dim.toString();
    if (dimVal) dimVal.textContent = `${dim}%`;

    const active = isEnabled && Boolean(currentMedia?.url);

    if (container) {
      if (active) {
        container.hidden = false;
        if (overlay) overlay.style.setProperty('--wallpaper-dim', (dim / 100).toString());
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
            image.src = currentMedia.url;
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

    if (previewWrap && removeBtn) {
      if (currentMedia?.url) {
        previewWrap.hidden = false;
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
              try { videoPreview.play?.(); } catch (error) { window.__sparkLogIgnored?.(error); }
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
            preview.src = currentMedia.url;
          }
        }
      } else {
        previewWrap.hidden = true;
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
    const zoomInput = $('wallpaper-zoom'), zoomVal = $('wallpaper-zoom-val');
    if (zoomInput) zoomInput.value = String(readZoom());
    if (zoomVal) zoomVal.textContent = `${readZoom()}%`;
    applyPosition();
  }

  function initControls() {
    initPositionDrag();
    const enabledToggle = $('wallpaper-enabled');
    const dimInput = $('wallpaper-dim');
    const dimVal = $('wallpaper-dim-val');
    const fileInput = $('wallpaper-file');
    const chooseBtn = $('wallpaper-choose');
    const removeBtn = $('wallpaper-remove');

    if (enabledToggle) {
      enabledToggle.addEventListener('change', async event => {
        $('menu-item-wallpaper')?.click();
        const checked = event.target.checked;
        setStorage('spark_wallpaper_enabled', checked ? 'true' : 'false');
        notifyUpdated();
        await applyWallpaper();
        if (checked && !currentMedia?.url) {
          setStatus(text('chooseWallpaper'));
        } else {
          setStatus('');
        }
      });
    }

    const zoomInput = $('wallpaper-zoom');
    if (zoomInput) {
      const zoomVal = $('wallpaper-zoom-val');
      zoomInput.addEventListener('input', event => {
        const val = Number(event.target.value);
        if (zoomVal) zoomVal.textContent = `${val}%`;
        zoomLive = val;
        // Zooming always happens around the centre; drag the preview afterwards to pan.
        applyPosition({ x: 50, y: 50 }, val);
      });
      zoomInput.addEventListener('change', event => {
        zoomLive = null;
        setStorage('spark_wallpaper_pos', '50,50');
        setStorage('spark_wallpaper_zoom', String(event.target.value));
        notifyUpdated();
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
        setStorage('spark_wallpaper_dim', val);
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
            resetPosition();
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
                resetPosition();
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
              resetPosition();
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
      if (
        event.key === 'spark_wallpaper_updated' ||
        event.key === 'spark_wallpaper_enabled' ||
        event.key === 'spark_wallpaper_dim' ||
        event.key === 'spark_wallpaper_pos' ||
        event.key === 'spark_wallpaper_zoom'
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
