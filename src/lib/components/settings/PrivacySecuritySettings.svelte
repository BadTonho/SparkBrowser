<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import PermissionSettings from "../permissions/PermissionSettings.svelte";
  import TrackingCleaningSettings from "./TrackingCleaningSettings.svelte";
  import GpcSettings from "./GpcSettings.svelte";
  import ShieldExceptions from "./ShieldExceptions.svelte";
  import { privacyBusy, privacyError, privacyStats, flipAdblockShield } from "../../stores/privacyStore";
  import { clearAllHistory } from "../../services/bridge/persistence";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import {
    fetchPrivacySettings,
    setHistoryRetention,
    setHttpsOnly,
    setPopupSetting,
    setSmartScreen,
    type HistoryRetention,
    type PrivacySettings,
  } from "../../services/bridge/privacy-settings";
  import { describeError } from "../../services/user-errors.js";
  import { networkBusy, networkError, networkStatus, refreshNetwork, restartForNetwork, setDohProvider } from "../../stores/networkStore";
  import type { DohProvider } from "../../services/bridge/network";

  const retentionOptions: { value: HistoryRetention; label: string }[] = [
    { value: "one_month", label: "1 month" },
    { value: "three_months", label: "3 months" },
    { value: "six_months", label: "6 months" },
    { value: "twelve_months", label: "12 months" },
    { value: "always", label: "Always" },
    { value: "never", label: "Never" },
  ];
  let settings = $state<PrivacySettings | null>(null);
  let busy = $state(false);
  let error = $state("");
  let notice = $state("");
  const desktop = isTauriRuntime();

  onMount(() => {
    void refreshNetwork();
    void fetchPrivacySettings().then(value => settings = value).catch(cause => error = describeError(cause));
  });

  async function chooseRetention(event: Event): Promise<void> {
    if (!settings) return;
    const retention = (event.currentTarget as HTMLSelectElement).value as HistoryRetention;
    if (!retentionOptions.some(option => option.value === retention)) return;
    busy = true; error = ""; notice = "";
    try {
      await setHistoryRetention(retention);
      settings = { ...settings, history_retention: retention };
    } catch (cause) { error = describeError(cause); }
    finally { busy = false; }
  }

  async function changePopups(event: Event): Promise<void> {
    if (!settings) return;
    const enabled = (event.currentTarget as HTMLInputElement).checked;
    busy = true; error = "";
    try {
      await setPopupSetting(enabled);
      settings = { ...settings, allow_popups: enabled };
    } catch (cause) { error = describeError(cause); }
    finally { busy = false; }
  }

  function chooseDoh(event: Event): void {
    const value = (event.currentTarget as HTMLSelectElement).value;
    void setDohProvider(value === "cloudflare" || value === "google" ? (value as DohProvider) : null);
  }

  async function changeHttpsOnly(event: Event): Promise<void> {
    if (!settings) return;
    const enabled = (event.currentTarget as HTMLInputElement).checked;
    busy = true; error = "";
    try {
      await setHttpsOnly(enabled);
      settings = { ...settings, https_only: enabled };
    } catch (cause) { error = describeError(cause); }
    finally { busy = false; }
  }

  async function changeSmartScreen(event: Event): Promise<void> {
    if (!settings) return;
    const enabled = (event.currentTarget as HTMLInputElement).checked;
    busy = true; error = "";
    try {
      await setSmartScreen(enabled);
      settings = { ...settings, smart_screen_enabled: enabled, smart_screen_supported: true };
    } catch (cause) { error = describeError(cause); }
    finally { busy = false; }
  }

  async function clearHistory(): Promise<void> {
    if (!desktop || busy || !window.confirm($t("settings.clear_history_confirm"))) return;
    busy = true; error = ""; notice = "";
    try {
      await clearAllHistory();
      notice = $t("settings.history_cleared");
    } catch (cause) { error = describeError(cause) || $t("settings.history_clear_failed"); }
    finally { busy = false; }
  }

  async function changeShield(event: Event): Promise<void> {
    const checkbox = event.currentTarget as HTMLInputElement;
    const enabled = checkbox.checked;
    checkbox.checked = !enabled;
    try { await flipAdblockShield(); }
    catch { /* privacyError is exposed below; the checked value remains the committed store value. */ }
  }
</script>

