(() => {
  const POSITION_KEY = 'spark_wallpaper_pos';
  const ZOOM_KEY = 'spark_wallpaper_zoom';
  const CENTER = { x: 50, y: 50 };
  const ZOOM_STEP = 5;
  const WHEEL_STEP = 40;
  const CENTER_SNAP_DISTANCE = 8;
  const clamp = (value, min, max) => Math.min(max, Math.max(min, value));
  const clampPosition = value => clamp(Number.isFinite(value) ? value : 50, 0, 100);
  const canMove = room => Math.abs(room) > 0.5;

  function nearCenter(position, room) {
    const travel = Math.abs(room);
    const distance = travel * Math.abs(position - 50) / 100;
    return canMove(room) && distance <= Math.min(CENTER_SNAP_DISTANCE, travel / 4);
  }

  function mediaGeometry(node, box, position, zoom) {
    const naturalWidth = node.naturalWidth || node.videoWidth;
    const naturalHeight = node.naturalHeight || node.videoHeight;
    const { width, height } = box.getBoundingClientRect();
    if (!naturalWidth || !naturalHeight || !width || !height) return null;

    const scale = Math.max(width / naturalWidth, height / naturalHeight) * zoom / 100;
    const mediaWidth = naturalWidth * scale;
    const mediaHeight = naturalHeight * scale;
    const roomX = width - mediaWidth;
    const roomY = height - mediaHeight;
    // Align the same edges with either spare space or overflow in each axis.
    return {
      width: mediaWidth, height: mediaHeight, roomX, roomY,
      left: roomX * position.x / 100, top: roomY * position.y / 100
    };
  }

  window.__sparkCreateWallpaperFraming = (getStorage, setStorage, logFailure) => {
    const $ = id => document.getElementById(id);
    const background = $('wallpaper-container');
    const preview = $('wallpaper-preview-wrap');
    const zoomValue = $('wallpaper-zoom-val');
    const previewMedia = [$('wallpaper-preview'), $('wallpaper-video-preview')];
    const media = [
      ...[$('wallpaper-image'), $('wallpaper-video')].map(node => [node, background]),
      ...previewMedia.map(node => [node, preview])
    ];
    let drag = null;
    let pendingZoom = null;
    let wheelTimer = null;
    let wheelDelta = 0;

    function readPosition() {
      const [x, y] = (getStorage(POSITION_KEY) ?? '50,50').split(',').map(Number);
      return { x: clampPosition(x), y: clampPosition(y) };
    }

    function readZoom() {
      const zoom = Number(getStorage(ZOOM_KEY) ?? 100);
      return clamp(Number.isFinite(zoom) ? zoom : 100, 25, 300);
    }

    function render() {
      const { width, height } = background.getBoundingClientRect();
      const viewportWidth = width || window.innerWidth;
      const viewportHeight = height || window.innerHeight;
      if (viewportWidth && viewportHeight) {
        preview.style.setProperty('--wallpaper-aspect', String(viewportWidth / viewportHeight));
      }

      const position = drag?.position ?? readPosition();
      const zoom = pendingZoom ?? readZoom();
      zoomValue.textContent = `${zoom}%`;
      for (const [node, box] of media) {
        const geometry = node.hidden ? null : mediaGeometry(node, box, position, zoom);
        const { style } = node;
        style.inset = geometry ? 'auto' : '';
        style.objectFit = geometry ? 'fill' : '';
        for (const key of ['width', 'height', 'left', 'top']) {
          style[key] = geometry ? `${geometry[key]}px` : '';
        }
      }
    }

    function currentGeometry() {
      const node = previewMedia.find(item => !item.hidden);
      return node ? mediaGeometry(node, preview, readPosition(), pendingZoom ?? readZoom()) : null;
    }

    function savePosition({ x, y }) {
      setStorage(POSITION_KEY, `${Math.round(x * 10) / 10},${Math.round(y * 10) / 10}`);
    }

    function stopWheel() {
      clearTimeout(wheelTimer);
      wheelTimer = null;
      wheelDelta = 0;
      const zoom = pendingZoom;
      pendingZoom = null;
      return zoom;
    }

    function commitZoom() {
      const zoom = stopWheel();
      if (zoom === null) return;
      setStorage(ZOOM_KEY, String(zoom));
      render();
    }

    function updateDragPosition(shiftHeld) {
      const { freePosition, geometry } = drag;
      const snapX = shiftHeld && nearCenter(freePosition.x, geometry.roomX);
      const snapY = shiftHeld && nearCenter(freePosition.y, geometry.roomY);
      drag.position = { x: snapX ? CENTER.x : freePosition.x, y: snapY ? CENTER.y : freePosition.y };
      preview.classList.toggle('snap-center-x', snapX);
      preview.classList.toggle('snap-center-y', snapY);
    }

    function endDrag(save) {
      if (!drag) return;
      const { id, origin, position } = drag;
      drag = null;
      preview.classList.remove('dragging', 'snap-center-x', 'snap-center-y');
      try {
        if (preview.hasPointerCapture(id)) preview.releasePointerCapture(id);
      } catch (error) {
        logFailure('Wallpaper preview: release pointer capture', error);
      }
      if (save && (origin.x !== position.x || origin.y !== position.y)) savePosition(position);
      render();
    }

    function discardGesture() {
      stopWheel();
      endDrag(false);
    }

    function reset() {
      discardGesture();
      savePosition(CENTER);
      setStorage(ZOOM_KEY, '100');
    }

    function init() {
      for (const [node] of media) {
        node.addEventListener('load', render);
        node.addEventListener('loadedmetadata', render);
      }
      window.addEventListener('resize', render);
      window.addEventListener('pagehide', () => {
        endDrag(true);
        commitZoom();
      });
      const updateShift = event => {
        if (!drag?.moved || event.key !== 'Shift' || event.repeat) return;
        updateDragPosition(event.shiftKey === true);
        render();
      };
      window.addEventListener('keydown', updateShift);
      window.addEventListener('keyup', updateShift);
      if (typeof ResizeObserver !== 'undefined') new ResizeObserver(render).observe(preview);

      preview.addEventListener('pointerdown', event => {
        if (event.button !== 0 || drag) return;
        const geometry = currentGeometry();
        if (!geometry) return;
        commitZoom();
        preview.focus({ preventScroll: true });
        try {
          preview.setPointerCapture(event.pointerId);
        } catch (error) {
          logFailure('Wallpaper preview: capture pointer for repositioning', error);
          return;
        }
        const origin = readPosition();
        drag = {
          id: event.pointerId, x: event.clientX, y: event.clientY, geometry,
          origin, freePosition: origin, position: origin, moved: false
        };
        preview.classList.add('dragging');
        event.preventDefault();
      });

      preview.addEventListener('pointermove', event => {
        if (!drag || event.pointerId !== drag.id) return;
        drag.moved ||= event.clientX !== drag.x || event.clientY !== drag.y;
        if (!drag.moved) return;
        const { roomX, roomY } = drag.geometry;
        const pan = (origin, delta, room) => canMove(room) ? clampPosition(origin + delta / room * 100) : origin;
        drag.freePosition = {
          x: pan(drag.origin.x, event.clientX - drag.x, roomX),
          y: pan(drag.origin.y, event.clientY - drag.y, roomY)
        };
        updateDragPosition(event.shiftKey === true);
        render();
      });

      const finish = event => {
        if (event.pointerId === drag?.id) endDrag(true);
      };
      preview.addEventListener('pointerup', event => {
        if (event.pointerId !== drag?.id) return;
        if (drag.moved) updateDragPosition(event.shiftKey === true);
        endDrag(true);
      });
      preview.addEventListener('pointercancel', finish);
      preview.addEventListener('lostpointercapture', finish);

      preview.addEventListener('wheel', event => {
        const { deltaMode, deltaY } = event;
        if (event.ctrlKey || !deltaY || !currentGeometry()) return;
        event.preventDefault();
        if (drag) return;
        wheelDelta += deltaY * (deltaMode === 0 ? 1 : WHEEL_STEP);
        if (Math.abs(wheelDelta) >= WHEEL_STEP) {
          const zoom = pendingZoom ?? readZoom();
          const next = clamp(zoom - Math.sign(wheelDelta) * ZOOM_STEP, 25, 300);
          wheelDelta = 0;
          if (next !== zoom) {
            pendingZoom = next;
            render();
          }
        }
        clearTimeout(wheelTimer);
        wheelTimer = setTimeout(commitZoom, 180);
      }, { passive: false });

      preview.addEventListener('keydown', event => {
        if (event.altKey || event.ctrlKey || event.metaKey || drag) return;
        const direction = { ArrowLeft: [-5, 0], ArrowRight: [5, 0], ArrowUp: [0, -5], ArrowDown: [0, 5] }[event.key];
        const zoomDirection = event.key === '+' || event.key === '=' ? 1 : event.key === '-' ? -1 : 0;
        if (!direction && !zoomDirection) return;
        const geometry = currentGeometry();
        if (!geometry) return;
        event.preventDefault();
        commitZoom();
        if (zoomDirection) {
          setStorage(ZOOM_KEY, String(clamp(readZoom() + zoomDirection * ZOOM_STEP, 25, 300)));
        } else {
          const position = readPosition();
          if (canMove(geometry.roomX)) position.x = clampPosition(position.x + direction[0] * Math.sign(geometry.roomX));
          if (canMove(geometry.roomY)) position.y = clampPosition(position.y + direction[1] * Math.sign(geometry.roomY));
          savePosition(position);
        }
        render();
      });
    }

    return { init, render, reset, discardGesture, handlesStorage: key => key === POSITION_KEY || key === ZOOM_KEY };
  };
})();
