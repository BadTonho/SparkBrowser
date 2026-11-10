/** The native surface receives presentation updates, so it owns its DOM focus. */
const pendingFocus = new WeakSet<HTMLElement>();
const previousForm = new WeakMap<HTMLElement, boolean>();
export function syncDownloadFocus(root: HTMLElement | undefined, opening: boolean, hadForm: boolean) {
  if (!root) return;
  const input = root.querySelector<HTMLElement>('[data-testid="download-url"]');
  if (opening || !!input !== (previousForm.get(root) ?? hadForm)) pendingFocus.add(root);
  previousForm.set(root, !!input);
  const target = input ?? root.querySelector<HTMLElement>('[data-testid="download-add-toggle"]');
  // Submission can collapse the form before the busy button becomes enabled.
  // Keep the focus request until the next confirmed presentation enables it.
  if (pendingFocus.has(root) && target && !(target as HTMLButtonElement).disabled) {
    target.focus({ preventScroll: true });
    pendingFocus.delete(root);
  }
}
