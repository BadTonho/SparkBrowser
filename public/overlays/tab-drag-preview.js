let latestRevision = 0;
window.setDragPreview = ({ revision, label, locale }) => {
  // Initial document loading may delay an earlier paint beyond a newer one.
  if (revision < latestRevision) return;
  latestRevision = revision;
  document.documentElement.lang = locale;
  document.getElementById('split-drop-label').textContent = label;
};
