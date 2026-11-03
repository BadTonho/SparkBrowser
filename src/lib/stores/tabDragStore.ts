import { get, writable } from "svelte/store";
import { tabs, beginTabReorder, previewTabReorder, finishTabReorder, applyTabsSnapshot, selectTab, changeTabWorkspace } from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { splitLayout, splitKey, splitGroupsKey } from "./splitViewStore";
import { replaceGroupOrder } from "../services/split-tab-groups.js";
import { tabLayout } from "./settingsStore";
import { locale, tr } from "../i18n";
import { onOverlayOpening } from "../services/bridge/overlays";
import { dockTabToSplit, moveTabInSplit, setDragPreview, flushDragPreview, type DockSide, type DockContext } from "../services/bridge/tab-drag";
import { crossedDragThreshold, splitDropSide } from "../services/tab-drag-geometry.js";
import { mediaPopover, closeMedia } from "./mediaPopoverStore";
import { privacyPopover, closePrivacy } from "./privacyPopoverStore";
import { zoomPopover, closeZoom } from "./zoomPopoverStore";
import { isTauriRuntime } from "../services/bridge/runtime";

interface Drag {
  id: string; pointerId: number; root: HTMLElement; startX: number; startY: number;
  x: number; y: number;
  moved: boolean; side: DockSide | null; context: DockContext; layout: string;
  cursor: string; userSelect: string; original: import("../types/browser").TabItem[];
}
export const tabDrag = writable<{ id: string; dragging: boolean; side: DockSide | null } | null>(null);
let drag: Drag | null = null, busy = false, frame = 0, lastPreview = "";

