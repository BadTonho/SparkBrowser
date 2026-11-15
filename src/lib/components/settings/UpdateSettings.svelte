<script lang="ts">
  import RuntimeSettings from "./RuntimeSettings.svelte";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { t, formatDate, formatNumber } from "../../i18n";
  import { updateState, updateBusy, updateActionError, checkUpdates, selectUpdateChannel, startUpdateDownload, startUpdateInstall } from "../../stores/updatesStore";
  import type { UpdateChannel } from "../../types/updates";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  const active = $derived($updateBusy || ["checking", "downloading", "installing"].includes($updateState.status));
  const status = $derived.by(() => {
    switch ($updateState.status) {
      case "checking": return $t("settings.checking_github_releases_71ad");
      case "up_to_date": return $t("settings.no_newer_version_is_available_for_this_channel_d0ae");
      case "available": return $t("settings.an_update_is_available_da18");
      case "downloading": return $t("settings.downloading_and_verifying_the_update_9dbf");
      case "ready": return $t("settings.the_signed_update_is_ready_to_install_5e30");
      case "installing": return $t("settings.saving_your_session_and_starting_the_installer_028b");
      case "error": return $t("settings.the_update_could_not_be_completed_5ba6");
      case "unavailable": return $t("common.updates_unavailable");
      default: return $t("settings.check_for_a_newer_version_of_spark_browser_078c");
    }
  });
  function date(timestamp: number) { return $formatDate(timestamp * 1000, { dateStyle: "medium", timeStyle: "short" }); }
</script>

<section class="setting-group" aria-labelledby="updates-heading">
  <h2 id="updates-heading" class="group-title">{$t("settings.about_updates_a1d4")}</h2>
  <p class="version">Spark {$updateState.current_version}</p>
  <div class="setting-row">
    <div class="row-text">
      <label for="update-channel" class="row-label">{$t("settings.release_channel_d89b")}</label>
      <span class="row-desc">{$t("settings.beta_includes_previews_and_newer_stable_releases_both_channels_share_you_f02b")}</span>
    </div>
    <select id="update-channel" class="engine-select" value={$updateState.channel} disabled={!isTauriRuntime() || active}
      onchange={event => { void selectUpdateChannel(event.currentTarget.value as UpdateChannel); }}>
      <option value="stable">{$t("settings.stable_90ee")}</option><option value="beta">{$t("settings.beta_7033")}</option>
    </select>
  </div>
  {#if $updateState.channel === "stable" && $updateState.current_version.includes("-beta.")}
    <p>{$t("settings.you_will_stay_on_this_beta_until_a_newer_stable_release_is_available_cha_2ff7")}</p>
  {/if}
  <p role="status" aria-live="polite">{status}</p>
  <p>{$t("settings.last_check_4941")} {$updateState.last_checked ? date($updateState.last_checked) : $t("settings.not_checked_yet_881d")}</p>
  {#if $updateState.error || $updateActionError}<ErrorMessage error={$updateActionError || $updateState.error} />{/if}
  {#if $updateState.retry_after}<p>{$t("settings.try_again_after_41f4")} {date($updateState.retry_after)}.</p>{/if}
  <div class="actions">
    <button data-testid="check-for-updates" class="pill-action" disabled={!$updateState.supported || active} onclick={() => { void checkUpdates(); }}>{$t("settings.check_for_updates_f26f")}</button>
    {#if $updateState.update}
      {#if $updateState.status === "ready"}
        <button class="pill-action" disabled={active} onclick={() => { void startUpdateInstall(); }}>{$t("settings.install_and_restart_1b99")}</button>
      {:else}
        <button class="pill-action" disabled={active} onclick={() => { void startUpdateDownload(); }}>{$t("settings.download_update_4f49")}</button>
      {/if}
    {/if}
  </div>
  {#if $updateState.status === "downloading"}
    <progress aria-label={$t("settings.update_download_1c20")} max={$updateState.total ?? 1} value={$updateState.total ? $updateState.received : undefined}></progress>
    <p>{$formatNumber($updateState.received / 1048576, { minimumFractionDigits: 1, maximumFractionDigits: 1 })} MB{$updateState.total ? ` / ${$formatNumber($updateState.total / 1048576, { minimumFractionDigits: 1, maximumFractionDigits: 1 })} MB` : ""}</p>
  {/if}
  {#if $updateState.update}
    <h3>{$t("settings.version_dd16")} {$updateState.update.version}</h3>
    <pre>{$updateState.update.notes || $t("common.no_release_notes")}</pre>
  {/if}
  <p>{$t("settings.updates_are_checked_when_the_browser_opens_downloading_and_installing_re_d689")}</p>
  <RuntimeSettings />
</section>

<style>
  p, pre { color: var(--text-secondary); font-size: 12px; line-height: 1.6; }
  .version { color: var(--text-primary); font-size: 16px; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin: 12px 0; }
  pre { white-space: pre-wrap; overflow-wrap: anywhere; font-family: inherit; max-height: 260px; overflow: auto; }
  progress { width: 100%; }
</style>
