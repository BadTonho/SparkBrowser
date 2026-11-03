import type { TitlebarAction } from "../stores/titlebarActionsStore";

interface Point { pointerId: number; clientX: number; clientY: number }
interface Bounds { left: number; right: number; top: number; bottom: number }
export interface ActionRect extends Bounds { id: TitlebarAction }
export interface ActionDragPreview {
  id: TitlebarAction;
  offset: number;
  before: TitlebarAction | null;
  marker: number | null;
}
interface Options {
  items: () => ActionRect[];
  bounds: () => Bounds;
  capture: (pointerId: number) => void;
  release: (pointerId: number) => void;
  preview: (value: ActionDragPreview | null) => void;
  commit: (id: TitlebarAction, before: TitlebarAction | null, visible: TitlebarAction[]) => void;
}

/** Owns the hold lifecycle; the UI keeps its DOM order stable until drop. */
export function createActionDrag(options: Options) {
  let session: { id: TitlebarAction; start: Point; active: boolean; blocked: boolean; items: ActionRect[] } | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let suppressClick = false;
  const inside = (p: Point) => {
    const b = options.bounds();
    return p.clientX >= b.left && p.clientX <= b.right && p.clientY >= b.top && p.clientY <= b.bottom;
  };
  const target = (p: Point) => session!.items.filter(item => item.id !== session!.id)
    .find(item => p.clientX < (item.left + item.right) / 2) ?? null;
  function cancel() {
    clearTimeout(timer);
    const previous = session;
    session = null;
    if (previous) {
      suppressClick = true;
      options.release(previous.start.pointerId);
    }
    options.preview(null);
  }
  return {
    isTracking: () => session !== null,
    down(id: TitlebarAction, event: Point & { button: number; isPrimary: boolean }) {
      if (event.button !== 0 || !event.isPrimary || session) return;
      suppressClick = false;
      session = { id, start: { pointerId: event.pointerId, clientX: event.clientX, clientY: event.clientY }, active: false, blocked: false, items: options.items() };
      options.capture(event.pointerId);
      timer = setTimeout(() => {
        if (!session || session.blocked) return;
        session.active = true;
        suppressClick = true;
        options.preview({ id, offset: 0, before: null, marker: null });
      }, 500);
    },
    move(event: Point) {
      if (!session || event.pointerId !== session.start.pointerId || session.blocked) return;
      if (!session.active) {
        if (Math.hypot(event.clientX - session.start.clientX, event.clientY - session.start.clientY) > 6) {
          clearTimeout(timer);
          session.blocked = true;
          suppressClick = true;
        }
        return;
      }
      const before = target(event);
      const last = session.items.filter(item => item.id !== session!.id).at(-1);
      options.preview({ id: session.id, offset: event.clientX - session.start.clientX,
        before: before?.id ?? null, marker: inside(event) ? before?.left ?? last?.right ?? null : null });
    },
    up(event: Point) {
      if (!session || event.pointerId !== session.start.pointerId) return;
      const previous = session;
      clearTimeout(timer);
      if (previous.active && inside(event)) {
        // A stationary long press consumes the click without changing the order.
        if (Math.abs(event.clientX - previous.start.clientX) > 6)
          options.commit(previous.id, target(event)?.id ?? null, previous.items.map(item => item.id));
      }
      session = null;
      options.preview(null);
      options.release(event.pointerId);
    },
    cancel,
    consumeClick(event: { detail: number }) { return event.detail !== 0 && suppressClick; },
  };
}
