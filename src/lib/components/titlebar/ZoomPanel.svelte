<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { locale, t } from "../../i18n";
  import { errorText } from "../../i18n/errors";
  import { tabs } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceSelectionStore";
  import { tabLayout, isZenMode } from "../../stores/settingsStore";
  import { zoomPopover, closeZoom } from "../../stores/zoomPopoverStore";
  import { zoomState, zoomDrafts, zoomErrors, zoomError, ZOOM_LEVELS, setZoomFactor } from "../../stores/zoomStore";
  import { onOverlayOpening } from "../../services/bridge/overlays";
  import { onZoomPopoverAction, setZoomPopover } from "../../services/bridge/zoom-popover";
  import { describeError } from "../../services/user-errors.js";

  const active = $derived($tabs.find(tab => tab.is_active));
  let anchor = $state({ left:0, top:0, width:0, height:0 });
  function measure() {
    const r = $zoomPopover?.anchor.getBoundingClientRect();
    if (r && (r.left !== anchor.left || r.top !== anchor.top || r.width !== anchor.width || r.height !== anchor.height))
      anchor = { left:r.left, top:r.top, width:r.width, height:r.height };
  }
  $effect(() => {
    const session = $zoomPopover;
    if (session && (active?.id !== session.tabId || active.url !== session.url || $activeWorkspaceId !== session.workspaceId || $tabLayout !== session.layout || $isZenMode)) void closeZoom();
  });
  $effect(() => {
    const session = $zoomPopover;
    if (!session) return;
    untrack(measure);
    let frame = 0;
    const follow = () => {
      if (!session.anchor.isConnected || !session.anchor.getClientRects().length) { void closeZoom(); return; }
      measure(); frame = requestAnimationFrame(follow);
    };
    frame = requestAnimationFrame(follow);
    return () => cancelAnimationFrame(frame);
  });
  $effect(() => {
    const session = $zoomPopover;
    if (!session || !anchor.width) return;
    const style = getComputedStyle(document.documentElement);
    void setZoomPopover(true, session.id, {
      tabId:session.tabId, url:session.url, origin:session.origin, anchor,
      factor:$zoomDrafts[session.origin] ?? $zoomState.sites.find(site => site.origin === session.origin)?.factor ?? 1,
      levels:ZOOM_LEVELS, pending:$zoomDrafts[session.origin] !== undefined,
      error:$zoomErrors[session.origin] ? $errorText($zoomErrors[session.origin]) : "",
      label:$t("zoom.heading"), inLabel:$t("zoom.in"), outLabel:$t("zoom.out"), resetLabel:$t("zoom.reset_100"), locale:$locale,
      theme:{ background:"#1c1c1c", border:style.getPropertyValue("--border-subtle").trim(), accent:style.getPropertyValue("--accent-primary").trim() },
    }).catch(error => {
      if ($zoomPopover?.id === session.id) { zoomError.set(describeError(error)); void closeZoom(); }
    });
  });
  onMount(() => {
    let disposed = false, stopActions: (() => void) | undefined;
    const stopOverlays = onOverlayOpening(() => { void closeZoom(); });
    void onZoomPopoverAction(event => {
      const session = $zoomPopover;
      if (!session || session.id !== event.sessionId || session.tabId !== event.tabId || session.url !== event.url || session.origin !== event.origin) return;
      if (event.value !== null && active?.id === session.tabId && active.url === session.url) void setZoomFactor(event.value, session.origin);
      if (event.action === "dismiss") void closeZoom(["escape", "tab", "trigger"].includes(event.reason));
    }).then(stop => { if (disposed) stop(); else stopActions = stop; });
    return () => { disposed = true; stopActions?.(); stopOverlays(); void closeZoom(); };
  });
</script>

<svelte:window onresize={measure} onkeydown={event => { if (event.key === "Escape") void closeZoom(true); }} />
