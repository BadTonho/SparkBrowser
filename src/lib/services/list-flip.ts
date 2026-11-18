/**
 * Slides the items of a list to their new places when only their order changed, so a reordered tab
 * moves instead of jumping. Adding or removing items is left to the list's own transitions.
 *
 * Call `measure()` before the DOM updates and `play()` after it.
 */
export interface ListFlip { measure(): void; play(): void }

export function createListFlip(list: () => HTMLElement | undefined, itemSelector: string, key = "tabId"): ListFlip {
  let before = new Map<string, DOMRect>();
  const read = () => {
    const found = new Map<string, DOMRect>();
    for (const element of list()?.querySelectorAll<HTMLElement>(itemSelector) ?? []) {
      const id = element.dataset[key];
      if (id) found.set(id, element.getBoundingClientRect());
    }
    return found;
  };
  return {
    measure() { before = read(); },
    play() {
      const reduced = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
      const root = list();
      if (!root || reduced || before.size === 0) { before = new Map(); return; }
      const after = read();
      const sameItems = after.size === before.size && [...after.keys()].every(id => before.has(id));
      if (sameItems) {
        for (const element of root.querySelectorAll<HTMLElement>(itemSelector)) {
          const id = element.dataset[key], from = id ? before.get(id) : undefined, to = id ? after.get(id) : undefined;
          if (!from || !to) continue;
          const dx = from.left - to.left, dy = from.top - to.top;
          if (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5) continue;
          element.animate(
            [{ transform: `translate(${dx}px, ${dy}px)` }, { transform: "translate(0, 0)" }],
            { duration: 220, easing: "cubic-bezier(0.22, 1, 0.36, 1)" },
          );
        }
      }
      before = new Map();
    },
  };
}
