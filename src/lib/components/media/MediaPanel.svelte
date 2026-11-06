<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { locale, t } from "../../i18n";
  import { tabs, selectTab } from "../../stores/browserStore";
  import { workspaces } from "../../stores/workspaceStore";
  import { activeWorkspaceId } from "../../stores/workspaceSelectionStore";
  import { mediaPopover, closeMedia } from "../../stores/mediaPopoverStore";
  import { mediaState, mediaError, volumeDrafts, queueVolume, toggleTabMute } from "../../stores/mediaStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { onOverlayOpening } from "../../services/bridge/overlays";
  import { onMediaPopoverAction, setMediaPopover } from "../../services/bridge/media-popover";
  import { describeError } from "../../services/user-errors.js";

  const native = isTauriRuntime();
  const active = $derived($tabs.find(tab => tab.is_active));
  const media = $derived($mediaState.tabs.find(tab => tab.tab_id === $mediaPopover?.tabId));
  const mediaItems = $derived.by(() => $mediaState.tabs.flatMap(tabMedia => {
    const tab = $tabs.find(item => item.id === tabMedia.tab_id);
    if (!tab) return [];
    const workspaceName = $workspaces.find(item => item.id === tab.workspace_id)?.name ?? "";
    const frames = tabMedia.frames.filter(frame => frame.playing || frame.picture_in_picture);
    const items = frames.map((frame, index) => ({
      tabId: tab.id, tabTitle: tab.title, workspaceName,
      frameLabel: frames.length === 1 ? $t("media.page") : index === 0 ? $t("media.page") : $t("media.frame", { index }),
      playing: frame.playing, audible: tabMedia.audible, muted: tabMedia.muted, muteAvailable: tabMedia.mute_available,
    }));
    if (!items.length && tabMedia.audible) items.push({
      tabId: tab.id, tabTitle: tab.title, workspaceName, frameLabel: $t("media.audio"),
      playing: false, audible: true, muted: tabMedia.muted, muteAvailable: tabMedia.mute_available,
    });
    return items;
  }));
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
      theme: { background: style.getPropertyValue("--bg-downloads-panel").trim() || "#1c1c1c", border: style.getPropertyValue("--border-subtle").trim(), accent: style.getPropertyValue("--accent-primary").trim() },
      items: mediaItems,
      frameTrackingSupported: $mediaState.frame_tracking_supported ?? false,
      labels: { title: $t("media.global_title"), playing: $t("media.playing"), audible: $t("media.audible"), mute: $t("media.mute"), unmute: $t("media.unmute"), noMedia: $t("media.no_active_media"), frameLimit: $t("media.frame_tracking_unavailable") },
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
      if ((event.action === "select" || event.action === "mute") && event.targetTabId) {
        const target = $tabs.find(tab => tab.id === event.targetTabId);
        if (!target) return;
        if (event.action === "select") void closeMedia().then(() => selectTab(target.id));
        else void toggleTabMute(target.id);
        return;
      }
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
    <div class="volume-popover" role="group" aria-label={$t("media.global_title")} style:left={`${previewLeft}px`} style:top={`${anchor.top + anchor.height + 6}px`}>
      <h3>{$t("media.global_title")}</h3>
      {#if !$mediaState.frame_tracking_supported}<p>{$t("media.frame_tracking_unavailable")}</p>{/if}
      {#if mediaItems.length}
        {#each mediaItems as item (`${item.tabId}:${item.frameLabel}`)}
          <div class="preview-item"><button onclick={event => { event.stopPropagation(); void closeMedia().then(() => selectTab(item.tabId)); }}>{item.tabTitle} — {item.frameLabel}</button>{#if item.muteAvailable}<button onclick={event => { event.stopPropagation(); void toggleTabMute(item.tabId); }}>{item.muted ? $t("media.unmute") : $t("media.mute")}</button>{/if}</div>
        {/each}
      {:else}<p>{$t("media.no_active_media")}</p>{/if}
    </div>
  </div>
{/if}

<style>
  .preview-dismiss { position: fixed; inset: 0; z-index: 2000; }
  .volume-popover { position: absolute; width: 320px; max-width: calc(100vw - 16px); max-height: min(440px, calc(100vh - 16px)); box-sizing: border-box; display: flex; flex-direction: column; align-items: stretch; gap: 6px; padding: 10px 12px; overflow: auto; background: var(--bg-downloads-panel, #1c1c1c); border: 1px solid var(--border-subtle); border-radius: 8px; box-shadow: 0 4px 12px #0004; }
  h3, p { margin: 0; font-size: 12px; }
  p { color: var(--text-secondary); }
  .preview-item { display: flex; gap: 6px; border-top: 1px solid var(--border-subtle); padding-top: 5px; }
  .preview-item button { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; background: transparent; color: inherit; border: 1px solid var(--border-subtle); border-radius: 5px; padding: 4px 6px; }
</style>
