let latestRevision = 0;
window.setDragPreview = ({ revision, label, locale, pointer, title, host, pane }) => {
  // Initial document loading may delay an earlier paint beyond a newer one.
  if (revision < latestRevision) return;
  latestRevision = revision;
  document.documentElement.lang = locale;
  document.getElementById('split-drop-label').textContent = label;
  const target = document.getElementById('split-drop-preview');
  target.hidden = !pane;
  if (pane) Object.assign(target.style, {
    left: `${pane.x + 4}px`, top: `${pane.y + 4}px`,
    width: `${Math.max(0, pane.width - 8)}px`, height: `${Math.max(0, pane.height - 8)}px`,
  });
  document.getElementById('dragged-tab-title').textContent = title;
  document.getElementById('dragged-tab-host').textContent = host;
  const ghost = document.getElementById('dragged-tab');
  const { width, height } = ghost.getBoundingClientRect();
  const x = pointer.x + width + 24 > innerWidth ? pointer.x - width - 16 : pointer.x + 16;
  const y = pointer.y + height + 26 > innerHeight ? pointer.y - height - 18 : pointer.y + 18;
  ghost.style.left = `${Math.max(8, Math.min(x, innerWidth - width - 8))}px`;
  ghost.style.top = `${Math.max(8, Math.min(y, innerHeight - height - 8))}px`;
};
