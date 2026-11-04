/** The native surface receives presentation updates, so it owns its DOM focus. */
export function syncDownloadFocus(root: HTMLElement | undefined, opening: boolean, hadForm: boolean) {
  if (!root) return;
  const input = root.querySelector<HTMLElement>('[data-testid="download-url"]');
  if (opening || !!input !== hadForm)
    (input ?? root.querySelector<HTMLElement>('[data-testid="download-add-toggle"]'))?.focus({ preventScroll: true });
}
