<script lang="ts">
  import { t } from "../../i18n";
import PermissionSettings from "../permissions/PermissionSettings.svelte";
import {
  autoHibernateMode,
  residentTabLimit,
  setResidentTabLimit,
  setAutoHibernateMode,
} from "../../stores/settingsStore";
import { privacyStats, flipAdblockShield } from "../../stores/privacyStore";
const autoHibernateOptions = [5, 10, 30, "auto", "off"] as const;
</script>

<section class="setting-group" aria-labelledby="performance-heading">
  <h2 class="group-title" id="performance-heading">{$t("settings.performance_privacy_41ef")}</h2>

  <div class="setting-row" id="setting-resident-limit" tabindex="-1">
    <div class="row-text"><span class="row-label">{$t("settings.resident_tab_limit_9a94")}</span><span class="row-desc">{$t("settings.keep_up_to_this_many_webviews_awake_active_pinned_loading_media_download_9f92")}</span></div>
    <input type="number" min="1" max="100" step="1" value={$residentTabLimit} aria-label={$t("settings.resident_tab_limit_e214")} onchange={event => setResidentTabLimit(Number(event.currentTarget.value))} />
  </div>

  <div class="setting-row" id="setting-adblock-shield" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.privacy_shield_10ca")}</span>
      <span class="row-desc">{$t("settings.filters_known_tracker_hosts_ba80")} {$privacyStats.supports_live_policy_updates ? "Changing protection preserves page state." : "Changing protection reloads tabs and resets page state."}</span>
    </div>
    <button
      class="pill-action"
      class:active={$privacyStats.shield_enabled}
      onclick={flipAdblockShield}
    >
      {$privacyStats.shield_enabled ? $t("settings.enabled_92c1") : $t("settings.paused_e159")}
    </button>
  </div>

  <div class="setting-row" id="setting-auto-hibernate" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.auto_hibernate_inactive_tabs_f0c5")}</span>
      <span class="row-desc">
        {#if $autoHibernateMode === "off"}
          {$t("settings.automatic_hibernation_is_off_manual_hibernation_remains_available_f03e")}
        {:else if $autoHibernateMode === "auto"}
          {$t("settings.hibernates_regular_tabs_after_10_min_and_frequently_used_tabs_after_5_ho_8119")}
        {:else}
          {$t("settings.suspends_background_webviews_after_1a25")} {$autoHibernateMode} {$t("settings.min_of_inactivity_0853")}
        {/if}
      </span>
    </div>
    <div class="timer-pills">
      {#each autoHibernateOptions as mode}
        <button
          class="mini-pill"
          class:selected={$autoHibernateMode === mode}
          aria-pressed={$autoHibernateMode === mode}
          onclick={() => setAutoHibernateMode(mode)}
        >
          {typeof mode === "number" ? `${mode}m` : mode === "auto" ? $t("settings.auto_0286") : $t("settings.off_ca79")}
        </button>
      {/each}
    </div>
  </div>
</section>

<PermissionSettings />
