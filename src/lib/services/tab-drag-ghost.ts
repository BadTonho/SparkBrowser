/**
 * The copy of a tab that follows the pointer while the tab is dragged. It is a clone of the real tab
 * element, so it looks exactly like the tab (icon, title, colors), and it keeps the point of the tab
 * the user grabbed under the pointer.
 */
export interface TabGhost {
  move(x: number, y: number): void;
  /** Hidden while the pointer is over the page, where the native preview takes over. */
  setHidden(hidden: boolean): void;
  destroy(): void;
}

export function createTabGhost(source: HTMLElement, container: HTMLElement, pointerX: number, pointerY: number): TabGhost {
  const rect = source.getBoundingClientRect();
  const grabX = pointerX - rect.left;
  const grabY = pointerY - rect.top;
  const node = source.cloneNode(true) as HTMLElement;
  // The copy must never be found as a tab, a target or a form control.
  for (const element of [node, ...node.querySelectorAll<HTMLElement>("[id],[data-tab-id],[tabindex]")]) {
    element.removeAttribute("id");
    element.removeAttribute("data-tab-id");
    element.removeAttribute("tabindex");
  }
  node.classList.remove("is-dragging");
  node.classList.add("tab-drag-ghost");
  node.setAttribute("aria-hidden", "true");
  node.removeAttribute("title");
  Object.assign(node.style, {
    position: "fixed", left: "0", top: "0", margin: "0",
    width: `${rect.width}px`, height: `${rect.height}px`, minWidth: "0", maxWidth: "none", flex: "none",
    pointerEvents: "none", zIndex: "2147483000", transition: "none", willChange: "transform",
    transform: `translate3d(${rect.left}px, ${rect.top}px, 0)`,
  });
  container.appendChild(node);
  let x = rect.left, y = rect.top;
  const draw = () => { node.style.transform = `translate3d(${Math.round(x)}px, ${Math.round(y)}px, 0) scale(1.04)`; };
  draw();
  return {
    move(pointerX, pointerY) { x = pointerX - grabX; y = pointerY - grabY; draw(); },
    setHidden(hidden) { node.style.visibility = hidden ? "hidden" : "visible"; },
    destroy() { node.remove(); },
  };
}
