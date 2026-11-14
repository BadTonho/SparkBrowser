import { preparePanel, type PanelName } from "../services/bridge/panels";

/** Panels after the first are only needed on a later click, so they must not compete with the one being opened now. */
const FOLLOW_UP_DELAY_MS = 800;
const EVENTS = ["pointerenter", "focus", "pointerdown"] as const;

/**
 * Prepares a panel as soon as the pointer reaches its button, it receives focus or is pressed, which is
 * usually before the click completes. The first panel listed is prepared at once; any others (for example
 * the full settings dialog behind the quick settings menu) follow after a delay, because creating two web
 * views together slows down the one the user is waiting for.
 */
export function prepares(node: HTMLElement, panels: PanelName | PanelName[]) {
  let list = Array.isArray(panels) ? panels : [panels];
  const timers = new Set<ReturnType<typeof setTimeout>>();
  const prepare = () => {
    const [first, ...followUps] = list;
    if (first) preparePanel(first);
    for (const panel of followUps) {
      const timer = setTimeout(() => {
        timers.delete(timer);
        preparePanel(panel);
      }, FOLLOW_UP_DELAY_MS);
      timers.add(timer);
    }
  };
  for (const type of EVENTS) node.addEventListener(type, prepare, { passive: true });
  return {
    update(next: PanelName | PanelName[]) { list = Array.isArray(next) ? next : [next]; },
    destroy() {
      for (const type of EVENTS) node.removeEventListener(type, prepare);
      timers.forEach(clearTimeout);
    },
  };
}