function publish() { tabDrag.set(drag ? { id: drag.id, dragging: drag.moved, side: drag.side } : null); }
function preview(side: DockSide | null) {
  if (!drag) return;
  const gesture = drag;
  const bounds = document.querySelector('.viewport-frame')?.getBoundingClientRect();
  const pointer = { x: Math.round(drag.x), y: Math.round(drag.y) };
  const signature = JSON.stringify([side, pointer, bounds?.x, bounds?.y, bounds?.width, bounds?.height, get(splitLayout)?.ratio, get(locale)]);
  if (signature === lastPreview) return;
  lastPreview = signature;
  void setDragPreview({ tabId: drag.id, side, context: drag.context, pointer,
    label: side ? tr(side === "left" ? "tabs.split_drop_left" : "tabs.split_drop_right") : '', locale: get(locale) })
    .catch(error => { if (drag !== gesture) return; cancelTabDrag(); void Promise.reject(error); });
}
function contextValid() {
  if (!drag) return false;
  const current = get(splitLayout);
  return get(activeWorkspaceId) === drag.context.workspaceId && get(tabs).find(tab => tab.is_active)?.id === drag.context.activeTabId
    && get(tabs).some(tab => tab.id === drag!.id && tab.workspace_id === drag!.context.workspaceId)
    && (current ? splitKey(current) : null) === drag.context.splitKey
    && (current ? splitGroupsKey(current) : null) === drag.context.groupsKey && get(tabLayout) === drag.layout;
}
function cleanup(keepDockPreview = false) {
  const previous = drag;
  drag = null; lastPreview = ""; cancelAnimationFrame(frame); frame = 0;
  window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', release);
  window.removeEventListener('pointercancel', cancelTabDrag); window.removeEventListener('blur', cancelTabDrag);
  window.removeEventListener('keydown', key, true);
  previous?.root.removeEventListener('lostpointercapture', cancelTabDrag);
  if (previous?.root.hasPointerCapture(previous.pointerId)) previous.root.releasePointerCapture(previous.pointerId);
  document.body.style.cursor = previous?.cursor ?? ''; document.body.style.userSelect = previous?.userSelect ?? '';
  document.querySelectorAll('.drop-hover').forEach(element => element.classList.remove('drop-hover'));
  tabDrag.set(null);
  const hidden = keepDockPreview ? flushDragPreview() : setDragPreview(null);
  void hidden.catch(() => {});
  return previous ? { ...previous, hidden } : null;
}
export function cancelTabDrag() {
  if (!drag) return;
  cleanup(); void finishTabReorder(true);
}
function key(event: KeyboardEvent) {
  if (event.key === 'Escape') { event.preventDefault(); event.stopImmediatePropagation(); cancelTabDrag(); }
}
function updateDestination() {
  if (!drag) return;
  const bounds = document.querySelector('.viewport-frame')?.getBoundingClientRect();
  const eligible = drag.original.filter(tab => tab.workspace_id === drag!.context.workspaceId).length > 1;
  const side = bounds && eligible ? splitDropSide(bounds, drag.x, drag.y) : null;
  if (drag.side !== side) { drag.side = side; publish(); }
  preview(side);
}
function move(event: PointerEvent) {
  if (!drag || event.pointerId !== drag.pointerId) return;
  if (!contextValid() || event.clientX < 0 || event.clientY < 0 || event.clientX > innerWidth || event.clientY > innerHeight) { cancelTabDrag(); return; }
  if (!drag.moved && crossedDragThreshold(drag.startX, drag.startY, event.clientX, event.clientY)) {
    drag.moved = true; document.body.style.cursor = 'grabbing'; document.body.style.userSelect = 'none';
    void closeMedia(); void closePrivacy(); void closeZoom();
  }
  if (!drag.moved) return;
  drag.x = event.clientX; drag.y = event.clientY;
  updateDestination();
  const target = document.elementFromPoint(event.clientX, event.clientY);
  const pill = target?.closest<HTMLElement>('.ws-pill');
  document.querySelectorAll('.ws-pill').forEach(element => element.classList.toggle('drop-hover', element === pill));
  const over = target?.closest<HTMLElement>('.h-tab,.tab-row')?.dataset.tabId;
  if (!drag.side && over && over !== drag.id) {
    const split = get(splitLayout);
    const membership = split && drag.layout === 'horizontal' ? (split.left_tab_ids.includes(drag.id) ? split.left_tab_ids : split.right_tab_ids) : null;
    if (membership && !membership.includes(over)) return;
    const ids = get(tabs).filter(tab => tab.workspace_id === drag!.context.workspaceId && (!membership || membership.includes(tab.id))).map(tab => tab.id);
    const from = ids.indexOf(drag.id), to = ids.indexOf(over);
    if (from >= 0 && to >= 0) { ids.splice(to, 0, ids.splice(from, 1)[0]); previewTabReorder(membership ? replaceGroupOrder(get(tabs), ids) : ids); }
  }
  publish();
}
async function release(event: PointerEvent) {
  if (!drag || event.pointerId !== drag.pointerId) return;
  if (!contextValid() || event.clientX < 0 || event.clientY < 0 || event.clientX > innerWidth || event.clientY > innerHeight) { cancelTabDrag(); return; }
  // Re-evaluate docking at release without repeating the last provisional reorder.
  if (!drag.moved) move(event);
  else { drag.x = event.clientX; drag.y = event.clientY; updateDestination(); }
  if (!drag) return;
  const target = document.elementFromPoint(event.clientX, event.clientY);
  const targetWorkspace = target?.closest<HTMLElement>('.ws-pill')?.dataset.wsId;
  const targetTabList = target?.closest('.tabs-scroll,.tabs-list');
  const targetSide = (targetTabList as HTMLElement | undefined)?.dataset.paneSide as DockSide | undefined;
  const targetTab = target?.closest<HTMLElement>('.h-tab')?.dataset.tabId;
  const displayed = targetSide && get(splitLayout) ? get(tabs).filter(tab => get(splitLayout)![targetSide === 'left' ? 'left_tab_ids' : 'right_tab_ids'].includes(tab.id)).map(tab => tab.id) : [];
  const beforeId = targetTab && targetTab !== drag.id ? targetTab : displayed.includes(drag.id) ? displayed[displayed.indexOf(drag.id) + 1] ?? null : null;
  const docking = drag.moved && drag.side !== null;
  const ended = cleanup(docking);
  if (!ended) return;
  busy = true;
  try {
    await ended.hidden;
    if (ended.moved && ended.side) {
      await finishTabReorder(true);
      if (isTauriRuntime()) applyTabsSnapshot(await dockTabToSplit(ended.id, ended.side, ended.context));
    } else if (ended.moved && targetSide && ended.context.splitKey) {
      await finishTabReorder(true);
      if (isTauriRuntime()) applyTabsSnapshot(await moveTabInSplit(ended.id, targetSide, beforeId, ended.context));
    } else if (ended.moved && targetWorkspace) {
      await finishTabReorder(true); await changeTabWorkspace(ended.id, targetWorkspace);
    } else {
      await finishTabReorder(ended.moved && !targetTabList);
      if (!ended.moved) await selectTab(ended.id);
    }
  } catch (error) {
    await finishTabReorder(true);
    throw error;
  } finally {
    try { if (docking) await setDragPreview(null); }
    finally { busy = false; }
  }
}
export function startTabDrag(event: PointerEvent, id: string) {
  if (busy || drag || event.button !== 0 || !event.isPrimary || (event.target as HTMLElement).closest('button,input,a')) return;
  const active = get(tabs).find(tab => tab.is_active), root = document.querySelector<HTMLElement>('.browser-root');
  if (!active || !root) return;
  const split = get(splitLayout);
  event.preventDefault();
  drag = { id, pointerId: event.pointerId, root, startX: event.clientX, startY: event.clientY, x: event.clientX, y: event.clientY, moved: false, side: null,
    context: { workspaceId: get(activeWorkspaceId), activeTabId: active.id, splitKey: split ? splitKey(split) : null, groupsKey: split ? splitGroupsKey(split) : null }, layout: get(tabLayout),
    cursor: document.body.style.cursor, userSelect: document.body.style.userSelect, original: get(tabs) };
  beginTabReorder();
  try { root.setPointerCapture(event.pointerId); }
  catch (error) { cancelTabDrag(); throw error; }
  publish();
  root.addEventListener('lostpointercapture', cancelTabDrag);
  window.addEventListener('pointermove', move); window.addEventListener('pointerup', release);
  window.addEventListener('pointercancel', cancelTabDrag); window.addEventListener('blur', cancelTabDrag); window.addEventListener('keydown', key, true);
  const follow = () => { if (!drag) return; if (!contextValid()) { cancelTabDrag(); return; } if (drag.moved) updateDestination(); frame = requestAnimationFrame(follow); };
  frame = requestAnimationFrame(follow);
}
export function initializeTabDrag() {
  const stopOverlay = onOverlayOpening(cancelTabDrag);
  const stopMedia = mediaPopover.subscribe(value => { if (value) cancelTabDrag(); });
  const stopPrivacy = privacyPopover.subscribe(value => { if (value) cancelTabDrag(); });
  const stopZoom = zoomPopover.subscribe(value => { if (value) cancelTabDrag(); });
  return () => { cancelTabDrag(); stopOverlay(); stopMedia(); stopPrivacy(); stopZoom(); };
}
