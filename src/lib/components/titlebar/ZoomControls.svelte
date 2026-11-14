<script lang="ts">
  import { t } from "../../i18n";
  import { prepares } from "../../actions/preparePanel";
  import { currentZoom, zoomOrigin } from "../../stores/zoomStore";
  import { zoomPopover, openZoom } from "../../stores/zoomPopoverStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
</script>
<button use:prepares={"zoom"} class="zoom-button" class:active={$currentZoom !== 1} data-testid="open-zoom" disabled={!isTauriRuntime() || !$zoomOrigin}
  aria-label={$t("zoom.heading")} title={$t(!isTauriRuntime() ? "zoom.desktop" : !$zoomOrigin ? "zoom.unsupported" : "zoom.heading")}
  aria-haspopup="dialog" aria-expanded={Boolean($zoomPopover)} onclick={event => openZoom(event.currentTarget)}>
  <svg width="15" height="15" viewBox="0 0 24 24" stroke="currentColor" fill="none" stroke-width="1.8" aria-hidden="true"><circle cx="10" cy="10" r="6" /><path d="m15 15 5 5M7 10h6M10 7v6" stroke-linecap="round" /></svg>
</button>
<style>
  button { width:25px; height:25px; display:grid; place-items:center; flex-shrink:0; color:var(--text-secondary); border-radius:4px; }
  button:hover { background:var(--bg-glass-hover); }
  button.active { color:var(--accent-primary); background:var(--bg-glass-hover); }
  button:disabled { opacity:.45; cursor:default; }
  button:focus-visible { outline: 2px solid var(--accent-primary); }
</style>
