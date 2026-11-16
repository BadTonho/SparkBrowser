/**
 * Scrolls the first child from side to side when it is wider than its container (the animation lives in
 * the component's CSS and reads `--shift` and `--duration`); otherwise the text stays still.
 */
export function marquee(node: HTMLElement, _text?: string) {
  const fit = () => {
    const text = node.firstElementChild as HTMLElement | null;
    if (!text) return;
    const overflow = Math.ceil(text.offsetWidth - node.clientWidth);
    if (overflow > 2) {
      node.style.setProperty("--shift", `-${overflow}px`);
      node.style.setProperty("--duration", `${Math.max(8, overflow / 12 + 5)}s`);
      node.dataset.scrolling = "true";
    } else {
      delete node.dataset.scrolling;
    }
  };
  const observer = typeof ResizeObserver === "function" ? new ResizeObserver(fit) : null;
  observer?.observe(node);
  requestAnimationFrame(fit);
  return {
    update() { requestAnimationFrame(fit); },
    destroy() { observer?.disconnect(); },
  };
}
