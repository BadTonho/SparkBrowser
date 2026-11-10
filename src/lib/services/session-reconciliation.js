/** Organizational snapshots must not roll back newer observations of a document.
 * @param {import('../types/browser').TabItem[]} previous
 * @param {import('../types/browser').TabItem[]} incoming
 */
export function mergeSessionObservations(previous, incoming) {
  const byId = new Map(previous.map(tab => [tab.id, tab]));
  return incoming.map(tab => {
    const old = byId.get(tab.id);
    return old && (old.navigation?.revision ?? 0) > (tab.navigation?.revision ?? 0)
      ? { ...tab, url: old.url, title: old.title, navigation: old.navigation, activity: old.activity }
      : tab;
  });
}

/** Navigation reports may update document observations, never tab organization.
 * @param {import('../types/browser').TabItem[]} items
 * @param {import('../types/browser').NavigationPatch} patch
 */
export function mergeNavigationObservation(items, patch) {
  return items.map(tab => tab.id === patch.id &&
    (tab.navigation?.revision ?? 0) < patch.navigation.revision
    ? { ...tab, url: patch.url, title: patch.title, navigation: patch.navigation,
      ...(patch.activity ? { activity: patch.activity } : {}) }
    : tab);
}
