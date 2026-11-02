<script lang="ts">
  import { t, formatNumber } from "../../i18n";
  import { currentZoom, zoomOrigin, zoomBusy, changeZoom } from "../../stores/zoomStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
</script>
<div class="zoom-controls" role="group" aria-label={$t("zoom.heading")} title={!isTauriRuntime() ? $t("zoom.desktop") : $zoomOrigin ?? ""}>
  <button data-testid="zoom-out" disabled={!isTauriRuntime() || !$zoomOrigin || $zoomBusy || $currentZoom <= 0.25} aria-label={$t("zoom.out")} onclick={() => changeZoom(-1)}>−</button>
  <button data-testid="zoom-reset" disabled={!isTauriRuntime() || !$zoomOrigin || $zoomBusy} aria-label={$t("zoom.reset")} onclick={() => changeZoom(0)}>{$formatNumber(Math.round($currentZoom * 100))}%</button>
  <button data-testid="zoom-in" disabled={!isTauriRuntime() || !$zoomOrigin || $zoomBusy || $currentZoom >= 5} aria-label={$t("zoom.in")} onclick={() => changeZoom(1)}>+</button>
</div>
<style>
  .zoom-controls { display: flex; align-items: center; flex-shrink: 0; gap: 2px; }
  button { background: transparent; border: 0; color: var(--text-secondary); padding: 4px; font-size: 11px; cursor: pointer; border-radius: 4px; }
  button:hover { background: var(--bg-hover); } button:disabled { opacity: .5; cursor: default; }
  button:focus-visible { outline: 2px solid var(--accent-primary); }
</style>
