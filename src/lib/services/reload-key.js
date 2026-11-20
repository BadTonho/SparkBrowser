/**
 * Keys that reload a page in every browser besides the configurable "Reload Active Tab" shortcut:
 * F5 (alone, with Ctrl or with Shift) and Ctrl+Shift+R.
 * @param {Pick<KeyboardEvent, "key" | "ctrlKey" | "shiftKey" | "altKey" | "metaKey">} event
 */
export function isReloadKey(event) {
  if (event.altKey || event.metaKey) return false;
  if (event.key === "F5") return true;
  return event.ctrlKey && event.shiftKey && (event.key === "R" || event.key === "r");
}
