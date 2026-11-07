<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import PermissionSettings from "../permissions/PermissionSettings.svelte";
  import { privacyBusy, privacyError, privacyStats, flipAdblockShield } from "../../stores/privacyStore";
  import { clearAllHistory } from "../../services/bridge/persistence";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import {
    fetchPrivacySettings,
    setHistoryRetention,
    setPopupSetting,
    setSmartScreen,
    type HistoryRetention,
    type PrivacySettings,
  } from "../../services/bridge/privacy-settings";
  import { describeError } from "../../services/user-errors.js";

  const retentionOptions: { value: HistoryRetention; label: string }[] = [
    { value: "one_month", label: "1 month" },
    { value: "three_months", label: "3 months" },
    { value: "six_months", label: "6 months" },
    { value: "twelve_months", label: "12 months" },
    { value: "always", label: "Always" },
  ];
  let settings = $state<PrivacySettings | null>(null);
  let busy = $state(false);
  let error = $state("");
  let notice = $state("");
  const desktop = isTauriRuntime();

  onMount(() => {
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

  <h3 class="group-title privacy-subheading">{$t("settings.network_privacy")}</h3>
  <div class="setting-row" id="setting-network-tor">
    <div class="row-text"><span class="row-label">{$t("settings.network_tor")}</span><span class="row-desc">{$t("settings.network_tor_unavailable")}</span></div>
  </div>
  <div class="setting-row" id="setting-network-doh">
    <div class="row-text"><span class="row-label">{$t("settings.network_doh")}</span><span class="row-desc">{$t("settings.network_doh_unavailable")}</span></div>
  </div>
  <div class="setting-row" id="setting-network-https">
    <div class="row-text"><span class="row-label">{$t("settings.network_https")}</span><span class="row-desc">{$t("settings.network_https_unavailable")}</span></div>
  </div>

  <h3 class="group-title">{$t("settings.clear_local_history")}</h3>
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
    <select aria-label={$t("settings.local_history_retention")} value={settings?.history_retention ?? "three_months"} disabled={!settings || busy} onchange={chooseRetention}>
      {#each retentionOptions as option (option.value)}<option value={option.value}>{$t(`settings.retention_${option.value}` as never)}</option>{/each}
    </select>
  </div>

  <h3 class="group-title privacy-subheading">{$t("settings.smart_screen")}</h3>
  <label class="privacy-switch-row" id="setting-smart-screen" tabindex="-1">
    <span class="row-text">
      <span class="row-label">{$t("settings.smart_screen")}</span>
      <span class="row-desc">{$t("settings.smart_screen_description")}</span>
      {#if settings && !settings.smart_screen_supported}<span class="row-desc unavailable">{$t(desktop ? "settings.smart_screen_unavailable" : "settings.smart_screen_platform_unavailable")}</span>{/if}
    </span>
    <span class="switch-control">
      <input type="checkbox" role="switch" aria-label={$t("settings.smart_screen")} checked={settings?.smart_screen_enabled ?? true} disabled={!settings?.smart_screen_supported || busy} onchange={changeSmartScreen} />
      <span class="switch-track" aria-hidden="true"></span>
    </span>
  </label>

  <h3 class="group-title privacy-subheading">{$t("settings.site_protection")}</h3>
  <label class="privacy-switch-row" id="setting-popup-policy" tabindex="-1">
    <span class="row-text">
      <span class="row-label">{$t("settings.popup_policy")}</span>
      <span class="row-desc">{$t("settings.popup_policy_description")}</span>
    </span>
    <span class="switch-control">
      <input type="checkbox" role="switch" aria-label={$t("settings.popup_policy")} checked={settings?.allow_popups ?? false} disabled={!settings || busy} onchange={changePopups} />
      <span class="switch-track" aria-hidden="true"></span>
    </span>
  </label>
  <label class="privacy-switch-row" id="setting-adblock-shield" tabindex="-1">
    <span class="row-text">
      <span class="row-label">{$t("settings.privacy_shield_10ca")}</span>
      <span class="row-desc">{$t("settings.filters_known_tracker_hosts_ba80")} {$t("settings.protection_reload_note")}</span>
    </span>
    <span class="switch-control">
      <input type="checkbox" role="switch" aria-label={$t("settings.privacy_shield_10ca")} checked={$privacyStats.shield_enabled} disabled={$privacyBusy} onchange={changeShield} />
      <span class="switch-track" aria-hidden="true"></span>
    </span>
  </label>

  {#if error}<p class="settings-message" role="alert">{error}</p>{/if}
  {#if $privacyError}<p class="settings-message" role="alert">{$privacyError}</p>{/if}
  {#if notice}<p class="settings-message" role="status">{notice}</p>{/if}

  <PermissionSettings />
</section>

<style>
  .privacy-security-settings { gap: 10px; }
  .privacy-security-settings > .group-title:first-child { margin-bottom: 4px; }
  .privacy-subheading { margin-top: 12px; }
  .privacy-security-settings .setting-row { min-height: 62px; }
  .privacy-security-settings select { width: min(180px, 40%); min-height: 36px; padding: 6px 10px; border: 1px solid var(--border-subtle); border-radius: var(--radius-md); background: rgba(255,255,255,.04); color: var(--text-primary); font: inherit; font-size: 11px; }
  .privacy-switch-row { min-height: 58px; display: flex; align-items: center; justify-content: space-between; gap: 18px; padding: 12px 14px; border-bottom: 1px solid var(--border-subtle); cursor: pointer; }
  .privacy-switch-row:first-of-type { border-top: 1px solid var(--border-subtle); }
  .privacy-switch-row:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 2px; border-radius: var(--radius-sm); }
  .privacy-switch-row .row-text { gap: 4px; }
  .privacy-switch-row .row-label { font-size: 12px; }
  .privacy-switch-row .row-desc { line-height: 1.45; }
  .privacy-switch-row .unavailable { color: var(--text-muted); }
  .switch-control { position: relative; flex: 0 0 auto; width: 40px; height: 22px; }
  .switch-control input { position: absolute; inset: 0; z-index: 1; width: 100%; height: 100%; margin: 0; opacity: 0; cursor: pointer; }
  .switch-track { position: absolute; inset: 0; border-radius: 99px; background: rgba(255,255,255,.2); transition: background .16s ease; }
  .switch-track::before { content: ""; position: absolute; top: 3px; left: 3px; width: 16px; height: 16px; border-radius: 50%; background: #fff; transition: transform .16s ease; }
  .switch-control input:checked + .switch-track { background: var(--accent-primary); }
  .switch-control input:checked + .switch-track::before { transform: translateX(18px); }
  .switch-control input:focus-visible + .switch-track { outline: 2px solid var(--accent-primary); outline-offset: 3px; }
  .switch-control input:disabled { cursor: not-allowed; }
  .switch-control input:disabled + .switch-track { opacity: .45; }
  .settings-message { margin: 0; font-size: 12px; color: var(--text-secondary); }
  @media (max-width: 720px) {
    .privacy-security-settings .setting-row { align-items: flex-start; flex-direction: column; }
    .privacy-security-settings .setting-row select { width: 100%; max-width: 260px; }
    .privacy-switch-row { padding-inline: 4px; }
  }
  @media (prefers-reduced-motion: reduce) {
    .switch-track, .switch-track::before { transition: none; }
  }
</style>
