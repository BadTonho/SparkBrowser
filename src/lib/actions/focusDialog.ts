type Options = { onEscape: () => unknown; ignoreEscape?: () => boolean };
const dialogs: HTMLElement[] = [];
const selector = 'input:not([disabled]), select:not([disabled]), textarea:not([disabled]), button:not([disabled]), summary, a[href], [tabindex="0"]';

function visibleControl(element: HTMLElement) {
  if (!element.getClientRects().length || element.closest('[inert], [hidden]')) return false;
  // Closed details can retain layout rectangles for their undisplayed content.
  // Its summary remains keyboard accessible, including with nested disclosures.
  for (let details = element.closest('details:not([open])'); details; details = details.parentElement?.closest('details:not([open])') ?? null) {
    if (!details.querySelector(':scope > summary')?.contains(element)) return false;
  }
  return true;
}

/** Shared focus lifecycle, including stacked dialogs and dynamically disabled controls. */
export function focusDialog(node: HTMLElement, options: Options) {
  const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  dialogs.push(node);
  const isTop = () => dialogs.at(-1) === node;
  const controls = () => [...node.querySelectorAll<HTMLElement>(selector)]
    .filter(visibleControl);
  const focusFirst = () => (controls()[0] ?? node).focus({ preventScroll: true });
  queueMicrotask(() => { if (isTop() && node.isConnected) focusFirst(); });
  function keydown(event: KeyboardEvent) {
    if (!isTop()) return;
    if (event.key === 'Escape' && !options.ignoreEscape?.()) {
      event.preventDefault();
      event.stopImmediatePropagation();
      void options.onEscape();
    } else if (event.key === 'Tab') {
      const items = controls();
      const index = items.indexOf(document.activeElement as HTMLElement);
      if (!items.length || (event.shiftKey ? index <= 0 : index < 0 || index === items.length - 1)) {
        event.preventDefault();
        (event.shiftKey ? items.at(-1) : items[0])?.focus();
      }
    }
  }
  function focusin(event: FocusEvent) {
    if (isTop() && event.target instanceof Node && !node.contains(event.target)) focusFirst();
  }
  document.addEventListener('keydown', keydown, true);
  document.addEventListener('focusin', focusin);
  return {
    update(next: Options) { options = next; },
    destroy() {
      const wasTop = isTop();
      dialogs.splice(dialogs.indexOf(node), 1);
      document.removeEventListener('keydown', keydown, true);
      document.removeEventListener('focusin', focusin);
      if (wasTop && previous?.isConnected) queueMicrotask(() => previous.focus({ preventScroll: true }));
    },
  };
}
