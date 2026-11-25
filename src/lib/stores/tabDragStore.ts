import { createSharedLifecycle } from "../services/lifecycle.js";
import { get, writable } from "svelte/store";
import { tabs, beginTabReorder, previewTabReorder, finishTabReorder, applyTabsSnapshot, selectTab, changeTabWorkspace } from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { splitLayout, splitKey, splitGroupsKey } from "./splitViewStore";
import { replaceGroupOrder } from "../services/split-tab-groups.js";
import { tabLayout } from "./settingsStore";
import { locale, tr } from "../i18n";
import { onOverlayOpening } from "../services/bridge/overlays";
import { detachTabToNewWindow } from "../services/bridge/windows";
import { dockTabToSplit, moveTabInSplit, setDragPreview, flushDragPreview, type DockSide, type DockContext } from "../services/bridge/tab-drag";
import { crossedDragThreshold, splitDropSide } from "../services/tab-drag-geometry.js";
import { tabGroups, addTabsToGroup, removeTabsFromGroup } from "./tabGroupsStore";
import { selectedTabIds } from "./tabSelectionStore";
import { mediaPopover, closeMedia } from "./mediaPopoverStore";
import { privacyPopover, closePrivacy } from "./privacyPopoverStore";
import { zoomPopover, closeZoom } from "./zoomPopoverStore";
import { isTauriRuntime } from "../services/bridge/runtime";
import { logIgnored } from "../services/ignored-error";
import { createTabGhost, type TabGhost } from "../services/tab-drag-ghost";

interface Drag {
  id: string; pointerId: number; root: HTMLElement; startX: number; startY: number;
  x: number; y: number;
  moved: boolean; side: DockSide | null; context: DockContext; layout: string;
  cursor: string; userSelect: string; original: import("../types/browser").TabItem[];
  ghost: TabGhost | null;
}
/** Vertical list in split view: where the dragged tab would land on the other side, so the list can show it. */
export const splitDragPreview = writable<{ id: string; side: DockSide; beforeId: string | null } | null>(null);
export const tabDrag = writable<{ id: string; dragging: boolean; side: DockSide | null } | null>(null);
let drag: Drag | null = null, busy = false, frame = 0, lastPreview = "", pendingMove = false;
let hoveredWorkspace: Element | null = null;
let hoveredGroup: Element | null = null;
let lastReorderTarget: string | null = null;

function publish() { tabDrag.set(drag ? { id: drag.id, dragging: drag.moved, side: drag.side } : null); }
function preview(side: DockSide | null, bounds: DOMRect | undefined) {
  if (!drag) return;
  const gesture = drag;
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
  previous?.ghost?.destroy();
  drag = null; lastPreview = ""; pendingMove = false; cancelAnimationFrame(frame); frame = 0;
  window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', release);
  window.removeEventListener('pointercancel', cancelTabDrag); window.removeEventListener('blur', cancelTabDrag);
  window.removeEventListener('keydown', key, true);
  previous?.root.removeEventListener('lostpointercapture', cancelTabDrag);
  if (previous?.root.hasPointerCapture(previous.pointerId)) previous.root.releasePointerCapture(previous.pointerId);
  document.body.style.cursor = previous?.cursor ?? ''; document.body.style.userSelect = previous?.userSelect ?? '';
  document.querySelectorAll('.drop-hover').forEach(element => element.classList.remove('drop-hover'));
  hoveredWorkspace = null; hoveredGroup = null; lastReorderTarget = null;
  tabDrag.set(null); splitDragPreview.set(null);
  const hidden = keepDockPreview ? flushDragPreview() : setDragPreview(null);
  void hidden.catch(logIgnored);
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
  // Over the page the shell is covered by the page's own layer, so the native preview shows the tab there.
  const overPage = Boolean(bounds && drag.x >= bounds.left && drag.x <= bounds.right && drag.y >= bounds.top && drag.y <= bounds.bottom);
  drag.ghost?.setHidden(overPage);
  if (drag.side !== side) { drag.side = side; publish(); }
  preview(side, bounds);
}
/** True when the pointer is outside this window: releasing there moves the tab to a new window. */
function outsideWindow(event: PointerEvent) {
  return event.clientX < 0 || event.clientY < 0 || event.clientX > innerWidth || event.clientY > innerHeight;
}
function move(event: PointerEvent) {
  if (!drag || event.pointerId !== drag.pointerId) return;
  if (!contextValid()) { cancelTabDrag(); return; }
  if (!drag.moved && crossedDragThreshold(drag.startX, drag.startY, event.clientX, event.clientY)) {
    drag.moved = true; document.body.style.cursor = 'grabbing'; document.body.style.userSelect = 'none';
    const source = document.querySelector<HTMLElement>(`[data-tab-id="${CSS.escape(drag.id)}"]`);
    if (source) drag.ghost = createTabGhost(source, drag.root, drag.startX, drag.startY);
    void closeMedia(); void closePrivacy(); void closeZoom();
    publish();
  }
  if (!drag.moved) return;
  drag.x = event.clientX; drag.y = event.clientY;
  drag.ghost?.move(drag.x, drag.y);
  pendingMove = true;
}
/** The split side of the vertical list section under the pointer: the tabs above the separator are the left
 *  side, the ones from the separator down to the next one are the right side. Undefined outside the list,
 *  above it, and in the section of tabs outside the split. */
