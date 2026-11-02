<script lang="ts">
  import { t, formatNumber } from "../../i18n";
  import { zoomState, zoomBusy, zoomError, changeZoom } from "../../stores/zoomStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import ErrorMessage from "../status/ErrorMessage.svelte";
</script>
<section class="setting-group" aria-labelledby="zoom-heading">
  <h2 id="zoom-heading" class="group-title">{$t("zoom.heading")}</h2>
  <p>{$t(isTauriRuntime() ? "zoom.description" : "zoom.desktop")}</p>
  <ErrorMessage error={$zoomError} />
  {#each $zoomState.sites as site (site.origin)}
    <div class="setting-row"><span>{site.origin} · {$formatNumber(Math.round(site.factor*100))}%</span><button class="pill-action" disabled={$zoomBusy} onclick={() => changeZoom(0,site.origin)}>{$t("zoom.reset")}</button></div>
  {/each}
</section>
