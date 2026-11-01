<script lang="ts">
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
  <h2 id="diagnostics-heading" class="group-title">Local Diagnostics</h2>
  <p>Session counters stay in memory. They contain operation names and timings, with no page URLs, titles, arguments or tokens.</p>
  <button class="pill-action" onclick={refresh}>Refresh</button>
  {#if error}<p role="alert">{error}</p>{/if}
  {#if native}
    <p>{native.tabs} tabs · {native.resident_webviews} resident WebViews · {native.protected_tabs} protected tabs</p>
    <table>
      <caption>Native work</caption>
      <thead><tr><th>Operation</th><th>Calls</th><th>Failures</th><th>Rows written</th></tr></thead>
      <tbody>{#each Object.entries(native.metrics.totals) as [operation, total]}
        <tr><td>{operation}</td><td>{total.calls}</td><td>{total.failures}</td><td>{total.affected_rows}</td></tr>
      {/each}</tbody>
    </table>
  {/if}
  <table>
    <caption>Application IPC</caption>
    <thead><tr><th>Command</th><th>Calls</th><th>Errors</th><th>Mean (ms)</th><th>Recent p95 (ms)</th></tr></thead>
    <tbody>{#each ipc as row (row.command)}
      <tr><td>{row.command}</td><td>{row.count}</td><td>{row.errors}</td><td>{row.mean_ms.toFixed(2)}</td><td>{row.p95_recent_ms.toFixed(2)}</td></tr>
    {/each}</tbody>
  </table>
  <p>The p95 uses the most recent 256 IPC samples. WebView count does not measure RAM; compare the full application process tree for memory benchmarks.</p>
</section>

<style>
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th, td { padding: 6px; text-align: left; border-bottom: 1px solid var(--border-subtle); }
  td:first-child { overflow-wrap: anywhere; }
  caption { text-align: left; font-weight: 600; margin: 12px 0; }
  p { color: var(--text-secondary); font-size: 12px; line-height: 1.6; }
</style>
