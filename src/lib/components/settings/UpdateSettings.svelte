<script lang="ts">
  import { onMount } from "svelte";
  import { loadProfiles, profiles } from "../../stores/profilesStore";
  import RuntimeSettings from "./RuntimeSettings.svelte";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import AppIcon from "../icons/AppIcon.svelte";
  import { t, formatDate, formatNumber } from "../../i18n";
  import { updateState, updateBusy, updateActionError, checkUpdates, selectUpdateChannel, startUpdateDownload, startUpdateInstall, setAutoUpdateEnabled } from "../../stores/updatesStore";
  import type { UpdateChannel } from "../../types/updates";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { createNewTab } from "../../services/tauri-bridge";
  import { REPOSITORY_URL } from "../../services/project-links.js";
  import { logIgnored } from "../../services/ignored-error";

  const active = $derived($updateBusy || ["checking", "downloading", "installing"].includes($updateState.status));
  const status = $derived.by(() => {
    switch ($updateState.status) {
      case "checking": return $t("settings.checking_github_releases_71ad");
      case "up_to_date": return $t("settings.no_newer_version_is_available_for_this_channel_d0ae");
      case "available": return $t("settings.an_update_is_available_da18");
      case "downloading": return $t("settings.downloading_and_verifying_the_update_9dbf");
      case "ready": return $updateState.auto_update && $updateState.staged ? $t("settings.update_ready_next_start") : $t("settings.the_signed_update_is_ready_to_install_5e30");
      case "installing": return $t("settings.saving_your_session_and_starting_the_installer_028b");
      case "error": return $t("settings.the_update_could_not_be_completed_5ba6");
      case "unavailable": return $t("common.updates_unavailable");
      default: return $t("settings.check_for_a_newer_version_of_spark_browser_078c");
    }
  });
  const statusTone = $derived.by<"ok" | "accent" | "busy" | "error" | "neutral">(() => {
    switch ($updateState.status) {
      case "up_to_date": return "ok";
      case "available":
      case "ready": return "accent";
      case "checking":
      case "downloading":
      case "installing": return "busy";
      case "error": return "error";
      default: return "neutral";
    }
  });
  const downloadPercent = $derived(
    $updateState.total && $updateState.total > 0
      ? Math.min(100, Math.round(($updateState.received / $updateState.total) * 100))
      : null
  );
  function openRepository() { void createNewTab(REPOSITORY_URL).catch(logIgnored); }
  function date(timestamp: number) { return $formatDate(timestamp * 1000, { dateStyle: "medium", timeStyle: "short" }); }
  // A profile other than the Default one does not install updates: say so instead of a dead button.
  const otherProfile = $derived(!!$profiles && $profiles.current !== "default");
  onMount(() => { void loadProfiles(); });
</script>

