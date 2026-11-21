// Drawing styles for the audio-reactive background.
// Each style is { layout: 'band' | 'full', draw(ctx, w, h, d, s) } where `d` holds 32 normalized
// levels (0..1) and `s` is { color, t (seconds), dt (seconds), store, gain (visual strength, 0.5..2) } with per-style state.
// The default `bars` style lives in audio-background.js so the effect still works if this file fails.
(() => {
  const TAU = Math.PI * 2, N = 32;
  const rgba = (hex, alpha) => {
    const n = parseInt(hex.slice(1), 16);
    return `rgba(${(n >> 16) & 255},${(n >> 8) & 255},${n & 255},${alpha})`;
  };
  const avg = (d, from, to) => { let sum = 0; for (let i = from; i < to; i++) sum += d[i]; return sum / (to - from); };
  const bass = d => avg(d, 0, 6), mid = d => avg(d, 6, 18), treble = d => avg(d, 18, N), level = d => avg(d, 0, N);
  const group = (d, index, count) => { const size = N / count; return avg(d, Math.floor(index * size), Math.floor((index + 1) * size)); };
  // Smooth path through points using quadratic curves between midpoints.
  function curve(ctx, points) {
    ctx.moveTo(points[0][0], points[0][1]);
    for (let i = 1; i < points.length - 1; i++) {
      const [x, y] = points[i], [nx, ny] = points[i + 1];
      ctx.quadraticCurveTo(x, y, (x + nx) / 2, (y + ny) / 2);
    }
    const last = points[points.length - 1]; ctx.lineTo(last[0], last[1]);
  }
  const bandPoints = (d, w, y) => Array.from({ length: N }, (_, i) => [(i / (N - 1)) * w, y(i, d[i])]);
  const history = (store, d, rows) => {
    (store.rows ??= []).unshift(Float32Array.from(d));
    if (store.rows.length > rows) store.rows.length = rows;
    return store.rows;
  };
  const verticalGradient = (ctx, h, color, from, to) => {
    const gradient = ctx.createLinearGradient(0, h, 0, 0);
    gradient.addColorStop(0, rgba(color, from)); gradient.addColorStop(1, rgba(color, to));
    return gradient;
  };

  const styles = {
    wave: { layout: 'band', draw(ctx, w, h, d, s) {
      const points = bandPoints(d, w, (i, v) => h / 2 + Math.sin(i * 0.65 + s.t * 6) * v * h * 0.45 * s.gain);
      ctx.lineWidth = Math.max(2, h * 0.02); ctx.lineCap = ctx.lineJoin = 'round'; ctx.strokeStyle = s.color;
      ctx.beginPath(); curve(ctx, points); ctx.stroke();
    } },
    area: { layout: 'band', draw(ctx, w, h, d, s) {
      const points = bandPoints(d, w, (_, v) => h - Math.min(1, v * s.gain) * h * 0.9);
      ctx.fillStyle = verticalGradient(ctx, h, s.color, 0.95, 0.15);
      ctx.beginPath(); curve(ctx, points); ctx.lineTo(w, h); ctx.lineTo(0, h); ctx.closePath(); ctx.fill();
    } },
    lines: { layout: 'band', draw(ctx, w, h, d, s) {
      ctx.lineWidth = Math.max(1.5, h * 0.012); ctx.lineJoin = 'round';
      for (let k = 0; k < 6; k++) {
        const base = h * (0.3 + k * 0.13);
        const points = bandPoints(d, w, (i, v) => base - (d[(i + k * 5) % N] * v * h * 0.5 + v * h * 0.08) * s.gain);
        ctx.strokeStyle = rgba(s.color, 0.95 - k * 0.14);
        ctx.beginPath(); curve(ctx, points); ctx.stroke();
      }
    } },
    pulse: { layout: 'band', draw(ctx, w, h, d, s) {
      const trace = s.store.trace ??= new Float32Array(96);
      trace.copyWithin(1, 0); trace[0] = bass(d) * 0.7 + level(d) * 0.3;
      const points = Array.from(trace, (v, j) => [w - (j / (trace.length - 1)) * w, h / 2 - v * h * 0.45 * s.gain * (j % 2 ? 1 : -0.6)]);
      ctx.lineWidth = Math.max(2, h * 0.018); ctx.lineJoin = ctx.lineCap = 'round'; ctx.strokeStyle = s.color;
      ctx.beginPath(); ctx.moveTo(points[0][0], points[0][1]);
      for (const [x, y] of points) ctx.lineTo(x, y); ctx.stroke();
    } },
    dots: { layout: 'band', draw(ctx, w, h, d, s) {
      const rows = 10, slot = w / N, cell = h / rows, radius = Math.max(2, Math.min(slot, cell) * 0.32);
      for (let i = 0; i < N; i++) {
        const lit = Math.min(rows, Math.round(d[i] * s.gain * rows));
        for (let row = 0; row < lit; row++) {
          ctx.fillStyle = rgba(s.color, 0.45 + 0.55 * ((row + 1) / rows));
          ctx.beginPath(); ctx.arc(i * slot + slot / 2, h - cell * (row + 0.5), radius, 0, TAU); ctx.fill();
        }
      }
    } },
    spectrogram: { layout: 'band', draw(ctx, w, h, d, s) {
      const rows = history(s.store, d, 24), slot = w / N, cell = h / 24;
      rows.forEach((row, r) => {
        for (let i = 0; i < N; i++) {
          if (row[i] < 0.02) continue;
          ctx.fillStyle = rgba(s.color, Math.min(1, row[i] * s.gain) * (1 - r / 28));
          ctx.fillRect(i * slot + 1, h - (r + 1) * cell, slot - 2, cell - 1);
        }
      });
    } },
    radial: { layout: 'full', draw(ctx, w, h, d, s) {
      const cx = w / 2, cy = h / 2, base = Math.min(w, h), r0 = base * 0.16, reach = base * 0.3, bars = N * 2;
      ctx.lineWidth = Math.max(2, base * 0.012); ctx.lineCap = 'round'; ctx.strokeStyle = s.color;
      ctx.beginPath();
      for (let k = 0; k < bars; k++) {
        const v = d[k < N ? k : bars - 1 - k], angle = (k / bars) * TAU + s.t * 0.2;
        const c = Math.cos(angle), sn = Math.sin(angle), r1 = r0 + 4 + v * reach * s.gain;
        ctx.moveTo(cx + c * r0, cy + sn * r0); ctx.lineTo(cx + c * r1, cy + sn * r1);
      }
      ctx.stroke();
    } },
    ring: { layout: 'full', draw(ctx, w, h, d, s) {
      const base = Math.min(w, h), radius = base * (0.14 + bass(d) * 0.18 * s.gain);
      ctx.fillStyle = rgba(s.color, 0.12 + bass(d) * 0.2);
      ctx.beginPath(); ctx.arc(w / 2, h / 2, radius, 0, TAU); ctx.fill();
      ctx.lineWidth = base * (0.008 + treble(d) * 0.03 * s.gain); ctx.strokeStyle = s.color;
      ctx.beginPath(); ctx.arc(w / 2, h / 2, radius, 0, TAU); ctx.stroke();
    } },
    concentric: { layout: 'full', draw(ctx, w, h, d, s) {
      const base = Math.min(w, h) * 0.42, rings = 8;
      ctx.lineWidth = Math.max(2, base * 0.025);
      for (let j = 0; j < rings; j++) {
        const v = group(d, j, rings);
        ctx.strokeStyle = rgba(s.color, 0.25 + v * 0.75);
        ctx.beginPath(); ctx.arc(w / 2, h / 2, (base * (j + 1)) / rings * (1 + v * 0.35 * s.gain), 0, TAU); ctx.stroke();
      }
    } },
    spiral: { layout: 'full', draw(ctx, w, h, d, s) {
      const base = Math.min(w, h), steps = 220, turns = 3;
      ctx.lineWidth = Math.max(2, base * 0.01); ctx.lineJoin = ctx.lineCap = 'round'; ctx.strokeStyle = s.color;
      ctx.beginPath();
      for (let i = 0; i <= steps; i++) {
        const f = i / steps, angle = f * turns * TAU + s.t * 0.5;
        const radius = base * (0.04 + f * 0.26) + d[Math.min(N - 1, Math.floor(f * N))] * base * 0.08 * s.gain;
        const x = w / 2 + Math.cos(angle) * radius, y = h / 2 + Math.sin(angle) * radius;
        if (i) ctx.lineTo(x, y); else ctx.moveTo(x, y);
      }
      ctx.stroke();
    } },
    vinyl: { layout: 'full', draw(ctx, w, h, d, s) {
      const base = Math.min(w, h), radius = base * 0.3, spin = s.t * 1.2, points = 96;
      ctx.fillStyle = 'rgba(0,0,0,0.35)'; ctx.beginPath(); ctx.arc(w / 2, h / 2, radius, 0, TAU); ctx.fill();
      ctx.lineWidth = Math.max(1.5, base * 0.006); ctx.strokeStyle = rgba(s.color, 0.7);
      for (let g = 0; g < 4; g++) {
        ctx.beginPath();
        for (let i = 0; i <= points; i++) {
          const angle = (i / points) * TAU + spin, v = d[Math.floor((i % points) / points * N) % N];
          const r = radius * (0.5 + g * 0.16) * (1 + v * 0.08 * s.gain);
          const x = w / 2 + Math.cos(angle) * r, y = h / 2 + Math.sin(angle) * r;
          if (i) ctx.lineTo(x, y); else ctx.moveTo(x, y);
        }
        ctx.stroke();
      }
      ctx.fillStyle = s.color; ctx.beginPath(); ctx.arc(w / 2, h / 2, radius * 0.22, 0, TAU); ctx.fill();
      ctx.strokeStyle = 'rgba(0,0,0,0.6)'; ctx.lineWidth = Math.max(2, base * 0.008);
      ctx.beginPath(); ctx.moveTo(w / 2, h / 2);
      ctx.lineTo(w / 2 + Math.cos(spin) * radius * 0.2, h / 2 + Math.sin(spin) * radius * 0.2); ctx.stroke();
    } },
    particles: { layout: 'full', draw(ctx, w, h, d, s) {
      const list = s.store.list ??= [], v = level(d), size = Math.min(w, h);
      for (let n = Math.round(v * 5 * s.gain); n > 0 && list.length < 120 * s.gain; n--) {
        list.push({ x: Math.random() * w, y: h + 4, vx: (Math.random() - 0.5) * size * 0.12,
          vy: -size * (0.2 + v * 0.7 * s.gain) * (0.5 + Math.random()), life: 1, r: size * (0.004 + treble(d) * 0.012 * s.gain + Math.random() * 0.004) });
      }
      for (let i = list.length - 1; i >= 0; i--) {
        const p = list[i]; p.life -= s.dt * 0.45; p.x += p.vx * s.dt; p.y += p.vy * s.dt;
        if (p.life <= 0) { list.splice(i, 1); continue; }
        ctx.fillStyle = rgba(s.color, p.life); ctx.beginPath(); ctx.arc(p.x, p.y, p.r, 0, TAU); ctx.fill();
      }
    } },
    bubbles: { layout: 'full', draw(ctx, w, h, d, s) {
      const list = s.store.list ??= [], v = level(d), size = Math.min(w, h);
      if (list.length < 40 && Math.random() < v * 0.8 * s.gain) {
        const i = Math.floor(Math.random() * N);
        list.push({ x: Math.random() * w, y: h + size * 0.05, r: size * (0.015 + d[i] * 0.05 * s.gain), speed: size * (0.08 + Math.random() * 0.12), phase: Math.random() * TAU, life: 1 });
      }
      ctx.lineWidth = Math.max(1.5, size * 0.004);
      for (let i = list.length - 1; i >= 0; i--) {
        const p = list[i]; p.y -= p.speed * s.dt; p.life -= s.dt * 0.25;
        if (p.life <= 0 || p.y < -p.r) { list.splice(i, 1); continue; }
        const x = p.x + Math.sin(s.t * 1.5 + p.phase) * size * 0.02;
        ctx.fillStyle = rgba(s.color, p.life * 0.18); ctx.strokeStyle = rgba(s.color, p.life * 0.8);
        ctx.beginPath(); ctx.arc(x, p.y, p.r, 0, TAU); ctx.fill(); ctx.stroke();
      }
    } },
    fireflies: { layout: 'full', draw(ctx, w, h, d, s) {
      const list = s.store.list ??= Array.from({ length: 40 }, (_, i) => ({ x: Math.random(), y: Math.random(), phase: Math.random() * TAU, band: i % N }));
      const size = Math.min(w, h);
      for (const p of list) {
        const v = d[p.band], x = (p.x + Math.sin(s.t * 0.3 + p.phase) * 0.03) * w, y = (p.y + Math.cos(s.t * 0.25 + p.phase) * 0.03) * h;
        const r = size * (0.004 + v * 0.016 * s.gain);
        ctx.fillStyle = rgba(s.color, 0.12 + v * 0.2); ctx.beginPath(); ctx.arc(x, y, r * 3, 0, TAU); ctx.fill();
        ctx.fillStyle = rgba(s.color, 0.35 + v * 0.65); ctx.beginPath(); ctx.arc(x, y, r, 0, TAU); ctx.fill();
      }
    } },
    glow: { layout: 'full', draw(ctx, w, h, d, s) {
      const gradient = ctx.createRadialGradient(w / 2, h / 2, Math.min(w, h) * 0.25, w / 2, h / 2, Math.max(w, h) * 0.75);
      gradient.addColorStop(0, rgba(s.color, 0)); gradient.addColorStop(1, rgba(s.color, Math.min(1, 0.15 + level(d) * 0.85 * s.gain)));
      ctx.fillStyle = gradient; ctx.fillRect(0, 0, w, h);
    } },
    orbs: { layout: 'full', draw(ctx, w, h, d, s) {
      const count = 5, size = Math.min(w, h);
      for (let k = 0; k < count; k++) {
        const v = group(d, k, count), x = w * (0.5 + 0.32 * Math.sin(s.t * (0.21 + k * 0.07) + k * 1.7));
        const y = h * (0.5 + 0.3 * Math.cos(s.t * (0.17 + k * 0.05) + k * 2.3)), r = size * (0.12 + v * 0.28 * s.gain);
        const gradient = ctx.createRadialGradient(x, y, 0, x, y, r);
        gradient.addColorStop(0, rgba(s.color, 0.35 + v * 0.55)); gradient.addColorStop(1, rgba(s.color, 0));
        ctx.fillStyle = gradient; ctx.beginPath(); ctx.arc(x, y, r, 0, TAU); ctx.fill();
      }
    } },
    aurora: { layout: 'full', draw(ctx, w, h, d, s) {
      const ribbons = 4, steps = 48;
      for (let k = 0; k < ribbons; k++) {
        const v = group(d, k, ribbons), top = [], bottom = [], thickness = h * (0.08 + v * 0.18 * s.gain);
        for (let i = 0; i <= steps; i++) {
          const x = (i / steps) * w, y = h * (0.25 + k * 0.1) + Math.sin((i / steps) * (3 + k) + s.t * (0.5 + k * 0.15) + k) * h * (0.04 + v * 0.1 * s.gain);
          top.push([x, y]); bottom.push([x, y + thickness]);
        }
        const gradient = ctx.createLinearGradient(0, top[0][1], 0, top[0][1] + thickness * 1.6);
        gradient.addColorStop(0, rgba(s.color, 0.1 + v * 0.5)); gradient.addColorStop(1, rgba(s.color, 0));
        ctx.fillStyle = gradient; ctx.beginPath();
        top.forEach(([x, y], i) => i ? ctx.lineTo(x, y) : ctx.moveTo(x, y));
        for (let i = bottom.length - 1; i >= 0; i--) ctx.lineTo(bottom[i][0], bottom[i][1]);
        ctx.closePath(); ctx.fill();
      }
    } },
    polygon: { layout: 'full', draw(ctx, w, h, d, s) {
      const sides = 6, base = Math.min(w, h), layers = 3;
      ctx.lineWidth = Math.max(2, base * 0.01); ctx.lineJoin = 'round';
      for (let l = 0; l < layers; l++) {
        const spin = s.t * (0.25 + l * 0.12) * (l % 2 ? -1 : 1);
        ctx.strokeStyle = rgba(s.color, 0.95 - l * 0.25); ctx.beginPath();
        for (let v = 0; v <= sides; v++) {
          const r = base * (0.1 + l * 0.08) * (1 + d[((v % sides) * 5 + l * 2) % N] * 1.1 * s.gain);
          const angle = ((v % sides) / sides) * TAU + spin;
          const x = w / 2 + Math.cos(angle) * r, y = h / 2 + Math.sin(angle) * r;
          if (v) ctx.lineTo(x, y); else ctx.moveTo(x, y);
        }
        ctx.stroke();
      }
    } },
    terrain: { layout: 'full', draw(ctx, w, h, d, s) {
      const depth = 14, rows = history(s.store, d, depth);
      ctx.lineWidth = Math.max(1.5, h * 0.003); ctx.lineJoin = 'round';
      for (let r = rows.length - 1; r >= 0; r--) {
        const z = r / depth, spread = w * (0.55 - z * 0.25), baseY = h * (0.95 - z * 0.45), lift = h * 0.28 * s.gain * (1 - z * 0.6);
        const points = Array.from({ length: N }, (_, i) => [w / 2 - spread + (i / (N - 1)) * spread * 2, baseY - rows[r][i] * lift]);
        ctx.strokeStyle = rgba(s.color, 0.9 - z * 0.7);
        ctx.beginPath(); curve(ctx, points); ctx.stroke();
      }
    } }
  };
  window.__sparkAudioStyles = styles;
})();
