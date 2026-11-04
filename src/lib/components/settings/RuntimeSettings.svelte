<script lang="ts">
  import { t } from "../../i18n";
  import { runtimeStatus, runtimeBusy, runtimeError, restartRuntime } from "../../stores/webviewRuntimeStore";
  import ErrorMessage from "../status/ErrorMessage.svelte";
</script>
<section class="runtime-settings" aria-labelledby="runtime-heading" data-testid="runtime-settings">
  <h3 id="runtime-heading">{$t("runtime.heading")}</h3>
  {#if $runtimeStatus.currentVersion}<p>{$t("runtime.current", { version: $runtimeStatus.currentVersion })}</p>{/if}
  <p>{$t(!$runtimeStatus.supported ? "runtime.unsupported" : $runtimeStatus.restartRequired ? "runtime.notice" : "runtime.managed")}</p>
  {#if $runtimeStatus.restartRequired}
    <p>{$t("runtime.available", { version: $runtimeStatus.availableVersion ?? "" })}</p>
    <button class="pill-action" data-testid="runtime-settings-restart" disabled={$runtimeBusy} onclick={restartRuntime}>{$t("runtime.restart")}</button>
  {/if}
  <ErrorMessage error={$runtimeError || $runtimeStatus.error} />
</section>
<style>
  section { padding-top: 16px; margin-top: 16px; border-top: 1px solid var(--border-subtle); }
  h3 { font-size: 13px; } p { color: var(--text-secondary); font-size: 12px; line-height: 1.6; margin: 8px 0; }
</style>
