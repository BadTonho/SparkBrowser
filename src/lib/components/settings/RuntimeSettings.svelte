<script lang="ts">
  import { t } from "../../i18n";
  import { runtimeStatus, runtimeBusy, runtimeError, restartRuntime } from "../../stores/webviewRuntimeStore";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import AppIcon from "../icons/AppIcon.svelte";
</script>

<section class="runtime-settings" aria-labelledby="runtime-heading" data-testid="runtime-settings">
  <header class="runtime-header">
    <span class="runtime-tile" class:attention={$runtimeStatus.restartRequired} aria-hidden="true">
      <AppIcon name="globe" size={17} />
    </span>
    <div class="runtime-titles">
      <h3 id="runtime-heading">{$t("runtime.heading")}</h3>
      <p class="runtime-desc">
        {$t(!$runtimeStatus.supported ? "runtime.unsupported" : $runtimeStatus.restartRequired ? "runtime.notice" : "runtime.managed")}
      </p>
    </div>
  </header>

  {#if $runtimeStatus.currentVersion || $runtimeStatus.restartRequired}
    <div class="runtime-meta">
      {#if $runtimeStatus.currentVersion}
        <p class="runtime-version">{$t("runtime.current", { version: $runtimeStatus.currentVersion })}</p>
      {/if}
      {#if $runtimeStatus.restartRequired}
        <p class="runtime-available">{$t("runtime.available", { version: $runtimeStatus.availableVersion ?? "" })}</p>
      {/if}
    </div>
  {/if}

  {#if $runtimeStatus.restartRequired}
    <div class="runtime-footer">
      <button
        class="pill-action runtime-restart-btn"
        data-testid="runtime-settings-restart"
        disabled={$runtimeBusy}
        onclick={restartRuntime}
      >
        <AppIcon name="refresh" size={14} />
        <span>{$t("runtime.restart")}</span>
      </button>
    </div>
  {/if}

  {#if $runtimeError || $runtimeStatus.error}
    <div class="runtime-error">
      <ErrorMessage error={$runtimeError || $runtimeStatus.error} />
    </div>
  {/if}
</section>

<style>
  .runtime-settings {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 16px;
    border-radius: var(--radius-md);
    border: 1px solid var(--settings-row-border);
    background: var(--settings-row-bg);
    min-width: 0;
  }

  .runtime-header {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    min-width: 0;
  }

  .runtime-tile {
    flex: 0 0 auto;
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    background: color-mix(in srgb, var(--accent-primary) 15%, transparent);
    color: var(--accent-primary);
  }

  .runtime-tile.attention {
    background: color-mix(in srgb, var(--download-status-paused) 18%, transparent);
    color: var(--download-status-paused);
  }

  .runtime-titles {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  h3 {
    margin: 0;
    font-size: 13.5px;
    font-weight: 650;
    color: var(--text-primary);
    line-height: 1.3;
  }

  .runtime-desc {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.5;
  }

  .runtime-meta {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 9px 11px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--text-primary) 3.5%, transparent);
  }

  .runtime-version,
  .runtime-available {
    margin: 0;
    font-size: 12px;
    line-height: 1.45;
    font-variant-numeric: tabular-nums;
  }

  .runtime-version {
    color: var(--text-primary);
    font-weight: 600;
  }

  .runtime-available {
    color: var(--accent-primary);
    font-weight: 600;
  }

  .runtime-footer {
    margin-top: auto;
    display: flex;
    align-items: center;
  }

  .runtime-restart-btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    min-height: 34px;
    padding: 0 15px;
    border-color: transparent;
    background: var(--accent-gradient);
    color: var(--accent-foreground);
    transition: filter 0.15s ease, opacity 0.15s ease;
  }

  .runtime-restart-btn:hover:not(:disabled) {
    filter: brightness(1.06);
  }

  .runtime-restart-btn:disabled {
    opacity: 0.48;
    cursor: not-allowed;
  }

  .runtime-error {
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--download-status-error) 36%, transparent);
    background: color-mix(in srgb, var(--download-status-error) 10%, transparent);
  }

  @media (prefers-reduced-motion: reduce) {
    .runtime-restart-btn {
      transition: none;
    }
  }
</style>
