<script lang="ts">
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { t } from "../../i18n";
  import { onMount } from "svelte";
  import { ipcDiagnostics } from "../../services/diagnostics.js";
  import { fetchDiagnostics, fetchRecentErrors, type NativeDiagnostics, type RecentErrorEntry } from "../../services/bridge/diagnostics";
  import { logIgnored } from "../../services/ignored-error";
  import { invoke, isTauriRuntime } from "../../services/bridge/runtime";

  let ipc = $state(ipcDiagnostics.snapshot());
  let native = $state<NativeDiagnostics | null>(null);
  let error = $state("");
  let disposed = false;
  let compactFeeds = $state(false);
  let experimentError = $state("");
  type LevelFilter = "ALL" | RecentErrorEntry["level"];
  const LEVELS: RecentErrorEntry["level"][] = ["CRITICAL", "ERROR", "WARN", "INFO"];
  let problems = $state<RecentErrorEntry[]>([]);
  let logPath = $state("");
  let filter = $state<LevelFilter>("ALL");
  let copied = $state(false);
  const visibleProblems = $derived(filter === "ALL" ? problems : problems.filter(entry => entry.level === filter));
  function levelLabel(level: RecentErrorEntry["level"]): string {
    return $t(level === "CRITICAL" ? "settings.problem_level_critical" : level === "ERROR" ? "settings.problem_level_error" : level === "WARN" ? "settings.problem_level_warning" : "settings.problem_level_info");
  }
  async function loadProblems() {
    try {
      const result = await fetchRecentErrors();
      if (disposed || !result) return;
      problems = result.entries;
      logPath = result.path;
    } catch (failure) {
      logIgnored(failure);
    }
  }
  async function copyProblems() {
    const text = visibleProblems.map(entry => `[${entry.time}] [${entry.level}] ${entry.text}`).join("\n");
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => { copied = false; }, 1800);
    } catch (failure) {
      logIgnored(failure);
    }
  }
  function nativeP95(operation: string): string {
    const values = (native?.metrics.samples ?? []).filter(sample => sample.operation === operation).map(sample => sample.duration_ms).sort((a, b) => a - b);
    return values.length ? values[Math.ceil(values.length * 0.95) - 1].toFixed(2) : "—";
  }
  async function setCompact(enabled: boolean) {
    try { await invoke("set_experimental_flag", { name: "compact_feed_cards", enabled }); compactFeeds = enabled; experimentError = ""; }
    catch { experimentError = $t("settings.experiments_error"); }
  }
  async function resetExperiments() {
    try { await invoke("reset_experimental_flags"); compactFeeds = false; experimentError = ""; }
    catch { experimentError = $t("settings.experiments_error"); }
  }
  async function refresh() {
    void loadProblems();
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
  onMount(() => { void refresh(); if (isTauriRuntime()) void invoke<{compact_feed_cards:boolean}>("get_experimental_flags").then(flags => compactFeeds = flags.compact_feed_cards).catch(() => experimentError = $t("settings.experiments_error")); return () => { disposed = true; }; });
</script>

<section class="setting-group" aria-labelledby="diagnostics-heading">
  <h2 id="diagnostics-heading" class="group-title">{$t("settings.recent_problems")}</h2>
  <p>{$t("settings.recent_problems_desc")}</p>
  <div class="problem-toolbar">
    <div class="timer-pills" role="group" aria-label={$t("settings.recent_problems")}>
      <button class="mini-pill" class:selected={filter === "ALL"} aria-pressed={filter === "ALL"} onclick={() => filter = "ALL"}>{$t("settings.problem_level_all")}</button>
      {#each LEVELS as level (level)}
        <button class="mini-pill" class:selected={filter === level} aria-pressed={filter === level} onclick={() => filter = level}>{levelLabel(level)}</button>
      {/each}
    </div>
    <button class="pill-action" onclick={refresh}>{$t("settings.refresh_0e91")}</button>
    <button class="pill-action" disabled={!visibleProblems.length} onclick={copyProblems}>{copied ? $t("settings.problem_copied") : $t("settings.problem_copy")}</button>
  </div>
  {#if visibleProblems.length}
    <ul class="problem-list" aria-label={$t("settings.recent_problems")}>
      {#each visibleProblems as entry, index (index)}
        <li class="problem problem-{entry.level.toLowerCase()}">
          <div class="problem-head"><span class="problem-level">{levelLabel(entry.level)}</span><time>{entry.time} UTC</time></div>
          <pre>{entry.text}</pre>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="problem-empty">{$t("settings.recent_problems_empty")}</p>
  {/if}
  {#if logPath}<p>{$t("settings.recent_problems_path", { path: logPath })}</p>{/if}

  <h2 class="group-title">{$t("settings.experiments_heading")}</h2>
  <p>{$t("settings.experiments_description")}</p>
  <label><input type="checkbox" checked={compactFeeds} disabled={!isTauriRuntime()} onchange={(event) => { const input = event.currentTarget as HTMLInputElement; const value = input.checked; input.checked = compactFeeds; void setCompact(value); }} /> {$t("settings.experiments_compact_feeds")}</label>
  <button class="pill-action" disabled={!isTauriRuntime()} onclick={resetExperiments}>{$t("settings.experiments_reset")}</button>
  {#if experimentError}<p role="alert">{experimentError}</p>{/if}
  <details class="performance">
  <summary class="group-title">{$t("settings.performance_counters")}</summary>
  <p>{$t("settings.session_counters_stay_in_memory_they_contain_operation_names_and_timings_fb73")}</p>
  {#if error}<ErrorMessage error={error} />{/if}
  {#if native}
    <p>{native.tabs} {$t("settings.tabs_9dcf")} {native.resident_webviews} {$t("settings.resident_webviews_f20f")} {native.protected_tabs} {$t("settings.protected_tabs_d119")}</p>
    <table>
      <caption>{$t("settings.native_work_05cc")}</caption>
      <thead><tr><th>{$t("settings.operation_0f04")}</th><th>{$t("settings.calls_b73a")}</th><th>{$t("settings.failures_7bd3")}</th><th>{$t("settings.rows_written_4bd0")}</th><th>{$t("settings.recent_p95_ms_337e")}</th></tr></thead>
      <tbody>{#each Object.entries(native.metrics.totals) as [operation, total]}
        <tr><td>{operation}</td><td>{total.calls}</td><td>{total.failures}</td><td>{total.affected_rows}</td><td>{nativeP95(operation)}</td></tr>
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
  </details>
</section>

<style>
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th, td { padding: 6px; text-align: left; border-bottom: 1px solid var(--border-subtle); }
  td:first-child { overflow-wrap: anywhere; }
  caption { text-align: left; font-weight: 600; margin: 12px 0; }
  p { color: var(--text-secondary); font-size: 12px; line-height: 1.6; }
  .problem-toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; margin: 8px 0 12px; }
  .problem-list { list-style: none; margin: 0 0 8px; padding: 0; display: flex; flex-direction: column; gap: 6px; max-height: 420px; overflow: auto; }
  .problem { padding: 8px 10px; border: 1px solid var(--border-subtle); border-left-width: 3px; border-radius: 8px; }
  .problem-critical { border-left-color: #ef4444; }
  .problem-error { border-left-color: #f97316; }
  .problem-warn { border-left-color: #eab308; }
  .problem-info { border-left-color: #60a5fa; }
  .problem-head { display: flex; justify-content: space-between; gap: 8px; font-size: 11px; color: var(--text-muted); }
  .problem-level { font-weight: 700; color: var(--text-primary); }
  .problem pre { margin: 4px 0 0; font: 11.5px/1.5 ui-monospace, Consolas, monospace; white-space: pre-wrap; overflow-wrap: anywhere; color: var(--text-primary); }
  .problem-empty { padding: 12px; border: 1px dashed var(--border-subtle); border-radius: 8px; }
  .performance { margin-top: 16px; }
  .performance > summary { cursor: pointer; }
</style>
