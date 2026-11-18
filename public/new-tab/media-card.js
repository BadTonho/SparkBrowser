// Helpers shared by the Spotify and YouTube now-playing cards.
(() => {
  const clock = seconds => {
    const total = Math.max(0, Math.floor(seconds)), h = Math.floor(total / 3600), m = Math.floor(total / 60) % 60, s = total % 60;
    return h ? `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}` : `${m}:${String(s).padStart(2, '0')}`;
  };
  // A title wider than its box scrolls from side to side (see media-card.css) instead of being cut off.
  function fitTitle(box, span) {
    const overflow = Math.ceil(span.offsetWidth - box.clientWidth);
    if (overflow > 2) {
      box.style?.setProperty('--shift', `-${overflow}px`);
      box.style?.setProperty('--duration', `${Math.max(8, overflow / 12 + 5)}s`);
      box.setAttribute('data-scrolling', 'true');
    } else {
      box.removeAttribute('data-scrolling');
    }
  }
  // Display choices are per-device conveniences, like the clock's; storage may be unavailable.
  const choice = (key, allowed) => { try { const value = localStorage.getItem(key); return allowed.includes(value) ? value : allowed[0]; } catch { return allowed[0]; } };
  const save = (key, allowed, value) => { try { localStorage.setItem(key, allowed.includes(value) ? value : allowed[0]); } catch (error) { window.__sparkLogIgnored?.(error); } };
  const later = fn => (window.requestAnimationFrame ?? (callback => callback()))(fn);
  window.__sparkMediaCard = { clock, fitTitle, choice, save, later };
})();
