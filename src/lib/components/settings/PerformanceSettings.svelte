<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import { t } from "../../i18n";
import {
  autoHibernateMode,
  residentTabLimit,
  hibernationExemptions,
  askDownloadDestination,
  hibernateOtherWorkspaces,
  setHibernateOtherWorkspaces,
  setResidentTabLimit,
  setAutoHibernateMode,
  setHibernationExemptions,
  setAskDownloadDestination,
} from "../../stores/settingsStore";
import { addHibernationDomain } from "../../services/hibernation-domains.js";
import { graphicsBusy, graphicsError, graphicsStatus, refreshGraphics, restartForGraphics, setHardwareAcceleration } from "../../stores/graphicsStore";
import { skipExitReview, setSkipExitReview } from "../../stores/shutdownStore";
import ErrorMessage from "../status/ErrorMessage.svelte";
const autoHibernateOptions = [5, 10, 30, "auto", "off"] as const;
let domainInput = $state("");
let domainError = $state(false);

async function addDomain() {
  const next = addHibernationDomain(domainInput, $hibernationExemptions);
  if (!next) { domainError = true; return; }
  domainError = false;
  try { await setHibernationExemptions(next); domainInput = ""; }
  catch { domainError = true; }
}

$effect(() => { void refreshGraphics(); });

async function removeDomain(domain: string) {
  await setHibernationExemptions($hibernationExemptions.filter(item => item !== domain));
}
</script>