<section class="setting-group privacy-security-settings" aria-labelledby="privacy-security-heading">
  <h2 class="group-title" id="privacy-security-heading">{$t("settings.privacy_security")}</h2>

  <div class="setting-subgroup" role="group" aria-labelledby="privacy-network-heading">
    <h3 class="subgroup-title" id="privacy-network-heading">{$t("settings.network_privacy")}</h3>
    <div class="setting-row" id="setting-network-doh" tabindex="-1" data-testid="doh-setting">
      <div class="row-text">
        <span class="row-label">{$t("settings.network_doh")}</span>
        <span class="row-desc">{$t("settings.network_doh_description")}</span>
        {#if $networkStatus.restartRequired}
          <span class="row-desc restart-note" role="status" data-testid="doh-restart-note">{$t("settings.network_doh_restart")}</span>
        {/if}
      </div>
      <div class="doh-controls">
        {#if $networkStatus.restartRequired}
          <button class="pill-action" type="button" data-testid="doh-restart" disabled={$networkBusy} onclick={restartForNetwork}>{$t("settings.network_doh_restart_now")}</button>
        {/if}
        <select class="privacy-select" aria-label={$t("settings.network_doh")} data-testid="doh-provider" value={$networkStatus.dohProvider ?? "off"} disabled={$networkBusy || !desktop} onchange={chooseDoh}>
          <option value="off">{$t("settings.network_doh_off")}</option>
          <option value="cloudflare">Cloudflare (1.1.1.1)</option>
          <option value="google">Google (8.8.8.8)</option>
        </select>
      </div>
    </div>
    {#if $networkError}<p class="settings-message" role="alert">{$networkError}</p>{/if}
    <label class="setting-row" id="setting-network-https" tabindex="-1">
      <span class="row-text">
        <span class="row-label">{$t("settings.network_https")}</span>
        <span class="row-desc">{$t("settings.network_https_description")}</span>
      </span>
      <input type="checkbox" role="switch" aria-label={$t("settings.network_https")} checked={settings?.https_only ?? false} disabled={!settings || busy} onchange={changeHttpsOnly} />
    </label>
  </div>

  <div class="setting-subgroup" role="group" aria-labelledby="privacy-history-heading">
    <h3 class="subgroup-title" id="privacy-history-heading">{$t("settings.privacy_group_history")}</h3>
    <div class="setting-row" id="clear-local-history" tabindex="-1">
      <div class="row-text">
        <span class="row-label">{$t("settings.clear_local_history")}</span>
        <span class="row-desc">{$t("settings.clear_local_history_description")}</span>
      </div>
      <button class="pill-action" type="button" disabled={!desktop || busy} onclick={clearHistory}>{$t("settings.clear_history_button")}</button>
    </div>
    <div class="setting-row" id="setting-history-retention" tabindex="-1">
      <div class="row-text">
        <span class="row-label">{$t("settings.local_history_retention")}</span>
        <span class="row-desc">{$t("settings.local_history_retention_description")}</span>
      </div>
      <select class="privacy-select" aria-label={$t("settings.local_history_retention")} value={settings?.history_retention ?? "three_months"} disabled={!settings || busy} onchange={chooseRetention}>
        {#each retentionOptions as option (option.value)}<option value={option.value}>{$t(`settings.retention_${option.value}` as never)}</option>{/each}
      </select>
    </div>
  </div>

  <div class="setting-subgroup" role="group" aria-labelledby="privacy-safe-browsing-heading">
    <h3 class="subgroup-title" id="privacy-safe-browsing-heading">{$t("settings.privacy_group_safe_browsing")}</h3>
    <label class="setting-row" id="setting-smart-screen" tabindex="-1">
      <span class="row-text">
        <span class="row-label">{$t("settings.smart_screen")}</span>
        <span class="row-desc">{$t("settings.smart_screen_description")}</span>
        {#if settings && !settings.smart_screen_supported}<span class="row-desc unavailable">{$t(desktop ? "settings.smart_screen_unavailable" : "settings.smart_screen_platform_unavailable")}</span>{/if}
      </span>
      <input type="checkbox" role="switch" aria-label={$t("settings.smart_screen")} checked={settings?.smart_screen_enabled ?? true} disabled={!settings?.smart_screen_supported || busy} onchange={changeSmartScreen} />
    </label>
  </div>

  <div class="setting-subgroup" role="group" aria-labelledby="privacy-shield-heading">
    <h3 class="subgroup-title" id="privacy-shield-heading">{$t("settings.privacy_group_shield")}</h3>
    <section class="setting-card" id="setting-adblock-shield" tabindex="-1" aria-labelledby="adblock-shield-title">
      <label class="setting-card-head">
        <span class="row-text">
          <span class="row-label" id="adblock-shield-title">{$t("settings.privacy_shield_10ca")}</span>
          <span class="row-desc">{$t("settings.filters_known_tracker_hosts_ba80")} {$t("settings.protection_reload_note")}</span>
        </span>
        <input type="checkbox" role="switch" aria-label={$t("settings.privacy_shield_10ca")} checked={$privacyStats.shield_enabled} disabled={$privacyBusy} onchange={changeShield} />
      </label>
      <ShieldExceptions />
    </section>
    <div class="setting-row">
      <div class="row-text"><span class="row-label">{$t("settings.advanced_protection")}</span><span class="row-desc">{$t("settings.advanced_protection_unavailable")}</span></div>
    </div>
    <label class="setting-row" id="setting-popup-policy" tabindex="-1">
      <span class="row-text">
        <span class="row-label">{$t("settings.popup_policy")}</span>
        <span class="row-desc">{$t("settings.popup_policy_description")}</span>
      </span>
      <input type="checkbox" role="switch" aria-label={$t("settings.popup_policy")} checked={settings?.allow_popups ?? false} disabled={!settings || busy} onchange={changePopups} />
    </label>
  </div>

  <div class="setting-subgroup" role="group" aria-labelledby="privacy-tracking-heading">
    <h3 class="subgroup-title" id="privacy-tracking-heading">{$t("settings.privacy_group_tracking")}</h3>
    <GpcSettings />
    <TrackingCleaningSettings />
  </div>

  {#if error}<p class="settings-message" role="alert">{error}</p>{/if}
  {#if $privacyError}<p class="settings-message" role="alert">{$privacyError}</p>{/if}
  {#if notice}<p class="settings-message" role="status">{notice}</p>{/if}

  <PermissionSettings />
</section>

<style>
  .doh-controls { display: flex; align-items: center; gap: 10px; flex: 0 0 auto; }
  .restart-note { color: #fcd34d; }
  .unavailable { color: var(--text-muted); }
  .privacy-select { min-width: 180px; min-height: 34px; padding: 6px 10px; border: 1px solid var(--border-subtle); border-radius: var(--radius-md); background: rgba(255,255,255,.04); color: var(--text-primary); font: inherit; font-size: 12px; }
  @media (max-width: 720px) {
    .privacy-security-settings .setting-row { align-items: flex-start; flex-direction: column; }
    .privacy-select { width: 100%; max-width: 260px; min-width: 0; }
  }
</style>