<section class="setting-group about-updates" aria-labelledby="updates-heading">
  <h2 id="updates-heading" class="group-title">{$t("settings.about_updates_a1d4")}</h2>

  <div class="hero-card" data-tone={statusTone}>
    <div class="hero-top">
      <div class="hero-brand">
        <div class="brand-mark" aria-hidden="true">
          <AppIcon name="zap" size={22} />
        </div>
        <div class="brand-copy">
          <div class="brand-title-row">
            <p class="version">Spark</p>
            <span class="version-pill">v{$updateState.current_version}</span>
            <span class="channel-pill" class:beta={$updateState.channel === "beta"}>
              {$updateState.channel === "beta" ? $t("settings.beta_7033") : $t("settings.stable_90ee")}
            </span>
          </div>
          <div class="status-line">
            <span class="status-dot" class:pulsing={statusTone === "busy"} aria-hidden="true"></span>
            <p class="status-text" role="status" aria-live="polite">{status}</p>
          </div>
        </div>
      </div>

      <div class="actions">
        <button
          data-testid="check-for-updates"
          class="pill-action action-btn"
          class:primary-action={!$updateState.update}
          disabled={!$updateState.supported || active}
          onclick={() => { void checkUpdates(); }}
        >
          <span class="btn-icon" class:spinning={$updateState.status === "checking"} aria-hidden="true">
            <AppIcon name="refresh" size={14} />
          </span>
          <span>{$t("settings.check_for_updates_f26f")}</span>
        </button>

        {#if $updateState.update}
          {#if $updateState.status === "ready"}
            <button class="pill-action action-btn primary-action" disabled={active} onclick={() => { void startUpdateInstall(); }}>
              <span class="btn-icon" aria-hidden="true"><AppIcon name="refresh" size={14} /></span>
              <span>{$updateState.auto_update ? $t("settings.restart_to_update") : $t("settings.install_and_restart_1b99")}</span>
            </button>
          {:else}
            <button class="pill-action action-btn primary-action" disabled={active} onclick={() => { void startUpdateDownload(); }}>
              <span class="btn-icon" aria-hidden="true"><AppIcon name="download" size={14} /></span>
              <span>{$t("settings.download_update_4f49")}</span>
            </button>
          {/if}
        {/if}
      </div>
    </div>

    {#if $updateState.status === "downloading"}
      <div class="download-progress">
        <progress aria-label={$t("settings.update_download_1c20")} max={$updateState.total ?? 1} value={$updateState.total ? $updateState.received : undefined}></progress>
        <div class="download-meta">
          <span>
            {$formatNumber($updateState.received / 1048576, { minimumFractionDigits: 1, maximumFractionDigits: 1 })} MB{$updateState.total ? ` / ${$formatNumber($updateState.total / 1048576, { minimumFractionDigits: 1, maximumFractionDigits: 1 })} MB` : ""}
          </span>
          {#if downloadPercent !== null}
            <span class="download-percent">{downloadPercent}%</span>
          {/if}
        </div>
      </div>
    {/if}

    {#if $updateState.channel === "stable" && $updateState.current_version.includes("-beta.")}
      <div class="inline-notice">
        <span class="notice-icon" aria-hidden="true"><AppIcon name="alert" size={14} /></span>
        <p>{$t("settings.you_will_stay_on_this_beta_until_a_newer_stable_release_is_available_cha_2ff7")}</p>
      </div>
    {/if}

    {#if otherProfile}
      <div class="inline-notice">
        <span class="notice-icon" aria-hidden="true"><AppIcon name="user" size={14} /></span>
        {#if otherProfile}<p role="note" data-testid="updates-other-profile">{$t("profiles.updates_note")}</p>{/if}
      </div>
    {/if}

    {#if $updateState.error || $updateActionError}
      <div class="error-banner">
        <ErrorMessage error={$updateActionError || $updateState.error} />
      </div>
    {/if}

    {#if $updateState.retry_after}
      <div class="inline-notice">
        <span class="notice-icon" aria-hidden="true"><AppIcon name="clock" size={14} /></span>
        <p>{$t("settings.try_again_after_41f4")} {date($updateState.retry_after)}.</p>
      </div>
    {/if}

    {#if $updateState.update}
      <div class="release-notes">
        <div class="release-notes-header">
          <span class="release-badge">{$t("settings.version_dd16")} {$updateState.update.version}</span>
        </div>
        <pre>{$updateState.update.notes || $t("common.no_release_notes")}</pre>
      </div>
    {/if}

    <div class="hero-footer">
      <span class="last-check">
        <AppIcon name="clock" size={13} />
        <span>{$t("settings.last_check_4941")} <strong>{$updateState.last_checked ? date($updateState.last_checked) : $t("settings.not_checked_yet_881d")}</strong></span>
      </span>
    </div>
  </div>

  <div class="preferences-stack">
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

    <div class="setting-row" id="setting-auto-update" tabindex="-1">
      <div class="row-text">
        <label for="auto-update" class="row-label">{$t("settings.auto_update")}</label>
        <span class="row-desc">{$t("settings.auto_update_description")}</span>
      </div>
      <input id="auto-update" type="checkbox" checked={$updateState.auto_update} disabled={!isTauriRuntime() || $updateBusy || $updateState.status === "installing"}
        onchange={event => { void setAutoUpdateEnabled(event.currentTarget.checked); }} />
    </div>

    <p class="policy-note">{$t("settings.updates_are_checked_when_the_browser_opens_downloading_and_installing_re_d689")}</p>
  </div>

  <div class="info-grid">
    <article class="info-card repository" aria-labelledby="repository-heading">
      <header class="info-card-header">
        <span class="info-tile" aria-hidden="true">
          <AppIcon name="code" size={17} />
        </span>
        <div class="info-titles">
          <h3 id="repository-heading">{$t("settings.repository_title")}</h3>
          <p>{$t("settings.repository_description")}</p>
        </div>
      </header>

      <div class="repository-url-box">
        <p class="repository-url">{REPOSITORY_URL}</p>
      </div>

      <div class="info-card-footer">
        <button class="pill-action action-btn" disabled={!isTauriRuntime()} onclick={openRepository}>
          <span>{$t("settings.repository_open")}</span>
          <span class="external-arrow" aria-hidden="true">↗</span>
        </button>
      </div>
    </article>

    <RuntimeSettings />
  </div>
</section>

<style>
  .about-updates {
    gap: 16px;
  }

  .hero-card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px 20px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--settings-row-border);
    background:
      radial-gradient(
        460px 180px at 0% 0%,
        color-mix(in srgb, var(--accent-primary) 11%, transparent),
        transparent 72%
      ),
      var(--settings-row-bg);
    overflow: hidden;
  }

  .hero-card[data-tone="ok"] {
    background:
      radial-gradient(
        460px 180px at 0% 0%,
        color-mix(in srgb, var(--status-on) 12%, transparent),
        transparent 72%
      ),
      var(--settings-row-bg);
  }

  .hero-card[data-tone="error"] {
    background:
      radial-gradient(
        460px 180px at 0% 0%,
        color-mix(in srgb, var(--download-status-error) 12%, transparent),
        transparent 72%
      ),
      var(--settings-row-bg);
  }

  .hero-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .hero-brand {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
    flex: 1 1 280px;
  }

  .brand-mark {
    flex: 0 0 auto;
    width: 46px;
    height: 46px;
    display: grid;
    place-items: center;
    border-radius: 13px;
    background: var(--accent-gradient);
    color: var(--accent-foreground);
  }

  .brand-copy {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .brand-title-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }

  .version {
    margin: 0;
    color: var(--text-primary);
    font-size: 18px;
    font-weight: 750;
    letter-spacing: -0.02em;
    line-height: 1.1;
  }

  .version-pill,
  .channel-pill {
    display: inline-flex;
    align-items: center;
    height: 22px;
    padding: 0 9px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 650;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }

  .version-pill {
    border: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--text-primary) 7%, transparent);
    color: var(--text-primary);
  }

  .channel-pill {
    border: 1px solid var(--border-subtle);
    background: transparent;
    color: var(--text-secondary);
  }

  .channel-pill.beta {
    border-color: color-mix(in srgb, var(--accent-primary) 42%, transparent);
    background: color-mix(in srgb, var(--accent-primary) 15%, transparent);
    color: var(--accent-primary);
  }

  .status-line {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .status-dot {
    flex: 0 0 auto;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-muted);
  }

  .hero-card[data-tone="ok"] .status-dot {
    background: var(--status-on);
    box-shadow: 0 0 0 3px var(--status-on-bg);
  }

  .hero-card[data-tone="accent"] .status-dot,
  .hero-card[data-tone="busy"] .status-dot {
    background: var(--accent-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 18%, transparent);
  }

  .hero-card[data-tone="error"] .status-dot {
    background: var(--download-status-error);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--download-status-error) 18%, transparent);
  }

  .status-dot.pulsing {
    animation: status-pulse 1.5s ease-in-out infinite;
  }

  .status-text {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12.5px;
    line-height: 1.45;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin: 0;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    min-height: 34px;
    padding: 0 15px;
    transition: background 0.15s ease, border-color 0.15s ease, color 0.15s ease, opacity 0.15s ease, transform 0.12s ease;
  }

  .action-btn:hover:not(:disabled) {
    border-color: color-mix(in srgb, var(--accent-primary) 45%, var(--border-subtle));
    background: color-mix(in srgb, var(--text-primary) 9%, transparent);
    color: var(--text-primary);
  }

  .action-btn:active:not(:disabled) {
    transform: translateY(0.5px);
  }

  .action-btn:disabled {
    opacity: 0.48;
    cursor: not-allowed;
  }

  .action-btn.primary-action {
    border-color: transparent;
    background: var(--accent-gradient);
    color: var(--accent-foreground);
  }

  .action-btn.primary-action:hover:not(:disabled) {
    filter: brightness(1.06);
    border-color: transparent;
    color: var(--accent-foreground);
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
  }

  .btn-icon.spinning {
    animation: icon-spin 1s linear infinite;
  }

  .download-progress {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 14px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--text-primary) 3%, transparent);
  }

  progress {
    width: 100%;
    height: 6px;
    appearance: none;
    -webkit-appearance: none;
    border: 0;
    border-radius: 999px;
    overflow: hidden;
    background: color-mix(in srgb, var(--text-primary) 12%, transparent);
  }

  progress::-webkit-progress-bar {
    background: color-mix(in srgb, var(--text-primary) 12%, transparent);
    border-radius: 999px;
  }

  progress::-webkit-progress-value {
    background: var(--accent-gradient);
    border-radius: 999px;
    transition: width 0.2s ease;
  }

  .download-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    color: var(--text-secondary);
    font-size: 11.5px;
    font-variant-numeric: tabular-nums;
  }

  .download-percent {
    color: var(--text-primary);
    font-weight: 650;
  }

  .inline-notice {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    border: 1px solid color-mix(in srgb, var(--accent-primary) 28%, var(--border-subtle));
    background: color-mix(in srgb, var(--accent-primary) 8%, transparent);
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.5;
  }

  .inline-notice p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.5;
  }

  .notice-icon {
    flex: 0 0 auto;
    margin-top: 2px;
    color: var(--accent-primary);
  }

  .error-banner {
    padding: 10px 12px;
    border-radius: var(--radius-md);
    border: 1px solid color-mix(in srgb, var(--download-status-error) 36%, transparent);
    background: color-mix(in srgb, var(--download-status-error) 10%, transparent);
  }

  .release-notes {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 14px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--text-primary) 3%, transparent);
  }

  .release-notes-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .release-badge {
    display: inline-flex;
    align-items: center;
    padding: 3px 9px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent-primary) 16%, transparent);
    color: var(--accent-primary);
    font-size: 11px;
    font-weight: 700;
  }

  pre {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.6;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font-family: inherit;
    max-height: 240px;
    overflow: auto;
  }

  .hero-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border-subtle);
  }

  .last-check {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    color: var(--text-muted);
    font-size: 11.5px;
    line-height: 1.4;
  }

  .last-check strong {
    color: var(--text-secondary);
    font-weight: 600;
  }

  .preferences-stack {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .policy-note {
    margin: 2px 4px 0;
    color: var(--text-muted);
    font-size: 11.5px;
    line-height: 1.55;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 12px;
    align-items: stretch;
  }

  .info-card {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 16px;
    border-radius: var(--radius-md);
    border: 1px solid var(--settings-row-border);
    background: var(--settings-row-bg);
    min-width: 0;
  }

  .info-card-header {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    min-width: 0;
  }

  .info-tile {
    flex: 0 0 auto;
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    background: color-mix(in srgb, var(--accent-primary) 15%, transparent);
    color: var(--accent-primary);
  }

  .info-titles {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .info-titles h3 {
    margin: 0;
    font-size: 13.5px;
    font-weight: 650;
    color: var(--text-primary);
    line-height: 1.3;
  }

  .info-titles p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.5;
  }

  .repository-url-box {
    padding: 8px 11px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--text-primary) 3.5%, transparent);
  }

  .repository-url {
    margin: 0;
    color: var(--text-secondary);
    font-family: ui-monospace, "Cascadia Code", "SFMono-Regular", Menlo, Consolas, monospace;
    font-size: 11.5px;
    line-height: 1.4;
    user-select: text;
    overflow-wrap: anywhere;
  }

  .info-card-footer {
    margin-top: auto;
    display: flex;
    align-items: center;
    justify-content: flex-start;
  }

  .external-arrow {
    font-size: 13px;
    line-height: 1;
    opacity: 0.8;
  }

  @keyframes status-pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.45; transform: scale(0.85); }
  }

  @keyframes icon-spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @media (prefers-reduced-motion: reduce) {
    .status-dot.pulsing,
    .btn-icon.spinning {
      animation: none;
    }
    .action-btn,
    progress::-webkit-progress-value {
      transition: none;
    }
  }
</style>
