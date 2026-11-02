<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { locale, t } from "../../i18n";
  import { tabs } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceSelectionStore";
  import { mediaPopover, closeMedia } from "../../stores/mediaPopoverStore";
  import { mediaState, mediaError, volumeDrafts, queueVolume } from "../../stores/mediaStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { onOverlayOpening } from "../../services/bridge/overlays";
  import { onMediaPopoverAction, setMediaPopover } from "../../services/bridge/media-popover";
  import { describeError } from "../../services/user-errors.js";

  const native = isTauriRuntime();
  const active = $derived($tabs.find(tab => tab.is_active));
  const media = $derived($mediaState.tabs.find(tab => tab.tab_id === $mediaPopover?.tabId));
  let anchor = $state({ left: 0, top: 0, width: 0, height: 0 });
  const previewLeft = $derived(Math.max(8, Math.min(anchor.left + anchor.width / 2 - 100, (typeof window === "undefined" ? 1280 : window.innerWidth) - 208)));

  function measure() {
    const rect = $mediaPopover?.anchor.getBoundingClientRect();
    if (rect && (rect.left !== anchor.left || rect.top !== anchor.top || rect.width !== anchor.width || rect.height !== anchor.height)) {
      anchor = { left: rect.left, top: rect.top, width: rect.width, height: rect.height };
    }
  }

  $effect(() => {
    const session = $mediaPopover;
    if (!session) return;
    if (active?.id !== session.tabId || active.url !== session.url || $activeWorkspaceId !== session.workspaceId
      || (media?.document_key ?? "") !== session.documentKey || !session.anchor.isConnected || !session.anchor.getClientRects().length) {
      void closeMedia();
    }
  });

  $effect(() => {
    const session = $mediaPopover;
    if (!session) return;
    untrack(measure);
    let frame = 0;
    // CSS focus transforms and sidebar animations can move the trigger without
    // resizing it. Read one rectangle while open; send only changed geometry.
    const followAnchor = () => {
      if (!session.anchor.isConnected || !session.anchor.getClientRects().length) { void closeMedia(); return; }
      measure();
      frame = requestAnimationFrame(followAnchor);
    };
    frame = requestAnimationFrame(followAnchor);
    return () => cancelAnimationFrame(frame);
  });

  $effect(() => {
    const session = $mediaPopover;
    if (!native || !session || !anchor.width) return;
    const style = getComputedStyle(document.documentElement);
    void setMediaPopover(true, session.id, {
      tabId: session.tabId, documentKey: session.documentKey, anchor,
      volume: $volumeDrafts[session.tabId] ?? media?.applied_volume ?? 100,
      disabled: !media?.document_key || media.volume_status === "unavailable" || !media.mute_available,
      label: $t("media.volume"), locale: $locale,
      theme: { background: "#1c1c1c", border: style.getPropertyValue("--border-subtle").trim(), accent: style.getPropertyValue("--accent-primary").trim() },
    }).catch(error => {
      if ($mediaPopover?.id !== session.id) return;
      mediaError.set(describeError(error));
      void closeMedia();
    });
  });

  onMount(() => {
    let disposed = false;
    let stopActions: (() => void) | undefined;
    const stopOverlays = onOverlayOpening(() => { void closeMedia(); });
    void onMediaPopoverAction(event => {
      const session = $mediaPopover;
      if (!session || event.sessionId !== session.id || event.tabId !== session.tabId || event.documentKey !== session.documentKey) return;
      if (event.value !== null && active?.id === session.tabId && media?.document_key === session.documentKey && media.volume_status !== "unavailable") {
        queueVolume(session.tabId, session.documentKey, event.value);
      }
      if (event.action === "dismiss") void closeMedia(["escape", "tab", "trigger"].includes(event.reason));
    }).then(stop => { if (disposed) stop(); else stopActions = stop; });
    return () => { disposed = true; stopActions?.(); stopOverlays(); void closeMedia(); };
  });
</script>

<svelte:window onresize={measure} onkeydown={event => { if (event.key === "Escape") void closeMedia(true); }} />

{#if !native && $mediaPopover}
  <div class="preview-dismiss" role="presentation" onclick={() => closeMedia()}>
    <div class="volume-popover" role="group" aria-label={$t("media.volume")} style:left={`${previewLeft}px`} style:top={`${anchor.top + anchor.height + 6}px`}>
      <input data-testid="media-volume" type="range" min="0" max="200" step="1" value="100" disabled aria-label={$t("media.volume")} aria-valuetext="100%" />
      <span class="volume-percentage" data-testid="media-volume-percentage" aria-hidden="true">100%</span>
    </div>
  </div>
{/if}

<style>
  .preview-dismiss { position: fixed; inset: 0; z-index: 2000; }
  .volume-popover { position: absolute; width: 200px; max-width: calc(100vw - 16px); height: 64px; box-sizing: border-box; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px; padding: 8px 12px; background: #1c1c1c; border: 1px solid var(--border-subtle); border-radius: 8px; box-shadow: 0 4px 12px #0004; }
  input { width: 100%; min-width: 0; height: 24px; margin: 0; flex-shrink: 0; accent-color: var(--accent-primary); opacity: .45; }
  .volume-percentage { color: #c9c9ce; font: 12px/16px system-ui, sans-serif; font-variant-numeric: tabular-nums; text-align: center; }
</style>
