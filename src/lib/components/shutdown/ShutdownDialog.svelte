<script lang="ts">
  import { tick } from "svelte";
  import { errorText, technicalError } from "../../i18n/errors";
  import { t } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { shutdownState, shutdownBusy, shutdownError, continueBrowsing, retryShutdown, exitAnyway } from "../../stores/shutdownStore";
  let continueButton: HTMLButtonElement | undefined = $state();
  $effect(() => {
    const phase = $shutdownState.phase;
    if (!$shutdownBusy && phase !== "idle" && phase !== "committing") {
      void tick().then(() => continueButton?.focus({ preventScroll: true }));
    }
  });
</script>
{#if $shutdownState.phase !== "idle"}
  <div class="shutdown-backdrop">
    <div class="shutdown-dialog" role="dialog" aria-modal="true" aria-label={$t("shutdown.heading")} tabindex="-1" use:focusDialog={{ onEscape: continueBrowsing }} data-testid="shutdown-dialog">
      <h2>{$t($shutdownState.update_version ? "shutdown.update_heading" : "shutdown.heading")}</h2>
      {#if $shutdownState.phase === "review" && $shutdownState.review}
        <p>{$t("shutdown.review")}</p>
        <ul>{#each $shutdownState.review.tabs as tab}<li>{tab.title || tab.url} — {tab.form ? $t("shutdown.form") : ""} {tab.media ? $t("shutdown.media") : ""}</li>{/each}</ul>
        {#if $shutdownState.review.downloads.length}<p>{$t("shutdown.manual", { count: $shutdownState.review.downloads.length })}</p>{/if}
        {#if $shutdownState.review.native_downloads.length}<p>{$t("shutdown.native", { count: $shutdownState.review.native_downloads.length })}</p>{/if}
      {:else}<p role="status">{$t($shutdownState.phase === "committing" ? "shutdown.saving" : "shutdown.preparing")}</p>{/if}
      {#if $shutdownError}<p role="alert">{$errorText($shutdownError)}</p><details><summary>{$t("shutdown.details")}</summary><pre>{technicalError($shutdownError)}</pre></details>{/if}
      <footer>
        <button bind:this={continueButton} data-testid="shutdown-continue" disabled={$shutdownBusy || $shutdownState.phase === "committing"} onclick={continueBrowsing}>{$t("shutdown.continue")}</button>
        {#if $shutdownState.phase === "review" && !$shutdownError}<button data-testid="shutdown-confirm" disabled={$shutdownBusy} onclick={exitAnyway}>{$t($shutdownState.update_version ? "shutdown.install" : "shutdown.exit")}</button>{/if}
        {#if $shutdownError}<button data-testid="shutdown-retry" disabled={$shutdownBusy} onclick={retryShutdown}>{$t("shutdown.retry")}</button>{/if}
      </footer>
    </div>
  </div>
{/if}
<style>
  .shutdown-backdrop { position: fixed; inset: 0; z-index: 100000; display: grid; place-items: center; background: #0008; backdrop-filter: blur(8px); }
  .shutdown-dialog { width: min(560px, 90vw); max-height: 80vh; overflow: auto; padding: 24px; border: 1px solid var(--border-color, #555); border-radius: 16px; background: var(--bg-primary, #202024); color: var(--text-primary, #fff); }
  footer { display: flex; gap: 12px; margin-top: 24px; }
  button { padding: 10px 16px; border-radius: 8px; cursor: pointer; }
  li { overflow-wrap: anywhere; }
</style>
