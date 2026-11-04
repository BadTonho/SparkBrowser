const GAP = 8;
const EDGE = 8;

/** Positions a permission card below the address bar's site indicator. */
export function permissionPopover(node: HTMLElement) {
  let resizeObserver: ResizeObserver | undefined;

  const position = () => {
    const anchor = document.querySelector<HTMLElement>(".omnibox-icon");
    const rect = anchor?.getBoundingClientRect();
    if (!rect) return;

    const width = Math.min(360, window.innerWidth - EDGE * 2);
    const left = Math.max(EDGE, Math.min(rect.left, window.innerWidth - width - EDGE));
    const top = Math.min(rect.bottom + GAP, Math.max(EDGE, window.innerHeight - EDGE - 120));
    node.style.setProperty("--permission-popover-left", `${left}px`);
    node.style.setProperty("--permission-popover-top", `${top}px`);
    node.style.setProperty("--permission-popover-max-height", `${Math.max(120, window.innerHeight - top - EDGE)}px`);
  };

  position();
  window.addEventListener("resize", position);
  window.addEventListener("scroll", position, true);
  resizeObserver = new ResizeObserver(position);
  resizeObserver.observe(node);
  const anchor = document.querySelector<HTMLElement>(".omnibox-icon");
  if (anchor) resizeObserver.observe(anchor);

  return {
    destroy() {
      window.removeEventListener("resize", position);
      window.removeEventListener("scroll", position, true);
      resizeObserver?.disconnect();
    },
  };
}
