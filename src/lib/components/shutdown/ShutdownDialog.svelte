<script lang="ts">
  import { tick } from "svelte";
  import { errorText, technicalError } from "../../i18n/errors";
  import { t } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import AppIcon from "../icons/AppIcon.svelte";
  import { hasIrreversibleRisk, reviewNeedsAttention } from "../../services/shutdown-skip.js";
  import { shutdownState, shutdownBusy, shutdownError, shutdownVisible, skipExitReview, continueBrowsing, retryShutdown, exitAnyway } from "../../stores/shutdownStore";
  let continueButton: HTMLButtonElement | undefined = $state();
  let dialog: HTMLDivElement | undefined = $state();
  let dontAskAgain = $state(false);
  const exiting = $derived($shutdownState.purpose === "exit" && !$shutdownState.update_version);
  const risky = $derived(hasIrreversibleRisk($shutdownState.review));
  // A review that is confirmed automatically is only "working"; it never shows its (empty) list.
  const asking = $derived($shutdownState.phase === "review" && reviewNeedsAttention($shutdownState, $skipExitReview));
  const failed = $derived($shutdownState.phase === "error" || Boolean($shutdownError));
  const working = $derived(!asking && !failed);
  const canContinue = $derived(!$shutdownBusy && $shutdownState.phase !== "committing");
  const heading = $derived($shutdownState.purpose === "restart_runtime" ? "shutdown.runtime_heading"
    : $shutdownState.update_version ? "shutdown.update_heading" : "shutdown.heading");
  const confirmLabel = $derived($shutdownState.purpose === "restart_runtime" ? "shutdown.restart"
    : $shutdownState.update_version ? "shutdown.install" : "shutdown.exit");
  $effect(() => {
    if (!$shutdownVisible || $shutdownState.phase === "idle") return;
    const targetIsContinue = canContinue;
    void tick().then(() => (targetIsContinue ? continueButton : dialog)?.focus({ preventScroll: true }));
  });
  $effect(() => { if ($shutdownState.phase === "idle") dontAskAgain = false; });
  function host(url: string) {
    try { return new URL(url).hostname; } catch { return ""; }
  }
