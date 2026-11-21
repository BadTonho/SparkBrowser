<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { logIgnored } from "../../services/ignored-error";
  import { askDownloadDestination, setAskDownloadDestination } from "../../stores/settingsStore";
  import {
    downloadLocation, downloadLocationError, loadDownloadLocation, pickDownloadLocation, setShowDownloadsWhenDone,
    showDownloadsWhenDone, useDefaultDownloadLocation,
  } from "../../stores/downloadSettingsStore";

  onMount(() => { void loadDownloadLocation(); });
</script>

<section class="setting-group" aria-labelledby="downloads-heading">
  <h2 id="downloads-heading" class="group-title">{$t("settings.downloads_category")}</h2>

  <div class="setting-row" id="setting-download-location" tabindex="-1" data-testid="download-location">
    <div class="row-text">
      <span class="row-label">{$t("settings.download_location")}</span>
      <span class="row-desc" data-testid="download-location-path">{$downloadLocation?.path ?? ""}</span>
    </div>
    <div>
      {#if $downloadLocation && !$downloadLocation.is_default}
        <button class="pill-action" type="button" onclick={() => void useDefaultDownloadLocation()} data-testid="download-location-reset">{$t("settings.download_location_reset")}</button>
      {/if}
      <button class="pill-action" type="button" onclick={() => void pickDownloadLocation()} data-testid="download-location-change">{$t("settings.download_location_change")}</button>
    </div>
  </div>
  {#if $downloadLocationError}<p class="row-desc" role="alert" data-testid="download-location-error">{$t("settings.download_location_error")}</p>{/if}

  <div class="setting-row" id="setting-download-destination" tabindex="-1">
    <div class="row-text"><span class="row-label">{$t("settings.ask_download_destination")}</span><span class="row-desc">{$t("settings.ask_download_destination_description")}</span></div>
    <input type="checkbox" checked={$askDownloadDestination} aria-label={$t("settings.ask_download_destination")} onchange={event => void setAskDownloadDestination(event.currentTarget.checked).catch(logIgnored)} />
  </div>

  <div class="setting-row" id="setting-show-downloads-when-done" tabindex="-1">
    <div class="row-text"><span class="row-label">{$t("settings.show_downloads_when_done")}</span><span class="row-desc">{$t("settings.show_downloads_when_done_description")}</span></div>
    <input type="checkbox" checked={$showDownloadsWhenDone} aria-label={$t("settings.show_downloads_when_done")} onchange={event => setShowDownloadsWhenDone(event.currentTarget.checked)} data-testid="show-downloads-when-done" />
  </div>
</section>
