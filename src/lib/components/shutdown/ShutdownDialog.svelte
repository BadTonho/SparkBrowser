<script lang="ts">
  import { tick } from "svelte";
  import { errorText, technicalError } from "../../i18n/errors";
  import { t } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { hasIrreversibleRisk } from "../../services/shutdown-skip.js";
  import { shutdownState, shutdownBusy, shutdownError, continueBrowsing, retryShutdown, exitAnyway } from "../../stores/shutdownStore";
  let continueButton: HTMLButtonElement | undefined = $state();
  let dontAskAgain = $state(false);
  const exiting = $derived($shutdownState.purpose === "exit" && !$shutdownState.update_version);
  const risky = $derived(hasIrreversibleRisk($shutdownState.review));
  $effect(() => {
    const phase = $shutdownState.phase;
    if (!$shutdownBusy && phase !== "idle" && phase !== "committing") {
      void tick().then(() => continueButton?.focus({ preventScroll: true }));
    }
  });
  $effect(() => { if ($shutdownState.phase === "idle") dontAskAgain = false; });
  function host(url: string) {
    try { return new URL(url).hostname; } catch { return ""; }
  }
</script>
{#if $shutdownState.phase !== "idle"}
  <div class="shutdown-backdrop">
    <div class="shutdown-dialog" role="dialog" aria-modal="true" aria-label={$t($shutdownState.purpose === "restart_runtime" ? "shutdown.runtime_heading" : "shutdown.heading")} tabindex="-1" use:focusDialog={{ onEscape: continueBrowsing }} data-testid="shutdown-dialog">
      <header>
        <span class="mark" class:alert={risky} aria-hidden="true">!</span>
        <div class="titles">
          <h2>{$t($shutdownState.purpose === "restart_runtime" ? "shutdown.runtime_heading" : $shutdownState.update_version ? "shutdown.update_heading" : "shutdown.heading")}</h2>
          {#if $shutdownState.phase === "review" && $shutdownState.review}<p class="lede">{$t("shutdown.review")}</p>{/if}
        </div>
      </header>
      {#if $shutdownState.phase === "review" && $shutdownState.review}
        {#if $shutdownState.review.tabs.length}
          <section aria-labelledby="shutdown-pages">
            <h3 id="shutdown-pages">{$t("shutdown.section_pages")}</h3>
            <ul class="activity">
              {#each $shutdownState.review.tabs as tab}
                <li>
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
        {#if $shutdownState.review.downloads.length}<p class="note">{$t("shutdown.manual", { count: $shutdownState.review.downloads.length })}</p>{/if}
        {#if $shutdownState.review.native_downloads.length}<p class="note warn">{$t("shutdown.native", { count: $shutdownState.review.native_downloads.length })}</p>{/if}
        {#if exiting && !$shutdownError}
          <label class="skip">
            <input type="checkbox" data-testid="shutdown-skip" bind:checked={dontAskAgain} disabled={$shutdownBusy} />
            <span><span class="skip-label">{$t("shutdown.dont_ask")}</span><span class="skip-hint">{$t("shutdown.dont_ask_hint")}</span></span>
          </label>
        {/if}
      {:else}<p role="status" class="lede">{$t($shutdownState.phase === "committing" ? "shutdown.saving" : "shutdown.preparing")}</p>{/if}
      {#if $shutdownError}<p role="alert" class="error">{$errorText($shutdownError)}</p><details><summary>{$t("shutdown.details")}</summary><pre>{technicalError($shutdownError)}</pre></details>{/if}
      <footer>
        <button class="primary" bind:this={continueButton} data-testid="shutdown-continue" disabled={$shutdownBusy || $shutdownState.phase === "committing"} onclick={continueBrowsing}>{$t("shutdown.continue")}</button>
        {#if $shutdownState.phase === "review" && !$shutdownError}<button class="danger" data-testid="shutdown-confirm" disabled={$shutdownBusy} onclick={() => exitAnyway(dontAskAgain)}>{$t($shutdownState.purpose === "restart_runtime" ? "shutdown.restart" : $shutdownState.update_version ? "shutdown.install" : "shutdown.exit")}</button>{/if}
        {#if $shutdownError}<button data-testid="shutdown-retry" disabled={$shutdownBusy} onclick={retryShutdown}>{$t("shutdown.retry")}</button>{/if}
      </footer>
    </div>
  </div>
{/if}
<style>
  .shutdown-backdrop { position: fixed; inset: 0; z-index: 100000; display: grid; place-items: center; background: #0008; backdrop-filter: blur(8px); }
  .shutdown-dialog { display: grid; gap: 16px; width: min(560px, 92vw); max-height: 82vh; overflow: auto; padding: 24px; border: 1px solid var(--border-color, #555); border-radius: 16px; background: var(--bg-primary, #202024); color: var(--text-primary, #fff); box-shadow: 0 24px 60px #0008; }
  header { display: flex; gap: 14px; align-items: flex-start; }
  .mark { flex: none; display: grid; place-items: center; width: 36px; height: 36px; border-radius: 50%; background: color-mix(in srgb, var(--accent-primary, #6e8efb) 22%, transparent); color: var(--accent-primary, #6e8efb); font-weight: 700; }
  .mark.alert { background: #e5534b2e; color: #ff8a82; }
  .titles { min-width: 0; display: grid; gap: 4px; }
  h2 { margin: 0; font-size: 20px; line-height: 1.25; }
  h3 { margin: 0 0 8px; font-size: 12px; font-weight: 600; letter-spacing: .04em; text-transform: uppercase; color: var(--text-muted, #9a9aa6); }
  .lede { margin: 0; color: var(--text-secondary, #c3c3cc); font-size: 14px; line-height: 1.45; }
  .activity { display: grid; gap: 8px; max-height: 230px; margin: 0; padding: 0; overflow: auto; list-style: none; }
  li { display: flex; gap: 12px; align-items: center; justify-content: space-between; padding: 10px 12px; border: 1px solid var(--border-subtle, #ffffff1f); border-radius: 10px; background: var(--bg-glass-hover, #ffffff0a); overflow-wrap: anywhere; }
  .page { min-width: 0; display: grid; gap: 2px; }
  .page-title { overflow: hidden; display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; font-size: 14px; }
  .page-host { color: var(--text-muted, #9a9aa6); font-size: 12px; }
  .badges { flex: none; display: flex; flex-wrap: wrap; gap: 6px; justify-content: flex-end; }
  .badge { padding: 3px 8px; border-radius: 999px; background: #ffffff14; color: var(--text-secondary, #c3c3cc); font-size: 11px; white-space: nowrap; }
  .badge.warn { background: #e5534b2e; color: #ffa59f; }
  .note { margin: 0; padding: 10px 12px; border-left: 3px solid var(--border-subtle, #ffffff33); border-radius: 4px; background: #ffffff08; color: var(--text-secondary, #c3c3cc); font-size: 13px; line-height: 1.45; }
  .note.warn { border-left-color: #e5534b; }
  .skip { display: flex; gap: 10px; align-items: flex-start; padding-top: 4px; cursor: pointer; }
  .skip input { margin-top: 3px; accent-color: var(--accent-primary, #6e8efb); }
  .skip-label { display: block; font-size: 14px; }
  .skip-hint { display: block; margin-top: 2px; color: var(--text-muted, #9a9aa6); font-size: 12px; line-height: 1.4; }
  .error { margin: 0; color: #ff8a82; }
  pre { overflow: auto; }
  footer { display: flex; flex-wrap: wrap; gap: 12px; justify-content: flex-end; margin-top: 4px; }
  button { padding: 10px 16px; border: 1px solid var(--border-subtle, #ffffff33); border-radius: 8px; background: transparent; color: inherit; cursor: pointer; font: inherit; }
  button:hover:not(:disabled) { background: var(--bg-glass-hover, #ffffff14); }
  button:disabled { opacity: .55; cursor: default; }
  button.primary { border-color: transparent; background: var(--accent-primary, #6e8efb); color: var(--accent-foreground, #fff); }
  button.primary:hover:not(:disabled) { background: var(--accent-primary, #6e8efb); filter: brightness(1.08); }
  button.danger { border-color: #e5534b88; color: #ff8a82; }
  button.danger:hover:not(:disabled) { background: #e5534b1f; }
  button:focus-visible, input:focus-visible { outline: 2px solid var(--accent-primary, #aaa6ed); outline-offset: 2px; }
</style>