function verticalPaneAt(target: Element | null | undefined, y: number): DockSide | undefined {
  const section = target?.closest('.tabs-section');
  if (!section) return undefined;
  let side: string | undefined = target?.closest('.tabs-list') ? 'left' : undefined;
  for (const separator of section.querySelectorAll<HTMLElement>('.pane-separator')) {
    if (separator.getBoundingClientRect().top <= y) side = separator.dataset.paneSide ?? 'other';
  }
  return side === 'left' || side === 'right' ? side : undefined;
}
/** Shows the dragged tab on the other side of the split while the pointer is over it. */
function updateSplitPreview(target: Element | null | undefined) {
  if (!drag) return;
  const split = get(splitLayout);
  const source: DockSide | undefined = split?.left_tab_ids.includes(drag.id) ? 'left' : split?.right_tab_ids.includes(drag.id) ? 'right' : undefined;
  const side = drag.layout === 'vertical' && drag.moved ? verticalPaneAt(target, drag.y) : undefined;
  const previous = get(splitDragPreview);
  if (!source || !side || side === source) { if (previous) splitDragPreview.set(null); return; }
  const row = target?.closest<HTMLElement>('.tab-row');
  const over = row && row.dataset.tabId !== drag.id && row.dataset.paneSide === side ? row.dataset.tabId ?? null : undefined;
  // Over nothing in particular (a gap, or the placeholder itself): keep the place already chosen on this side.
  const beforeId = over !== undefined ? over : previous?.side === side ? previous.beforeId : null;
  if (previous?.side === side && previous.beforeId === beforeId) return;
  splitDragPreview.set({ id: drag.id, side, beforeId });
}
/** The group header (or a tab of a group) under the pointer, unless the dragged tab already belongs to it. */
function groupUnder(target: Element | null | undefined): HTMLElement | null {
  if (!drag) return null;
  const element = target?.closest<HTMLElement>('.tab-group-header,.tab-row[data-group-id]') ?? null;
  const id = element?.dataset.groupId;
  return element && id && get(tabGroups).members[drag.id] !== id ? element : null;
}
/** The tabs a drag moves: the whole selection when the dragged tab is part of it. */
function draggedIds(id: string) {
  const selected = get(selectedTabIds);
  return selected.includes(id) ? [...selected] : [id];
}
function processMove() {
  if (!drag || !pendingMove) return;
  pendingMove = false;
  updateDestination();
  const target = document.elementFromPoint(drag.x, drag.y);
  updateSplitPreview(target);
  const pill = target?.closest<HTMLElement>('.ws-pill');
  if (hoveredWorkspace !== pill) {
    hoveredWorkspace?.classList.remove('drop-hover');
    pill?.classList.add('drop-hover');
    hoveredWorkspace = pill ?? null;
  }
  const group = groupUnder(target);
  if (hoveredGroup !== group) {
    hoveredGroup?.classList.remove('drop-hover');
    group?.classList.add('drop-hover');
    hoveredGroup = group;
  }
  const over = target?.closest<HTMLElement>('.h-tab,.tab-row')?.dataset.tabId;
  if (drag.side || !over || over === drag.id) lastReorderTarget = null;
  if (!drag.side && over && over !== drag.id && over !== lastReorderTarget) {
    const split = get(splitLayout);
    const membership = split && drag.layout === 'horizontal' ? (split.left_tab_ids.includes(drag.id) ? split.left_tab_ids : split.right_tab_ids) : null;
    if (membership && !membership.includes(over)) return;
    const ids = get(tabs).filter(tab => tab.workspace_id === drag!.context.workspaceId && (!membership || membership.includes(tab.id))).map(tab => tab.id);
    const from = ids.indexOf(drag.id), to = ids.indexOf(over);
    if (from >= 0 && to >= 0) { lastReorderTarget = over; ids.splice(to, 0, ids.splice(from, 1)[0]); previewTabReorder(membership ? replaceGroupOrder(get(tabs), ids) : ids); }
  }
}
async function release(event: PointerEvent) {
  if (!drag || event.pointerId !== drag.pointerId) return;
  if (!contextValid()) { cancelTabDrag(); return; }
  if (outsideWindow(event)) { await detachDraggedTab(event); return; }
  // Re-evaluate docking at release without repeating the last provisional reorder.
  move(event);
  processMove();
  if (drag?.moved) updateDestination();
  if (!drag) return;
  const target = document.elementFromPoint(event.clientX, event.clientY);
  const targetWorkspace = target?.closest<HTMLElement>('.ws-pill')?.dataset.wsId;
  const targetTabList = target?.closest('.tabs-scroll,.tabs-list');
  // Vertical list: the side under the pointer is the section between two separators, so a drop anywhere in it counts.
  const split = get(splitLayout);
  const sourceSide: DockSide | undefined = split?.left_tab_ids.includes(drag.id) ? 'left' : split?.right_tab_ids.includes(drag.id) ? 'right' : undefined;
  const sectionSide = drag.layout === 'vertical' ? verticalPaneAt(target, event.clientY) : undefined;
  const shown = get(splitDragPreview);
  // The place the list was showing wins, so the tab lands exactly where the preview put it.
  const crossSide = shown?.side ?? (sectionSide && sourceSide && sectionSide !== sourceSide ? sectionSide : undefined);
  const targetSide = (targetTabList as HTMLElement | undefined)?.dataset.paneSide as DockSide | undefined ?? crossSide;
  // A tab only works as the drop position when it is on the destination side.
  const rowUnder = target?.closest<HTMLElement>('.h-tab,.tab-row');
  const targetTab = shown ? shown.beforeId ?? undefined : !rowUnder || (rowUnder.dataset.paneSide && rowUnder.dataset.paneSide !== targetSide) ? undefined : rowUnder.dataset.tabId;
  const displayed = targetSide && get(splitLayout) ? get(tabs).filter(tab => get(splitLayout)![targetSide === 'left' ? 'left_tab_ids' : 'right_tab_ids'].includes(tab.id)).map(tab => tab.id) : [];
  const beforeId = targetTab && targetTab !== drag.id ? targetTab : displayed.includes(drag.id) ? displayed[displayed.indexOf(drag.id) + 1] ?? null : null;
  const dropGroup = drag.moved ? groupUnder(target)?.dataset.groupId : undefined;
  const dropRow = drag.moved ? target?.closest<HTMLElement>('.tab-row') : null;
  const leaveGroup = Boolean(dropRow && !dropRow.dataset.groupId && get(tabGroups).members[drag.id]);
  const movedIds = draggedIds(drag.id);
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
    } else if (ended.moved && dropGroup) {
      await finishTabReorder(true); addTabsToGroup(movedIds, dropGroup);
    } else {
      if (leaveGroup) removeTabsFromGroup(movedIds);
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
/** Dropping a tab outside the window opens it in a new window at the drop point. */
async function detachDraggedTab(event: PointerEvent) {
  const dragged = drag;
  if (!dragged) return;
  // The only tab of a window has nothing to leave behind, so it stays where it is.
  const detachable = dragged.moved && isTauriRuntime() && get(tabs).length > 1;
  const ended = cleanup();
  if (!ended) return;
  busy = true;
  try {
    if (!detachable) { await finishTabReorder(true); return; }
    await finishTabReorder(true);
    const scale = window.devicePixelRatio || 1;
    await detachTabToNewWindow(ended.id, Math.round(event.screenX * scale), Math.round(event.screenY * scale));
  } finally { busy = false; }
}
export function startTabDrag(event: PointerEvent, id: string) {
  if (busy || drag || event.button !== 0 || !event.isPrimary || (event.target as HTMLElement).closest('button,input,a')) return;
  const active = get(tabs).find(tab => tab.is_active), root = document.querySelector<HTMLElement>('.browser-root');
  if (!active || !root) return;
  const split = get(splitLayout);
  event.preventDefault();
  drag = { id, pointerId: event.pointerId, root, startX: event.clientX, startY: event.clientY, x: event.clientX, y: event.clientY, moved: false, side: null,
    context: { workspaceId: get(activeWorkspaceId), activeTabId: active.id, splitKey: split ? splitKey(split) : null, groupsKey: split ? splitGroupsKey(split) : null }, layout: get(tabLayout),
    cursor: document.body.style.cursor, userSelect: document.body.style.userSelect, original: get(tabs), ghost: null };
  beginTabReorder();
  try { root.setPointerCapture(event.pointerId); }
  catch (error) { cancelTabDrag(); throw error; }
  publish();
  root.addEventListener('lostpointercapture', cancelTabDrag);
  window.addEventListener('pointermove', move); window.addEventListener('pointerup', release);
  window.addEventListener('pointercancel', cancelTabDrag); window.addEventListener('blur', cancelTabDrag); window.addEventListener('keydown', key, true);
  const follow = () => { if (!drag) return; if (!contextValid()) { cancelTabDrag(); return; } if (pendingMove) processMove(); else if (drag.moved) updateDestination(); frame = requestAnimationFrame(follow); };
  frame = requestAnimationFrame(follow);
}
function startTabDragLifecycle() {
  const stopOverlay = onOverlayOpening(cancelTabDrag);
  const stopMedia = mediaPopover.subscribe(value => { if (value) cancelTabDrag(); });
  const stopPrivacy = privacyPopover.subscribe(value => { if (value) cancelTabDrag(); });
  const stopZoom = zoomPopover.subscribe(value => { if (value) cancelTabDrag(); });
  return () => { cancelTabDrag(); stopOverlay(); stopMedia(); stopPrivacy(); stopZoom(); };
}

export const initializeTabDrag = createSharedLifecycle(scope => scope.own(startTabDragLifecycle()));
