<script lang="ts">
  import { t } from "../../i18n";
  import { runtimeStatus, runtimeBusy, dismissedRuntimeVersion, restartRuntime } from "../../stores/webviewRuntimeStore";
</script>
{#if $runtimeStatus.restartRequired && $runtimeStatus.availableVersion !== $dismissedRuntimeVersion}
  <aside class="runtime-notice" role="status" aria-label={$t("runtime.heading")} data-testid="runtime-notice">
    <span>{$t("runtime.notice")}</span>
    <button data-testid="runtime-restart" disabled={$runtimeBusy} onclick={restartRuntime}>{$t("runtime.restart")}</button>
    <button data-testid="runtime-later" onclick={() => dismissedRuntimeVersion.set($runtimeStatus.availableVersion ?? "")}>{$t("runtime.later")}</button>
  </aside>
{/if}
<style>
  aside { display: flex; flex: 0 0 auto; align-items: center; justify-content: center; flex-wrap: wrap; gap: 10px; padding: 6px 12px; color: var(--text-primary); background: var(--bg-glass-elevated); border-bottom: 1px solid var(--border-subtle); font-size: 12px; }
  button { padding: 4px 8px; border: 1px solid var(--border-subtle); border-radius: 6px; }
  button:focus-visible { outline: 2px solid var(--accent-primary); }
  button:disabled { opacity: .5; }
</style>
