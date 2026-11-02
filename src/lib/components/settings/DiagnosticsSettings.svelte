<script lang="ts">
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { t } from "../../i18n";
  import { onMount } from "svelte";
  import { ipcDiagnostics } from "../../services/diagnostics.js";
  import { fetchDiagnostics, type NativeDiagnostics } from "../../services/bridge/diagnostics";

  let ipc = $state(ipcDiagnostics.snapshot());
  let native = $state<NativeDiagnostics | null>(null);
  let error = $state("");
  let disposed = false;
  async function refresh() {
    try {
      const result = await fetchDiagnostics();
      if (disposed) return;
      native = result;
      ipc = ipcDiagnostics.snapshot();
      error = "";
    } catch {
      if (!disposed) error = "Could not read local diagnostics";
    }
  }
  onMount(() => { void refresh(); return () => { disposed = true; }; });
</script>

<section class="setting-group" aria-labelledby="diagnostics-heading">
  <h2 id="diagnostics-heading" class="group-title">{$t("settings.local_diagnostics_271b")}</h2>
  <p>{$t("settings.session_counters_stay_in_memory_they_contain_operation_names_and_timings_fb73")}</p>
  <button class="pill-action" onclick={refresh}>{$t("settings.refresh_0e91")}</button>
  {#if error}<ErrorMessage error={error} />{/if}
  {#if native}
    <p>{native.tabs} {$t("settings.tabs_9dcf")} {native.resident_webviews} {$t("settings.resident_webviews_f20f")} {native.protected_tabs} {$t("settings.protected_tabs_d119")}</p>
    <table>
      <caption>{$t("settings.native_work_05cc")}</caption>
      <thead><tr><th>{$t("settings.operation_0f04")}</th><th>{$t("settings.calls_b73a")}</th><th>{$t("settings.failures_7bd3")}</th><th>{$t("settings.rows_written_4bd0")}</th></tr></thead>
      <tbody>{#each Object.entries(native.metrics.totals) as [operation, total]}
        <tr><td>{operation}</td><td>{total.calls}</td><td>{total.failures}</td><td>{total.affected_rows}</td></tr>
      {/each}</tbody>
    </table>
  {/if}
  <table>
    <caption>{$t("settings.application_ipc_d01d")}</caption>
    <thead><tr><th>{$t("settings.command_7131")}</th><th>{$t("settings.calls_b73a")}</th><th>{$t("settings.errors_cb70")}</th><th>{$t("settings.mean_ms_482b")}</th><th>{$t("settings.recent_p95_ms_337e")}</th></tr></thead>
    <tbody>{#each ipc as row (row.command)}
      <tr><td>{row.command}</td><td>{row.count}</td><td>{row.errors}</td><td>{row.mean_ms.toFixed(2)}</td><td>{row.p95_recent_ms.toFixed(2)}</td></tr>
    {/each}</tbody>
  </table>
  <p>{$t("settings.the_p95_uses_the_most_recent_256_ipc_samples_webview_count_does_not_meas_f66b")}</p>
</section>

<style>
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th, td { padding: 6px; text-align: left; border-bottom: 1px solid var(--border-subtle); }
  td:first-child { overflow-wrap: anywhere; }
  caption { text-align: left; font-weight: 600; margin: 12px 0; }
  p { color: var(--text-secondary); font-size: 12px; line-height: 1.6; }
</style>
