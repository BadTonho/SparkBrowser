// Draws the hover card of a tab. Spark sizes and places the window around the card, and sends every
// paint with an increasing revision.
let latestRevision = 0;

window.setTabHoverCard = ({ revision, locale, title, memory, noPreview, picture }) => {
  // Initial document loading may delay an earlier paint beyond a newer one.
  if (revision < latestRevision) return;
  latestRevision = revision;
  document.documentElement.lang = locale;
  document.getElementById('tab-hover-title').textContent = title;
  document.getElementById('tab-hover-memory').textContent = memory;
  const image = document.getElementById('tab-hover-image');
  const empty = document.getElementById('tab-hover-empty');
  // Only pictures Spark took itself are shown.
  const usable = typeof picture === 'string' && picture.startsWith('data:image/');
  if (usable) { if (image.src !== picture) image.src = picture; } else image.removeAttribute('src');
  image.hidden = !usable;
  empty.hidden = usable;
  empty.textContent = usable ? '' : noPreview;
  document.getElementById('tab-hover-card').hidden = false;
};

// Cleared before the window is hidden, so showing it again never flashes the previous card.
window.hideTabHoverCard = () => {
  document.getElementById('tab-hover-card').hidden = true;
};