<section class="setting-group" aria-labelledby="performance-heading">
  <h2 class="group-title" id="performance-heading"><AppIcon name="rocket" /> {$t("settings.performance_privacy_41ef")}</h2>

  <div class="setting-row" id="setting-hardware-acceleration" tabindex="-1" data-testid="hardware-acceleration">
    <div class="row-text">
      <span class="row-label">{$t("settings.hardware_acceleration")}</span>
      <span class="row-desc">{$t("settings.hardware_acceleration_desc")}</span>
      {#if $graphicsStatus.restartRequired}
        <span class="row-desc restart-note" role="status" data-testid="hardware-acceleration-restart-note">{$t("settings.hardware_acceleration_restart")}</span>
      {/if}
    </div>
    <div class="gpu-controls">
      {#if $graphicsStatus.restartRequired}
        <button class="pill-action" type="button" data-testid="hardware-acceleration-restart" disabled={$graphicsBusy} onclick={restartForGraphics}>{$t("settings.hardware_acceleration_restart_now")}</button>
      {/if}
      <label class="gpu-switch">
        <input type="checkbox" role="switch" data-testid="hardware-acceleration-switch" aria-label={$t("settings.hardware_acceleration")} checked={$graphicsStatus.enabled} disabled={$graphicsBusy} onchange={event => setHardwareAcceleration(event.currentTarget.checked)} />
        <span class="gpu-track" aria-hidden="true"></span>
      </label>
    </div>
  </div>
  <ErrorMessage error={$graphicsError} />

  <div class="setting-row" id="setting-resident-limit" tabindex="-1">
    <div class="row-text"><span class="row-label">{$t("settings.resident_tab_limit_9a94")}</span><span class="row-desc">{$t("settings.keep_up_to_this_many_webviews_awake_active_pinned_loading_media_download_9f92")}</span></div>
    <input type="number" min="1" max="100" step="1" disabled={$autoHibernateMode === "off"} value={$residentTabLimit} aria-label={$t("settings.resident_tab_limit_e214")} onchange={event => setResidentTabLimit(Number(event.currentTarget.value))} />
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

  <div class="setting-row" id="setting-hibernate-other-workspaces" tabindex="-1">
    <div class="row-text"><span class="row-label">{$t("settings.hibernate_other_workspaces")}</span><span class="row-desc">{$t("settings.hibernate_other_workspaces_description")}</span></div>
    <input type="checkbox" checked={$hibernateOtherWorkspaces} aria-label={$t("settings.hibernate_other_workspaces")} onchange={event => setHibernateOtherWorkspaces(event.currentTarget.checked)} />
  </div>

  <div class="setting-row" id="setting-hibernation-exceptions" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.hibernate_exceptions")}</span>
      <span class="row-desc">{$t("settings.hibernate_exceptions_description")}</span>
    </div>
    <div class="domain-editor">
      <form onsubmit={event => { event.preventDefault(); void addDomain(); }}>
        <input bind:value={domainInput} type="text" autocomplete="off" spellcheck="false" placeholder={$t("settings.domain_placeholder")} aria-label={$t("settings.domain_placeholder")} />
        <button type="submit" disabled={!domainInput.trim()}>{$t("settings.add_domain")}</button>
      </form>
      {#if domainError}<span class="domain-error" role="alert">{$t("settings.invalid_domain")}</span>{/if}
      {#if $hibernationExemptions.length}
        <ul>{#each $hibernationExemptions as domain (domain)}<li><span>{domain}</span><button type="button" aria-label={$t("settings.remove_domain", { domain })} onclick={() => void removeDomain(domain)}>×</button></li>{/each}</ul>
      {/if}
    </div>
  </div>

  <div class="setting-row" id="setting-download-destination" tabindex="-1">
    <div class="row-text"><span class="row-label">{$t("settings.ask_download_destination")}</span><span class="row-desc">{$t("settings.ask_download_destination_description")}</span></div>
    <input type="checkbox" checked={$askDownloadDestination} aria-label={$t("settings.ask_download_destination")} onchange={event => void setAskDownloadDestination(event.currentTarget.checked).catch(() => {})} />
  </div>

  <div class="setting-row" id="setting-skip-exit-review" tabindex="-1" data-testid="skip-exit-review">
    <div class="row-text"><span class="row-label">{$t("settings.skip_exit_review")}</span><span class="row-desc">{$t("settings.skip_exit_review_description")}</span></div>
    <input type="checkbox" checked={$skipExitReview} aria-label={$t("settings.skip_exit_review")} onchange={event => setSkipExitReview(event.currentTarget.checked)} />
  </div>
</section>

<style>
  .domain-editor { display: grid; gap: 6px; min-width: min(320px, 48%); }
  .domain-editor form { display: flex; gap: 6px; }
  .domain-editor input { min-width: 0; width: 100%; }
  .domain-editor button, .domain-editor li button { flex: 0 0 auto; }
  .domain-editor ul { display: flex; flex-wrap: wrap; gap: 5px; list-style: none; padding: 0; margin: 0; }
  .domain-editor li { display: flex; align-items: center; gap: 5px; border: 1px solid var(--border-subtle); border-radius: 6px; padding: 2px 5px 2px 8px; }
  .domain-editor li button { border: 0; background: transparent; color: inherit; }
  .domain-error { color: var(--status-danger, #e55); }
  @media (max-width: 700px) { .domain-editor { width: 100%; min-width: 0; } }
  .restart-note { color: #fcd34d; }
  .gpu-controls { display: flex; flex: 0 0 auto; align-items: center; gap: 12px; }
  .gpu-switch { position: relative; flex: 0 0 auto; width: 40px; height: 22px; cursor: pointer; }
  .gpu-switch input { position: absolute; inset: 0; z-index: 1; width: 100%; height: 100%; margin: 0; opacity: 0; cursor: inherit; }
  .gpu-track { position: absolute; inset: 0; border-radius: 99px; background: rgba(255, 255, 255, 0.2); transition: background 0.16s ease; }
  .gpu-track::before { content: ""; position: absolute; top: 3px; left: 3px; width: 16px; height: 16px; border-radius: 50%; background: #fff; transition: transform 0.16s ease; }
  .gpu-switch input:checked + .gpu-track { background: var(--accent-primary); }
  .gpu-switch input:checked + .gpu-track::before { transform: translateX(18px); }
  .gpu-switch input:focus-visible + .gpu-track { outline: 2px solid var(--accent-primary); outline-offset: 3px; }
  @media (prefers-reduced-motion: reduce) { .gpu-track, .gpu-track::before { transition: none; } }
</style>
