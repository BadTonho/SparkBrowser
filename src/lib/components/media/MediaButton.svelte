<script lang="ts">
  import { t } from "../../i18n";
  import { openMedia, isMediaOpen } from "../../stores/mediaPopoverStore";
</script>
<button data-testid="open-media" aria-label={$t("media.global_title")} title={$t("media.global_title")} aria-expanded={$isMediaOpen} onclick={event => openMedia(event.currentTarget)}>
  <svg width="15" height="15" viewBox="0 0 24 24" stroke="currentColor" fill="none" stroke-width="2" aria-hidden="true"><path d="M4 9v6m5-11v16m5-13v10m5-7v4" /></svg>
</button>
<style>button {width:25px;height:25px;display:grid;place-items:center;flex-shrink:0;color:var(--text-secondary);border-radius:4px;}button:hover{background:var(--bg-glass-hover);}button:focus-visible{outline:2px solid var(--accent-primary);}</style>