</script>
{#if $shutdownVisible && $shutdownState.phase !== "idle"}
  <div class="shutdown-backdrop" class:working>
    <div bind:this={dialog} class="shutdown-dialog" class:working role="dialog" aria-modal="true"
      aria-labelledby="shutdown-heading" tabindex="-1" use:focusDialog={{ onEscape: () => { if (canContinue) void continueBrowsing(); } }}
      data-testid="shutdown-dialog" data-view={working ? "working" : failed ? "error" : "review"}>
      {#if working}
        <div class="progress" role="status">
          <span class="spinner" aria-hidden="true"></span>
          <h2 id="shutdown-heading">{$t($shutdownState.phase === "committing" ? "shutdown.saving" : "shutdown.preparing")}</h2>
        </div>
      {:else}
        <header>
          <span class="mark" class:alert={risky || failed}>
            <AppIcon name={risky || failed ? "alert" : $shutdownState.purpose === "restart_runtime" ? "refresh" : "window"} size={20} />
          </span>
          <div class="titles">
            <h2 id="shutdown-heading">{$t(failed ? "shutdown.error_heading" : heading)}</h2>
            {#if asking && !failed}<p class="lede">{$t("shutdown.review")}</p>{/if}
          </div>
        </header>
      {/if}
      {#if failed}
        <div class="failure">
          <p role="alert" class="error">{$shutdownError ? $errorText($shutdownError) : $t("shutdown.save_failed")}</p>
          {#if $shutdownError}<details><summary>{$t("shutdown.details")}</summary><pre>{technicalError($shutdownError)}</pre></details>{/if}
        </div>
      {/if}
      {#if asking && $shutdownState.review}
        {#if $shutdownState.review.tabs.length}
          <section aria-labelledby="shutdown-pages">
            <h3 id="shutdown-pages">{$t("shutdown.section_pages")}</h3>
            <ul class="activity">
              {#each $shutdownState.review.tabs as tab (tab.id)}
                <li class:at-risk={tab.form || tab.native_download}>
                  <div class="page">
                    <span class="page-title">{tab.title || tab.url}</span>
                    {#if host(tab.url)}<span class="page-host">{host(tab.url)}</span>{/if}
                  </div>
                  <div class="badges">
                    {#if tab.form}<span class="badge warn">{$t("shutdown.badge_form")}</span>{/if}
                    {#if tab.native_download}<span class="badge warn">{$t("shutdown.badge_download")}</span>{/if}
                    {#if tab.media}<span class="badge">{$t("shutdown.badge_media")}</span>{/if}
                  </div>
                </li>
              {/each}
            </ul>
          </section>
        {/if}
        {#if $shutdownState.review.downloads.length || $shutdownState.review.native_downloads.length}
          <div class="transfer-notices">
            {#if $shutdownState.review.downloads.length}<p class="note">{$t("shutdown.manual", { count: $shutdownState.review.downloads.length })}</p>{/if}
            {#if $shutdownState.review.native_downloads.length}<p class="note warn">{$t("shutdown.native", { count: $shutdownState.review.native_downloads.length })}</p>{/if}
          </div>
        {/if}
        {#if exiting && !failed}
          <label class="skip">
            <input type="checkbox" data-testid="shutdown-skip" bind:checked={dontAskAgain} disabled={$shutdownBusy} />
            <span><span class="skip-label">{$t("shutdown.dont_ask")}</span><span class="skip-hint">{$t("shutdown.dont_ask_hint")}</span></span>
          </label>
        {/if}
      {/if}
      {#if !working || canContinue}
        <footer>
          <button class:primary={!failed && !working} class:subtle={working} bind:this={continueButton}
            data-testid="shutdown-continue" disabled={!canContinue} onclick={continueBrowsing}>{$t("shutdown.continue")}</button>
          {#if asking && !failed}<button class:danger={risky} data-testid="shutdown-confirm" disabled={$shutdownBusy} onclick={() => exitAnyway(dontAskAgain)}>{$t(confirmLabel)}</button>{/if}
          {#if failed}<button class="primary" data-testid="shutdown-retry" disabled={$shutdownBusy} onclick={retryShutdown}>{$t("shutdown.retry")}</button>{/if}
        </footer>
      {/if}
    </div>
  </div>
{/if}
<style>
  .shutdown-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100000;
    display: grid;
    place-items: center;
    padding: 16px;
    background: rgb(0 0 0 / 38%);
    backdrop-filter: blur(4px);
  }
  .shutdown-backdrop.working { background: rgb(0 0 0 / 26%); backdrop-filter: none; }

  .shutdown-dialog {
    --warning-ink: #ffb3a4;
    --warning-fill: color-mix(in srgb, var(--warning-ink) 11%, transparent);
    --warning-border: color-mix(in srgb, var(--warning-ink) 28%, transparent);
    box-sizing: border-box;
    display: grid;
    gap: 18px;
    width: min(100%, 490px);
    max-height: calc(100dvh - 32px);
    overflow: auto;
    padding: clamp(16px, 4vw, 24px);
    border: var(--modal-surface-border);
    border-radius: 18px;
    background: var(--modal-surface-bg);
    backdrop-filter: var(--modal-surface-filter);
    color: var(--text-primary);
    color-scheme: var(--settings-control-scheme, dark);
    box-shadow: var(--modal-surface-shadow);
  }
  :global(html[data-theme="light"]) .shutdown-dialog { --warning-ink: #8d2920; }
  .shutdown-dialog.working { width: min(100%, 360px); gap: 10px; padding: 16px 18px; border-radius: 14px; }
  .shutdown-dialog:focus { outline: none; }

  .progress { display: flex; align-items: center; gap: 12px; }
  .progress h2 { font-size: 14px; font-weight: 600; line-height: 1.4; }
  .spinner {
    box-sizing: border-box;
    flex: none;
    width: 18px;
    height: 18px;
    border: 2px solid var(--border-subtle);
    border-top-color: var(--text-primary);
    border-radius: 50%;
    animation: shutdown-spin 850ms linear infinite;
  }
  @keyframes shutdown-spin { to { transform: rotate(1turn); } }
  @media (prefers-reduced-motion: reduce) { .spinner { animation: none; } }

  header { display: flex; gap: 12px; align-items: flex-start; }
  .mark {
    flex: none;
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border: 1px solid var(--border-subtle);
    border-radius: 10px;
    background: var(--bg-glass-hover);
    color: var(--text-secondary);
  }
  .mark.alert { border-color: var(--warning-border); background: var(--warning-fill); color: var(--warning-ink); }
  .titles { min-width: 0; display: grid; gap: 5px; }
  h2 { margin: 0; font-size: 20px; line-height: 1.25; text-wrap: balance; }
  h3 { margin: 0 0 8px; font-size: 12px; font-weight: 600; color: var(--text-secondary); }
  .lede, .note, .error, .skip-hint { margin: 0; font-size: 13px; line-height: 1.45; }
  .lede, .note, .skip-hint { color: var(--text-secondary); }

  .activity {
    max-height: 210px;
    margin: 0;
    padding: 0;
    overflow: auto;
    list-style: none;
    border: 1px solid var(--border-subtle);
    border-radius: 12px;
  }
  li {
    display: flex;
    flex-wrap: wrap;
    gap: 7px 12px;
    align-items: center;
    justify-content: space-between;
    padding: 9px 11px;
    background: var(--bg-glass-hover);
    overflow-wrap: anywhere;
  }
  li + li { border-top: 1px solid var(--border-subtle); }
  li.at-risk { background: var(--warning-fill); }
  .page { flex: 1 1 145px; min-width: 0; display: grid; gap: 3px; }
  .page-title { overflow: hidden; display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; font-size: 13px; font-weight: 600; }
  .page-host { color: var(--text-secondary); font-size: 12px; }
  .badges { display: flex; flex-wrap: wrap; gap: 5px; }
  .badge { padding: 3px 7px; border-radius: 6px; background: var(--bg-glass-active); color: var(--text-secondary); font-size: 11px; }
  .badge.warn { background: var(--warning-fill); color: var(--warning-ink); }

  .transfer-notices, .failure { display: grid; gap: 8px; }
  .note, .failure { padding: 10px 12px; border: 1px solid var(--border-subtle); border-radius: 10px; background: var(--bg-glass-hover); }
  .note.warn, .failure { border-color: var(--warning-border); background: var(--warning-fill); }
  .error { color: var(--text-primary); }
  summary { width: fit-content; cursor: pointer; color: var(--text-secondary); font-size: 12px; }
  pre { max-height: 120px; margin: 8px 0 0; overflow: auto; font-size: 12px; white-space: pre-wrap; overflow-wrap: anywhere; }

  .skip { display: flex; gap: 10px; align-items: flex-start; cursor: pointer; }
  .skip input { flex: none; width: 16px; height: 16px; margin: 1px 0 0; accent-color: var(--accent-primary); }
  .skip-label { display: block; font-size: 13px; font-weight: 600; }
  .skip-hint { display: block; margin-top: 4px; font-size: 12px; }

  footer { display: flex; flex-wrap: wrap; gap: 8px; justify-content: flex-end; padding-top: 12px; border-top: 1px solid var(--border-subtle); }
  .working footer { padding-top: 0; border-top: 0; }
  button {
    min-height: 36px;
    padding: 7px 12px;
    border: 1px solid var(--border-subtle);
    border-radius: 9px;
    background: var(--bg-glass-hover);
    color: inherit;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  button:enabled:hover { background: var(--bg-glass-active); }
  button:disabled { opacity: .55; cursor: default; }
  button.primary { border-color: transparent; background: var(--accent-primary); color: var(--accent-foreground); }
  button.primary:enabled:hover { background: var(--accent-primary); filter: brightness(1.08); }
  button.danger { border-color: var(--warning-border); background: var(--warning-fill); color: var(--warning-ink); }
  button.danger:enabled:hover { background: var(--warning-border); }
  button.subtle { min-height: 30px; border-color: transparent; background: transparent; color: var(--text-secondary); }
  :is(button, input, summary):focus-visible { outline: 2px solid var(--text-primary); outline-offset: 2px; }
  @media (max-width: 420px) {
    .shutdown-dialog:not(.working) footer { display: grid; }
  }
</style>
