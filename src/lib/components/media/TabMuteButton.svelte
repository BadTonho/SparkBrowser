<script lang="ts">
  import { t } from "../../i18n";
  import { mediaState, mediaBusy, toggleTabMute } from "../../stores/mediaStore";
  let { tabId }: {tabId: string} = $props();
  const media = $derived($mediaState.tabs.find(t => t.tab_id === tabId));
</script>
{#if media?.mute_available && (media.audible || media.muted)}
  <button class="media-mute" data-testid="tab-mute" data-media-tab={tabId}
    disabled={$mediaBusy.has(tabId)} aria-pressed={media.muted}
    aria-label={$t(media.muted ? "media.unmute" : "media.mute")}
    title={$t(media.muted ? "media.unmute" : "media.mute")}
    onpointerdown={e => e.stopPropagation()} onmouseup={e => e.stopPropagation()}
    onclick={e => { e.stopPropagation(); void toggleTabMute(tabId); }}>
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
      <path d="M11 5 6 9H3v6h3l5 4z" />
      {#if media.muted}<path d="m16 9 6 6m0-6-6 6" />{:else}<path d="M15 8a6 6 0 0 1 0 8m3-11a10 10 0 0 1 0 14" />{/if}
    </svg>
  </button>
{/if}
<style>
  .media-mute { flex-shrink: 0; width: 23px; height: 23px; display: grid; place-items: center; color: var(--text-secondary); border-radius: 4px; }
  .media-mute:hover, .media-mute[aria-pressed="true"] { color: var(--text-primary); background: var(--bg-glass-hover); }
  .media-mute:focus-visible { outline: 2px solid var(--accent-primary); }
</style>
